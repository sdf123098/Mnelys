//! The Mnelys desktop composition root.
//!
//! This crate owns wiring only: it chooses the concrete adapters, builds the
//! application services the command layer reads, and starts the Tauri runtime. The
//! layers underneath — domain vocabulary, application use-cases, infrastructure
//! adapters and the Tauri command surface — are separate crates so the dependency
//! direction stays visible.

use std::sync::Arc;

use mnelys_application::ports::{
    EntryRevealer, FileStore, GameRunner, HttpClient, InstanceRepository, JavaProvider,
    LoaderProvider, SecretStore,
};
use mnelys_application::{
    AccountService, AppSettings, ContentRegistry, ContentService, DiagnosticService, GamePaths,
    InstallService, InstanceService, LaunchService, LogLevel, LogService, MetadataService,
    MicrosoftConfig, MirrorPolicy, SettingsStore, TaskRegistry, WorkspaceService,
};
use mnelys_infrastructure::{
    default_root, CurseForgeProvider, DesktopRevealer, JsonAccountRepository,
    JsonInstanceRepository, JsonSettingsStore, KeyringSecretStore, LocalFileStore,
    LocalJavaProvider, MemoryLogStore, MetaLoaderProvider, ModrinthProvider, ReqwestHttp,
    TokioGameRunner,
};
use mnelys_presentation::{CoreServices, CoreState};

pub use mnelys_presentation::{
    CoreError, CoreErrorCode, CoreResult, DiagnosticId, COMMAND_NAMES, CONTRACT_VERSION,
};

/// Builds and runs the desktop application.
///
/// # Panics
///
/// Panics if the Tauri runtime cannot be initialized or if the event loop exits with an
/// error. There is no meaningful recovery for a shell that cannot start.
pub fn run() {
    let logs = LogService::new(Arc::new(MemoryLogStore::default()));
    logs.record(
        LogLevel::Info,
        "core",
        format!("Mnelys core {} starting", env!("CARGO_PKG_VERSION")),
    );
    logs.record(
        LogLevel::Debug,
        "core.runtime",
        format!(
            "{} / {} · contract v{}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            CONTRACT_VERSION
        ),
    );

    let state = build_state(&logs);
    logs.record(
        LogLevel::Info,
        "core.commands",
        format!("{} commands registered", COMMAND_NAMES.len()),
    );

    let builder = tauri::Builder::default().manage(state);

    mnelys_presentation::attach(builder)
        .run(tauri::generate_context!())
        .expect("failed to run the Mnelys desktop shell");
}

/// Wires every adapter and service exactly once.
fn build_state(logs: &LogService) -> CoreState {
    let root = default_root();
    let paths = GamePaths::new(&root);
    logs.record(
        LogLevel::Info,
        "core.paths",
        format!("data directory: {}", root.display()),
    );

    let files: Arc<dyn FileStore> = Arc::new(LocalFileStore::new());
    let client: Arc<dyn HttpClient> = match ReqwestHttp::new() {
        Ok(client) => Arc::new(client),
        Err(error) => {
            logs.record(
                LogLevel::Error,
                "core.http",
                format!("HTTP client unavailable: {error}"),
            );
            panic!("the HTTP client could not be created: {error}");
        }
    };
    let clock: Arc<dyn mnelys_application::Clock> = Arc::new(mnelys_application::SystemClock);

    let settings: Arc<dyn SettingsStore> =
        Arc::new(JsonSettingsStore::new(Arc::clone(&files), paths.clone()));
    let stored = match tauri::async_runtime::block_on(settings.load()) {
        Ok(settings) => settings,
        Err(error) => {
            logs.record(
                LogLevel::Warn,
                "core.settings",
                format!("settings could not be read ({error}); using defaults"),
            );
            AppSettings::default()
        }
    };
    logs.record(
        LogLevel::Info,
        "core.settings",
        format!(
            "mirror {:?}, default memory {} MB",
            stored.mirror, stored.memory_mb
        ),
    );
    let mirrors = MirrorPolicy::new(stored.mirror);

    let instance_repo: Arc<dyn InstanceRepository> = Arc::new(JsonInstanceRepository::new(
        Arc::clone(&files),
        paths.clone(),
    ));
    let account_repo = Arc::new(JsonAccountRepository::new(
        Arc::clone(&files),
        paths.clone(),
    ));
    let secrets: Arc<dyn SecretStore> = Arc::new(KeyringSecretStore::new());
    let loaders: Arc<dyn LoaderProvider> = Arc::new(MetaLoaderProvider::new(Arc::clone(&client)));
    let java: Arc<dyn JavaProvider> =
        Arc::new(LocalJavaProvider::new(Arc::clone(&client), paths.clone()));
    let runner: Arc<dyn GameRunner> = Arc::new(TokioGameRunner::new(Arc::clone(&clock)));

    let tasks = TaskRegistry::new(Arc::clone(&clock));
    let instances = Arc::new(InstanceService::new(
        Arc::clone(&instance_repo),
        Arc::clone(&files),
        paths.clone(),
        Arc::clone(&clock),
    ));
    let metadata = Arc::new(MetadataService::new(
        Arc::clone(&client),
        Arc::clone(&files),
        paths.clone(),
        Arc::clone(&clock),
        Arc::clone(&loaders),
        mirrors.clone(),
    ));
    let installs = Arc::new(InstallService::new(
        Arc::clone(&client),
        Arc::clone(&files),
        paths.clone(),
        Arc::clone(&metadata),
        Arc::clone(&instances),
        Arc::clone(&loaders),
        Arc::clone(&java),
        mirrors.clone(),
        Arc::clone(&clock),
    ));
    let microsoft = MicrosoftConfig::from_env();
    logs.record(
        LogLevel::Info,
        "core.accounts",
        if microsoft.client_id.is_some() {
            "Microsoft sign-in configured"
        } else {
            "Microsoft sign-in unavailable: MNELYS_MS_CLIENT_ID is not set"
        },
    );
    let accounts = Arc::new(AccountService::new(
        account_repo,
        secrets,
        Arc::clone(&client),
        Arc::clone(&clock),
        microsoft,
    ));
    let launch = Arc::new(LaunchService::new(
        Arc::clone(&instances),
        Arc::clone(&accounts),
        Arc::clone(&metadata),
        Arc::clone(&java),
        runner,
        Arc::clone(&files),
        paths.clone(),
        Arc::clone(&clock),
    ));

    let mut providers: Vec<Arc<dyn mnelys_application::ports::ContentProvider>> =
        vec![Arc::new(ModrinthProvider::new(Arc::clone(&client)))];
    match CurseForgeProvider::from_env(Arc::clone(&client)) {
        Some(provider) => providers.push(Arc::new(provider)),
        None => logs.record(
            LogLevel::Info,
            "core.content",
            "CurseForge is not configured: MNELYS_CURSEFORGE_API_KEY is not set",
        ),
    }
    let registry = ContentRegistry::new(providers);
    let content = Arc::new(ContentService::new(
        registry,
        Arc::clone(&client),
        Arc::clone(&files),
        paths.clone(),
        Arc::clone(&instances),
        instance_repo,
        Arc::clone(&clock),
    ));

    let reveal: Arc<dyn EntryRevealer> = Arc::new(DesktopRevealer::new());
    let workspace = Arc::new(WorkspaceService::new(
        Arc::clone(&files),
        paths.clone(),
        Arc::clone(&instances),
        reveal,
    ));
    let diagnostics = Arc::new(DiagnosticService::new(
        Arc::clone(&instances),
        Arc::clone(&launch),
        Arc::clone(&workspace),
        Arc::clone(&accounts),
        Arc::clone(&files),
        paths.clone(),
        Arc::clone(&clock),
    ));

    CoreState::new(CoreServices {
        logs: logs.clone(),
        paths,
        tasks,
        instances,
        metadata,
        installs,
        launch,
        accounts,
        content,
        workspace,
        diagnostics,
        java,
        loaders,
        mirrors,
        settings,
        client,
    })
}
