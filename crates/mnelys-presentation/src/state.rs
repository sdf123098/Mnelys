//! State shared by every command invocation.
//!
//! The composition root builds every service and hands the bundle to
//! [`CoreState::new`]. Nothing here depends on an infrastructure adapter: the fields
//! are application services and ports, so the presentation crate stays transport-only.

use std::sync::Arc;

use mnelys_application::mirrors::MirrorPolicy;
use mnelys_application::ports::{HttpClient, JavaProvider, LoaderProvider};
use mnelys_application::{
    AccountService, ContentService, GamePaths, InstallService, InstanceService, LaunchService,
    LogService, MetadataService, SettingsStore, TaskRegistry, WorkspaceService,
};

/// Everything the command surface can reach.
pub struct CoreServices {
    pub logs: LogService,
    pub paths: GamePaths,
    pub tasks: TaskRegistry,
    pub instances: Arc<InstanceService>,
    pub metadata: Arc<MetadataService>,
    pub installs: Arc<InstallService>,
    pub launch: Arc<LaunchService>,
    pub accounts: Arc<AccountService>,
    pub content: Arc<ContentService>,
    pub workspace: Arc<WorkspaceService>,
    pub java: Arc<dyn JavaProvider>,
    pub loaders: Arc<dyn LoaderProvider>,
    pub mirrors: MirrorPolicy,
    pub settings: Arc<dyn SettingsStore>,
    pub client: Arc<dyn HttpClient>,
}

/// What the command layer can reach.
///
/// Cloning is cheap: the bundle is behind one `Arc`, and every service is shared.
#[derive(Clone)]
pub struct CoreState {
    services: Arc<CoreServices>,
}

impl CoreState {
    #[must_use]
    pub fn new(services: CoreServices) -> Self {
        Self {
            services: Arc::new(services),
        }
    }

    #[must_use]
    pub fn logs(&self) -> &LogService {
        &self.services.logs
    }

    #[must_use]
    pub fn paths(&self) -> &GamePaths {
        &self.services.paths
    }

    #[must_use]
    pub fn tasks(&self) -> &TaskRegistry {
        &self.services.tasks
    }

    #[must_use]
    pub fn instances(&self) -> &Arc<InstanceService> {
        &self.services.instances
    }

    #[must_use]
    pub fn metadata(&self) -> &Arc<MetadataService> {
        &self.services.metadata
    }

    #[must_use]
    pub fn installs(&self) -> &Arc<InstallService> {
        &self.services.installs
    }

    #[must_use]
    pub fn launch(&self) -> &Arc<LaunchService> {
        &self.services.launch
    }

    #[must_use]
    pub fn accounts(&self) -> &Arc<AccountService> {
        &self.services.accounts
    }

    #[must_use]
    pub fn content(&self) -> &Arc<ContentService> {
        &self.services.content
    }

    #[must_use]
    pub fn workspace(&self) -> &Arc<WorkspaceService> {
        &self.services.workspace
    }

    #[must_use]
    pub fn java(&self) -> &Arc<dyn JavaProvider> {
        &self.services.java
    }

    #[must_use]
    pub fn loaders(&self) -> &Arc<dyn LoaderProvider> {
        &self.services.loaders
    }

    #[must_use]
    pub fn mirrors(&self) -> &MirrorPolicy {
        &self.services.mirrors
    }

    #[must_use]
    pub fn settings(&self) -> &Arc<dyn SettingsStore> {
        &self.services.settings
    }

    #[must_use]
    pub fn client(&self) -> &Arc<dyn HttpClient> {
        &self.services.client
    }
}
