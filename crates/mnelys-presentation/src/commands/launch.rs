//! Starting, stopping and observing game sessions.

use mnelys_application::{LaunchOptions, LogLevel};
use mnelys_domain::{CoreResult, LaunchSession};
use tauri::State;

use crate::CoreState;

/// Starts an installed instance with the selected account.
///
/// The access token never crosses the IPC boundary: only the session descriptor is
/// returned.
#[tauri::command]
pub async fn launch_instance(
    state: State<'_, CoreState>,
    id: String,
    options: Option<LaunchOptions>,
) -> CoreResult<LaunchSession> {
    let mut options = options.unwrap_or_default();
    if options.memory_mb.is_none() {
        let settings = state.settings().load().await?;
        if settings.memory_mb > 0 {
            options.memory_mb = Some(settings.memory_mb);
        }
    }
    let session = state.launch().launch(&id, options).await?;
    state.logs().record(
        LogLevel::Info,
        "core.launch",
        format!(
            "started instance `{id}` as pid {} (session {})",
            session.pid, session.id
        ),
    );
    Ok(session)
}

#[tauri::command]
pub async fn stop_instance(state: State<'_, CoreState>, session_id: String) -> CoreResult<bool> {
    let stopped = state.launch().stop(&session_id).await?;
    state.logs().record(
        LogLevel::Info,
        "core.launch",
        if stopped {
            format!("requested stop of session {session_id}")
        } else {
            format!("session {session_id} is not running")
        },
    );
    Ok(stopped)
}

#[tauri::command]
pub async fn launch_sessions(state: State<'_, CoreState>) -> CoreResult<Vec<LaunchSession>> {
    state.launch().sessions().await
}

#[tauri::command]
pub async fn launch_logs(
    state: State<'_, CoreState>,
    session_id: String,
    limit: Option<usize>,
) -> CoreResult<Vec<String>> {
    let limit = limit.unwrap_or(400).clamp(1, 4000);
    state.launch().logs(&session_id, limit).await
}
