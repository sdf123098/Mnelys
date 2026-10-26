//! Java discovery and managed runtime installation.

use mnelys_application::LogLevel;
use mnelys_domain::{CoreResult, JavaRuntime, TaskKind};
use tauri::State;

use crate::CoreState;

#[tauri::command]
pub async fn list_java(state: State<'_, CoreState>) -> CoreResult<Vec<JavaRuntime>> {
    let runtimes = state.java().discover().await?;
    state.logs().record(
        LogLevel::Debug,
        "core.java",
        format!("discovered {} Java runtimes", runtimes.len()),
    );
    Ok(runtimes)
}

/// Downloads and unpacks an Adoptium runtime, returning a task id.
#[tauri::command]
pub async fn install_java(state: State<'_, CoreState>, major: u32) -> CoreResult<String> {
    if !(8..=64).contains(&major) {
        return Err(mnelys_domain::CoreError::invalid_request(
            "Java major version must be between 8 and 64",
        ));
    }
    let java = std::sync::Arc::clone(state.java());
    let task = state.tasks().spawn(
        TaskKind::Java,
        format!("Install Java {major}"),
        move |handle| async move { java.install(major, &handle).await.map(|_| ()) },
    );
    state.logs().record(
        LogLevel::Info,
        "core.java",
        format!("Java {major} installation started as task {task}"),
    );
    Ok(task)
}
