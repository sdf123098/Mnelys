//! Loader versions for instance creation.

use mnelys_application::LogLevel;
use mnelys_domain::{CoreResult, LoaderKind, LoaderVersionInfo};
use tauri::State;

use crate::CoreState;

/// Every installable build of a loader for one game version.
#[tauri::command]
pub async fn list_loaders(
    state: State<'_, CoreState>,
    loader: LoaderKind,
    game_version: String,
) -> CoreResult<Vec<LoaderVersionInfo>> {
    let versions = state.loaders().versions(loader, &game_version).await?;
    state.logs().record(
        LogLevel::Debug,
        "core.loaders",
        format!(
            "{} has {} builds for Minecraft {game_version}",
            loader.as_str(),
            versions.len()
        ),
    );
    Ok(versions)
}
