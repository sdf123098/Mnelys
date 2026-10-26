//! Minecraft version metadata for the download page.

use mnelys_application::LogLevel;
use mnelys_domain::{CoreResult, VersionManifest};
use tauri::State;

use crate::CoreState;

/// The Mojang version manifest, cached by the metadata service.
#[tauri::command]
pub async fn list_versions(state: State<'_, CoreState>) -> CoreResult<VersionManifest> {
    let manifest = state.metadata().manifest().await?;
    state.logs().record(
        LogLevel::Debug,
        "core.versions",
        format!(
            "version manifest: {} versions (latest release {})",
            manifest.versions.len(),
            manifest.latest.release
        ),
    );
    Ok(manifest)
}
