//! Commands that expose the in-memory core log.

use mnelys_application::{LogEntry, LogLevel};
use mnelys_domain::CoreResult;
use tauri::State;

use crate::CoreState;

/// Returns the newest log entries, newest first.
///
/// `limit` is optional: leaving it out means "everything the store keeps". The service
/// caps larger values at the store's capacity and rejects zero.
#[tauri::command]
pub fn app_logs(state: State<'_, CoreState>, limit: Option<usize>) -> CoreResult<Vec<LogEntry>> {
    state.logs().recent(limit)
}

/// Drops every entry and returns how many were removed.
///
/// The clear itself is recorded afterwards, so the panel shows that the action happened
/// instead of going empty with no explanation.
#[tauri::command]
pub fn clear_app_logs(state: State<'_, CoreState>) -> CoreResult<usize> {
    let removed = state.logs().clear()?;
    state.logs().record(
        LogLevel::Info,
        "core.logs",
        format!("cleared {removed} log entries"),
    );
    Ok(removed)
}
