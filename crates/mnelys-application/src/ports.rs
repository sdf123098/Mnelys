//! The ports every adapter implements.
//!
//! Application services depend on these traits only. Tests supply fakes; the real
//! adapters in `mnelys-infrastructure` supply the network, filesystem, process and
//! keystore. Keeping every port here makes the dependency direction impossible to
//! reverse by accident.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use mnelys_domain::{
    AccountRecord, ContentKind, ContentProviderId, ContentSearchPage, ContentVersion, CoreError,
    CoreResult, InstanceRecord, JavaRuntime, LaunchPlan, LaunchSession, LoaderKind,
    LoaderVersionInfo, TaskKind, TaskProgress, VersionDocument,
};
use serde_json::Value;

use crate::tasks::TaskReporter;

/// One HTTP call.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<HttpBody>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone)]
pub enum HttpBody {
    Json(Value),
    Form(Vec<(String, String)>),
}

impl HttpRequest {
    #[must_use]
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Get,
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    #[must_use]
    pub fn post_json(url: impl Into<String>, body: Value) -> Self {
        Self {
            method: HttpMethod::Post,
            url: url.into(),
            headers: Vec::new(),
            body: Some(HttpBody::Json(body)),
        }
    }

    #[must_use]
    pub fn post_form(url: impl Into<String>, form: Vec<(String, String)>) -> Self {
        Self {
            method: HttpMethod::Post,
            url: url.into(),
            headers: Vec::new(),
            body: Some(HttpBody::Form(form)),
        }
    }

    #[must_use]
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

/// The raw response. Status mapping happens in one place, [`HttpResponse::success_body`].
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Returns the body for 2xx responses and a stable core error otherwise.
    pub fn success_body(self, url: &str) -> CoreResult<Vec<u8>> {
        match self.status {
            200..=299 => Ok(self.body),
            401 | 403 => Err(CoreError::unauthorized(format!(
                "the server refused access to {url}"
            ))
            .with_details(format!("HTTP {}", self.status))),
            404 => Err(CoreError::unavailable(format!("{url} was not found"))),
            409 => Err(CoreError::conflict(format!("{url} rejected the request"))),
            status => Err(CoreError::network(format!(
                "request to {url} failed with HTTP {status}"
            ))),
        }
    }
}

/// The network. Implementations must set a descriptive user agent.
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn execute(&self, request: HttpRequest) -> CoreResult<HttpResponse>;
}

/// Fetches bytes and maps the status.
pub async fn fetch_bytes(client: &dyn HttpClient, url: &str) -> CoreResult<Vec<u8>> {
    client
        .execute(HttpRequest::get(url))
        .await?
        .success_body(url)
}

/// Fetches and parses JSON.
pub async fn fetch_json<T: serde::de::DeserializeOwned>(
    client: &dyn HttpClient,
    url: &str,
) -> CoreResult<T> {
    let body = fetch_bytes(client, url).await?;
    serde_json::from_slice(&body).map_err(|error| {
        CoreError::network(format!("{url} did not return the expected JSON"))
            .with_details(error.to_string())
    })
}

/// Executes a request and parses a JSON response.
pub async fn request_json<T: serde::de::DeserializeOwned>(
    client: &dyn HttpClient,
    request: HttpRequest,
) -> CoreResult<T> {
    let url = request.url.clone();
    let body = client.execute(request).await?.success_body(&url)?;
    serde_json::from_slice(&body).map_err(|error| {
        CoreError::network(format!("{url} did not return the expected JSON"))
            .with_details(error.to_string())
    })
}

/// The filesystem. All writes are atomic (temporary file plus rename).
#[async_trait]
pub trait FileStore: Send + Sync {
    async fn read(&self, path: &Path) -> CoreResult<Vec<u8>>;
    async fn write_atomic(&self, path: &Path, data: &[u8]) -> CoreResult<()>;
    async fn exists(&self, path: &Path) -> bool;
    async fn remove(&self, path: &Path) -> CoreResult<()>;
    async fn remove_dir_all(&self, path: &Path) -> CoreResult<()>;
    async fn create_dir_all(&self, path: &Path) -> CoreResult<()>;
    async fn list(&self, path: &Path) -> CoreResult<Vec<DirEntry>>;
    async fn len(&self, path: &Path) -> Option<u64>;
    async fn copy(&self, from: &Path, to: &Path) -> CoreResult<()>;
    /// Extracts a zip archive, skipping entries whose names start with one of the
    /// excluded prefixes. Returns the number of files written.
    async fn extract_zip(
        &self,
        archive: &Path,
        destination: &Path,
        excluded_prefixes: &[String],
    ) -> CoreResult<usize>;
}

/// One directory entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

/// The platform keystore. Secrets are keyed by account id and never logged.
pub trait SecretStore: Send + Sync {
    fn put(&self, key: &str, secret: &str) -> CoreResult<()>;
    fn get(&self, key: &str) -> CoreResult<Option<String>>;
    fn delete(&self, key: &str) -> CoreResult<()>;
}

/// Wall-clock time, injected so tests are deterministic.
pub trait Clock: Send + Sync {
    fn now_ms(&self) -> u64;
}

/// The real clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Everything an installer-based loader needs from the core.
#[derive(Debug, Clone)]
pub struct LoaderInstallContext {
    pub game_version: String,
    pub loader_version: String,
    pub java_path: PathBuf,
    pub game_paths: crate::paths::GamePaths,
    pub vanilla_jar: PathBuf,
    pub vanilla_version_json: PathBuf,
}

/// Loader metadata providers (Fabric, Quilt, Forge, NeoForge, …).
#[async_trait]
pub trait LoaderProvider: Send + Sync {
    async fn versions(
        &self,
        loader: LoaderKind,
        game_version: &str,
    ) -> CoreResult<Vec<LoaderVersionInfo>>;

    async fn profile(
        &self,
        loader: LoaderKind,
        game_version: &str,
        loader_version: &str,
    ) -> CoreResult<VersionDocument>;

    /// Runs the loader's own installer when the provider needs one.
    ///
    /// Fabric and Quilt merge a profile and never call this. Forge and NeoForge run
    /// processors, so the application hands over a fully prepared Vanilla install.
    async fn install(
        &self,
        loader: LoaderKind,
        context: &LoaderInstallContext,
        reporter: &dyn TaskReporter,
    ) -> CoreResult<Option<VersionDocument>> {
        let _ = (loader, context, reporter);
        Ok(None)
    }
}

/// Java discovery and managed runtime installation.
#[async_trait]
pub trait JavaProvider: Send + Sync {
    async fn discover(&self) -> CoreResult<Vec<JavaRuntime>>;
    async fn install(&self, major: u32, reporter: &dyn TaskReporter) -> CoreResult<JavaRuntime>;
}

/// Starting, stopping and reading game processes.
#[async_trait]
pub trait GameRunner: Send + Sync {
    async fn launch(&self, plan: LaunchPlan) -> CoreResult<LaunchSession>;
    async fn stop(&self, session_id: &str) -> CoreResult<bool>;
    async fn sessions(&self) -> CoreResult<Vec<LaunchSession>>;
    async fn logs(&self, session_id: &str, limit: usize) -> CoreResult<Vec<String>>;
}

/// Persistence for instances.
#[async_trait]
pub trait InstanceRepository: Send + Sync {
    async fn list(&self) -> CoreResult<Vec<InstanceRecord>>;
    async fn get(&self, id: &str) -> CoreResult<Option<InstanceRecord>>;
    async fn save(&self, record: &InstanceRecord) -> CoreResult<()>;
    async fn delete(&self, id: &str) -> CoreResult<()>;
}

/// Persistence for accounts and the selected account.
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn list(&self) -> CoreResult<Vec<AccountRecord>>;
    async fn get(&self, id: &str) -> CoreResult<Option<AccountRecord>>;
    async fn save(&self, record: &AccountRecord) -> CoreResult<()>;
    async fn delete(&self, id: &str) -> CoreResult<()>;
    async fn selected(&self) -> CoreResult<Option<String>>;
    async fn select(&self, id: Option<&str>) -> CoreResult<()>;
}

/// A content platform (Modrinth, CurseForge).
#[async_trait]
pub trait ContentProvider: Send + Sync {
    fn id(&self) -> ContentProviderId;

    async fn search(&self, request: ContentSearchRequest) -> CoreResult<ContentSearchPage>;

    async fn versions(
        &self,
        project_id: &str,
        filter: &ContentVersionFilter,
    ) -> CoreResult<Vec<ContentVersion>>;
}

/// Parameters for one search.
#[derive(Debug, Clone)]
pub struct ContentSearchRequest {
    pub query: String,
    pub kind: ContentKind,
    pub game_version: Option<String>,
    pub loader: Option<String>,
    pub offset: u32,
    pub limit: u32,
}

/// Compatibility filter for a version listing.
#[derive(Debug, Clone, Default)]
pub struct ContentVersionFilter {
    pub game_version: Option<String>,
    pub loader: Option<String>,
}

/// A no-op reporter for work that is already synchronous or too small to display.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullReporter;

impl TaskReporter for NullReporter {
    fn progress(&self, _current: u64, _total: Option<u64>, _message: &str) {}
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// Builds a progress snapshot for callers that need one.
#[must_use]
pub fn progress(current: u64, total: Option<u64>, message: &str) -> TaskProgress {
    TaskProgress {
        current,
        total,
        message: message.to_owned(),
    }
}

/// Maps a task kind to the label the frontend shows when a command did not provide
/// a more specific one.
#[must_use]
pub const fn default_task_label(kind: TaskKind) -> &'static str {
    match kind {
        TaskKind::Metadata => "metadata",
        TaskKind::Download => "download",
        TaskKind::Install => "install",
        TaskKind::Launch => "launch",
        TaskKind::Java => "java",
        TaskKind::Account => "account",
        TaskKind::Content => "content",
        TaskKind::Import => "import",
        TaskKind::Export => "export",
    }
}
