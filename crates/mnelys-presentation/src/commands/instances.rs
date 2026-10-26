//! Instance lifecycle and installation.

use std::sync::Arc;

use mnelys_application::{InstanceSummary, LogLevel, UpdateInstance};
use mnelys_domain::{CoreError, CoreResult, InstanceSpec, TaskKind};
use tauri::State;

use crate::CoreState;

#[tauri::command]
pub async fn list_instances(state: State<'_, CoreState>) -> CoreResult<Vec<InstanceSummary>> {
    state.instances().list().await
}

#[tauri::command]
pub async fn get_instance(state: State<'_, CoreState>, id: String) -> CoreResult<InstanceSummary> {
    state.instances().get(&id).await
}

#[tauri::command]
pub async fn create_instance(
    state: State<'_, CoreState>,
    spec: InstanceSpec,
) -> CoreResult<InstanceSummary> {
    let summary = state.instances().create(spec).await?;
    state.logs().record(
        LogLevel::Info,
        "core.instances",
        format!("created instance `{}` ({})", summary.id, summary.version_id),
    );
    Ok(summary)
}

#[tauri::command]
pub async fn update_instance(
    state: State<'_, CoreState>,
    id: String,
    update: UpdateInstance,
) -> CoreResult<InstanceSummary> {
    state.instances().update(&id, update).await
}

/// Removes the record and the whole instance directory.
#[tauri::command]
pub async fn delete_instance(state: State<'_, CoreState>, id: String) -> CoreResult<()> {
    if state
        .launch()
        .sessions()
        .await?
        .iter()
        .any(|session| session.instance_id == id && session.state.is_running())
    {
        return Err(CoreError::conflict(
            "stop the running game before deleting this instance",
        ));
    }
    state.instances().delete(&id).await?;
    state.logs().record(
        LogLevel::Info,
        "core.instances",
        format!("deleted instance `{id}`"),
    );
    Ok(())
}

/// Installs the version, libraries, assets and loader for an instance.
#[tauri::command]
pub async fn install_instance(state: State<'_, CoreState>, id: String) -> CoreResult<String> {
    let summary = state.instances().get(&id).await?;
    let installs = Arc::clone(state.installs());
    let task_id = id.clone();
    let task = state.tasks().spawn(
        TaskKind::Install,
        format!("Install {}", summary.name),
        move |handle| async move { installs.install(&task_id, &handle).await.map(|_| ()) },
    );
    state.logs().record(
        LogLevel::Info,
        "core.instances",
        format!("install of `{id}` started as task {task}"),
    );
    Ok(task)
}
