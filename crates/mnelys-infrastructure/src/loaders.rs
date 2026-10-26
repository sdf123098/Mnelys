//! One loader provider for every supported mod loader.
//!
//! Fabric-family loaders publish metadata; Forge and NeoForge only publish installer
//! jars. The facade routes each call so the application depends on one port.

use std::sync::Arc;

use async_trait::async_trait;
use mnelys_application::ports::{HttpClient, LoaderInstallContext, LoaderProvider};
use mnelys_application::TaskReporter;
use mnelys_domain::{CoreError, CoreResult, LoaderKind, LoaderVersionInfo, VersionDocument};

use crate::fabric::FabricLikeProvider;
use crate::forge::InstallerLoaderProvider;

/// Routes loader calls to the provider that can answer them.
pub struct MetaLoaderProvider {
    fabric: FabricLikeProvider,
    installer: InstallerLoaderProvider,
}

impl MetaLoaderProvider {
    #[must_use]
    pub fn new(client: Arc<dyn HttpClient>) -> Self {
        Self {
            fabric: FabricLikeProvider::new(Arc::clone(&client)),
            installer: InstallerLoaderProvider::new(client),
        }
    }
}

#[async_trait]
impl LoaderProvider for MetaLoaderProvider {
    async fn versions(
        &self,
        loader: LoaderKind,
        game_version: &str,
    ) -> CoreResult<Vec<LoaderVersionInfo>> {
        if loader.uses_metadata_profile() {
            self.fabric.versions(loader, game_version).await
        } else if loader.uses_installer() {
            self.installer.versions(loader, game_version).await
        } else {
            Err(CoreError::invalid_request(format!(
                "{} has no loader versions",
                loader.as_str()
            )))
        }
    }

    async fn profile(
        &self,
        loader: LoaderKind,
        game_version: &str,
        loader_version: &str,
    ) -> CoreResult<VersionDocument> {
        if loader.uses_metadata_profile() {
            self.fabric
                .profile(loader, game_version, loader_version)
                .await
        } else {
            self.installer
                .profile(loader, game_version, loader_version)
                .await
        }
    }

    async fn install(
        &self,
        loader: LoaderKind,
        context: &LoaderInstallContext,
        reporter: &dyn TaskReporter,
    ) -> CoreResult<Option<VersionDocument>> {
        if loader.uses_installer() {
            self.installer.install(loader, context, reporter).await
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Offline;

    #[async_trait]
    impl HttpClient for Offline {
        async fn execute(
            &self,
            request: mnelys_application::ports::HttpRequest,
        ) -> CoreResult<mnelys_application::ports::HttpResponse> {
            Err(CoreError::network(format!("offline: {}", request.url)))
        }
    }

    #[tokio::test]
    async fn vanilla_has_no_loader_versions() {
        let provider = MetaLoaderProvider::new(Arc::new(Offline));
        let error = provider
            .versions(LoaderKind::Vanilla, "1.21.4")
            .await
            .expect_err("vanilla has no loader");
        assert_eq!(error.code, mnelys_domain::CoreErrorCode::InvalidRequest);
    }
}
