//! The task centre: snapshots of background work.

use mnelys_domain::{CoreResult, TaskSnapshot};
use tauri::State;

use crate::CoreState;

/// Every task the core knows about, newest first.
#[tauri::command]
pub fn list_tasks(state: State<'_, CoreState>) -> CoreResult<Vec<TaskSnapshot>> {
    Ok(state.tasks().list())
}

/// Requests cancellation. Returns false when the task is unknown or already finished.
#[tauri::command]
pub fn cancel_task(state: State<'_, CoreState>, id: String) -> CoreResult<bool> {
    Ok(state.tasks().cancel(&id))
}
