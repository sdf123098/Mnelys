//! Reading, pruning and showing the files one instance owns.
//!
//! Every command here goes through `WorkspaceService`, which refuses anything that does
//! not resolve below `<instance>/game` (see `mnelys_application::workspace`).

use mnelys_application::{FileListing, LogLevel, LogListing, LogTail, WorkspaceEntry, WorldEntry};
use mnelys_domain::{CoreError, CoreResult};
use tauri::State;

use crate::CoreState;

#[tauri::command]
pub async fn list_instance_files(
    state: State<'_, CoreState>,
    instance_id: String,
    dir: Option<String>,
) -> CoreResult<FileListing> {
    state
        .workspace()
        .list(&instance_id, dir.as_deref().unwrap_or(""))
        .await
}

#[tauri::command]
pub async fn list_instance_logs(
    state: State<'_, CoreState>,
    instance_id: String,
) -> CoreResult<LogListing> {
    state.workspace().logs(&instance_id).await
}

#[tauri::command]
pub async fn read_instance_log(
    state: State<'_, CoreState>,
    instance_id: String,
    path: String,
) -> CoreResult<LogTail> {
    state.workspace().read_log(&instance_id, &path).await
}

#[tauri::command]
pub async fn list_instance_worlds(
    state: State<'_, CoreState>,
    instance_id: String,
) -> CoreResult<Vec<WorldEntry>> {
    state.workspace().worlds(&instance_id).await
}

#[tauri::command]
pub async fn list_instance_schematics(
    state: State<'_, CoreState>,
    instance_id: String,
) -> CoreResult<Vec<WorkspaceEntry>> {
    state.workspace().schematics(&instance_id).await
}

/// Deletes one file, or one whole world folder, inside the instance.
#[tauri::command]
pub async fn delete_instance_entry(
    state: State<'_, CoreState>,
    instance_id: String,
    path: String,
) -> CoreResult<()> {
    if state
        .launch()
        .sessions()
        .await?
        .iter()
        .any(|session| session.instance_id == instance_id && session.state.is_running())
    {
        return Err(CoreError::conflict(
            "stop the running game before deleting files from this instance",
        ));
    }
    state.workspace().delete(&instance_id, &path).await?;
    state.logs().record(
        LogLevel::Warn,
        "core.workspace",
        format!("deleted `{path}` from instance `{instance_id}`"),
    );
    Ok(())
}

/// Shows one entry in the desktop's own file manager.
#[tauri::command]
pub async fn reveal_instance_entry(
    state: State<'_, CoreState>,
    instance_id: String,
    path: String,
) -> CoreResult<()> {
    state.workspace().reveal(&instance_id, &path).await
}
