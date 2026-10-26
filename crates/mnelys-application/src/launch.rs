//! Building a launch plan and handing it to the process runner.
//!
//! The plan is the whole contract with the game: java path, classpath, natives,
//! placeholders and memory. Nothing is spawned until every piece has been resolved, so
//! failures here leave no half-started process behind.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use mnelys_domain::{
    render_argument_list, CoreError, CoreResult, InstanceRecord, JavaRuntime, LaunchPlan,
    LaunchSession, MavenCoordinate, Platform, VersionDocument,
};
use serde::Deserialize;

use crate::accounts::AccountService;
use crate::instances::InstanceService;
use crate::metadata::MetadataService;
use crate::paths::GamePaths;
use crate::ports::{Clock, FileStore, GameRunner, JavaProvider};
use crate::util::url_file_name;

/// Per-launch overrides.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchOptions {
    /// Defaults to the selected account.
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub memory_mb: Option<u32>,
}

const DEFAULT_MEMORY_MB: u32 = 2048;

/// Builds launch plans and controls game sessions.
pub struct LaunchService {
    instances: Arc<InstanceService>,
    accounts: Arc<AccountService>,
    metadata: Arc<MetadataService>,
    java: Arc<dyn JavaProvider>,
    runner: Arc<dyn GameRunner>,
    files: Arc<dyn FileStore>,
    paths: GamePaths,
    clock: Arc<dyn Clock>,
}

impl LaunchService {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instances: Arc<InstanceService>,
        accounts: Arc<AccountService>,
        metadata: Arc<MetadataService>,
        java: Arc<dyn JavaProvider>,
        runner: Arc<dyn GameRunner>,
        files: Arc<dyn FileStore>,
        paths: GamePaths,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            instances,
            accounts,
            metadata,
            java,
            runner,
            files,
            paths,
            clock,
        }
    }

    /// Starts an installed instance.
    pub async fn launch(
        &self,
        instance_id: &str,
        options: LaunchOptions,
    ) -> CoreResult<LaunchSession> {
        let plan = self.build_plan(instance_id, &options).await?;
        let session = self.runner.launch(plan).await?;
        self.instances.touch_launch(instance_id).await?;
        Ok(session)
    }

    /// Builds the plan without starting anything.
    pub async fn build_plan(
        &self,
        instance_id: &str,
        options: &LaunchOptions,
    ) -> CoreResult<LaunchPlan> {
        let record = self.instances.record(instance_id).await?;
        if !self
            .files
            .exists(&self.paths.installed_marker(&record.version_id))
            .await
        {
            return Err(CoreError::unavailable(
                "this instance is not installed yet; install it before launching",
            ));
        }
        let document = self.metadata.installed_document(&record.version_id).await?;
        let session = self.accounts.session(options.account_id.as_deref()).await?;
        let java_path = self
            .resolve_java(&record, document.required_java_major())
            .await?;

        let memory_mb = options.memory_mb.unwrap_or(DEFAULT_MEMORY_MB);
        if !(512..=65536).contains(&memory_mb) {
            return Err(CoreError::invalid_request(
                "memory must be between 512 MB and 65536 MB",
            ));
        }

        let classpath = self.classpath(&document);
        if classpath.is_empty() {
            return Err(CoreError::internal(
                "the installed version has no classpath entries",
            ));
        }
        let natives = self.paths.version_natives(&record.version_id);
        self.files.create_dir_all(&natives).await?;
        let game_directory = self.paths.instance_game_dir(&record.spec.id);
        self.files.create_dir_all(&game_directory).await?;

        let assets_index = document.asset_index_id();
        let game_assets = self.paths.virtual_assets_dir(&assets_index);
        let classpath_joined = classpath.join(mnelys_domain::classpath_separator());

        let variables = launch_variables(
            &record,
            &document,
            &session,
            &java_path,
            &classpath_joined,
            &natives,
            &game_directory,
            &self.paths.assets_dir(),
            &game_assets,
        );

        let (jvm_template, game_template) = document.filtered_arguments(Platform::current());
        let mut jvm_arguments = render_argument_list(&jvm_template, &variables);
        let game_arguments = render_argument_list(&game_template, &variables);

        if !jvm_arguments
            .iter()
            .any(|argument| argument.contains("-Djava.library.path"))
        {
            jvm_arguments.insert(
                0,
                format!("-Djava.library.path={}", natives.to_string_lossy()),
            );
        }
        if let Some(logging_path) = self.logging_path(&document) {
            if let Some(client) = document
                .logging
                .as_ref()
                .and_then(|logging| logging.client.as_ref())
            {
                let mut logging_variables = variables.clone();
                logging_variables.insert(
                    String::from("path"),
                    logging_path.to_string_lossy().into_owned(),
                );
                jvm_arguments.insert(
                    0,
                    mnelys_domain::render_arguments(&client.argument, &logging_variables),
                );
            }
        }
        let classpath_present = jvm_arguments
            .iter()
            .any(|argument| argument == &classpath_joined)
            || jvm_arguments
                .windows(2)
                .any(|pair| pair[0] == "-cp" && pair[1] == classpath_joined);
        if !classpath_present {
            jvm_arguments.push(String::from("-cp"));
            jvm_arguments.push(classpath_joined.clone());
        }

        Ok(LaunchPlan {
            instance_id: record.spec.id.clone(),
            version_id: record.version_id.clone(),
            java_path,
            main_class: document
                .main_class
                .clone()
                .ok_or_else(|| CoreError::internal("the installed version has no main class"))?,
            classpath,
            native_path: Some(natives.to_string_lossy().into_owned()),
            game_directory: game_directory.to_string_lossy().into_owned(),
            assets_directory: self.paths.assets_dir().to_string_lossy().into_owned(),
            asset_index: assets_index,
            jvm_arguments,
            game_arguments,
            environment: BTreeMap::new(),
            memory_mb,
        })
    }

    async fn resolve_java(&self, record: &InstanceRecord, required: u32) -> CoreResult<String> {
        if let Some(path) = &record.java_path {
            if self.files.exists(std::path::Path::new(path)).await {
                return Ok(path.clone());
            }
        }
        let mut runtimes: Vec<JavaRuntime> = self.java.discover().await?;
        let available: Vec<String> = runtimes
            .iter()
            .map(|runtime| format!("{} ({})", runtime.major, runtime.vendor))
            .collect();
        runtimes.retain(|runtime| runtime.satisfies(required));
        runtimes.sort_by(|left, right| {
            right
                .major
                .cmp(&left.major)
                .then_with(|| left.rank().cmp(&right.rank()))
        });
        runtimes
            .into_iter()
            .next()
            .map(|runtime| runtime.path)
            .ok_or_else(|| {
                CoreError::unavailable(format!(
                    "Java {required} or newer is required; found: {}",
                    if available.is_empty() {
                        String::from("none")
                    } else {
                        available.join(", ")
                    }
                ))
            })
    }

    fn classpath(&self, document: &VersionDocument) -> Vec<String> {
        let mut seen = BTreeSet::new();
        let mut entries = Vec::new();
        let client = self.paths.version_jar(&document.id);
        if client.exists() {
            entries.push(client.to_string_lossy().into_owned());
        }
        for library in &document.libraries {
            if !mnelys_domain::rules_allow(&library.rules, Platform::current()) {
                continue;
            }
            let path = library
                .downloads
                .as_ref()
                .and_then(|downloads| downloads.artifact.as_ref())
                .and_then(|artifact| artifact.path.clone())
                .or_else(|| {
                    MavenCoordinate::parse(&library.name)
                        .ok()
                        .map(|coordinate| coordinate.repository_path())
                });
            let Some(path) = path else {
                continue;
            };
            if !seen.insert(path.clone()) {
                continue;
            }
            entries.push(
                self.paths
                    .libraries_dir()
                    .join(&path)
                    .to_string_lossy()
                    .into_owned(),
            );
        }
        entries
    }

    fn logging_path(&self, document: &VersionDocument) -> Option<std::path::PathBuf> {
        let client = document.logging.as_ref()?.client.as_ref()?;
        let name = client
            .file
            .id
            .clone()
            .or_else(|| url_file_name(&client.file.url).map(str::to_owned))
            .unwrap_or_else(|| String::from("client.xml"));
        Some(self.paths.log_configs_dir().join(name))
    }

    pub async fn stop(&self, session_id: &str) -> CoreResult<bool> {
        self.runner.stop(session_id).await
    }

    pub async fn sessions(&self) -> CoreResult<Vec<LaunchSession>> {
        self.runner.sessions().await
    }

    pub async fn logs(&self, session_id: &str, limit: usize) -> CoreResult<Vec<String>> {
        self.runner.logs(session_id, limit).await
    }

    /// When the plan was built, for diagnostics.
    #[must_use]
    pub fn now(&self) -> u64 {
        self.clock.now_ms()
    }
}

#[allow(clippy::too_many_arguments)]
fn launch_variables(
    record: &InstanceRecord,
    document: &VersionDocument,
    session: &mnelys_domain::AuthSession,
    java_path: &str,
    classpath: &str,
    natives: &std::path::Path,
    game_directory: &std::path::Path,
    assets_directory: &std::path::Path,
    game_assets: &std::path::Path,
) -> BTreeMap<String, String> {
    let mut variables = BTreeMap::new();
    let insert = |variables: &mut BTreeMap<String, String>, key: &str, value: String| {
        variables.insert(key.to_owned(), value);
    };
    insert(&mut variables, "auth_player_name", session.username.clone());
    insert(&mut variables, "auth_uuid", session.uuid.clone());
    insert(
        &mut variables,
        "auth_access_token",
        session.access_token.clone(),
    );
    insert(
        &mut variables,
        "auth_session",
        format!("token:{}:{}", session.access_token, session.uuid),
    );
    insert(
        &mut variables,
        "auth_xuid",
        session.xuid.clone().unwrap_or_default(),
    );
    insert(&mut variables, "user_type", session.user_type.clone());
    insert(&mut variables, "user_properties", String::from("{}"));
    insert(&mut variables, "clientid", String::new());
    insert(&mut variables, "version_name", record.version_id.clone());
    insert(
        &mut variables,
        "version_type",
        document
            .kind
            .clone()
            .unwrap_or_else(|| String::from("release")),
    );
    insert(
        &mut variables,
        "game_directory",
        game_directory.to_string_lossy().into_owned(),
    );
    insert(
        &mut variables,
        "assets_root",
        assets_directory.to_string_lossy().into_owned(),
    );
    insert(
        &mut variables,
        "assets_index_name",
        document.asset_index_id(),
    );
    insert(
        &mut variables,
        "game_assets",
        game_assets.to_string_lossy().into_owned(),
    );
    insert(
        &mut variables,
        "natives_directory",
        natives.to_string_lossy().into_owned(),
    );
    insert(&mut variables, "classpath", classpath.to_owned());
    insert(
        &mut variables,
        "classpath_separator",
        String::from(mnelys_domain::classpath_separator()),
    );
    insert(&mut variables, "launcher_name", String::from("Mnelys"));
    insert(
        &mut variables,
        "launcher_version",
        String::from(env!("CARGO_PKG_VERSION")),
    );
    insert(&mut variables, "java_path", java_path.to_owned());
    variables
}
