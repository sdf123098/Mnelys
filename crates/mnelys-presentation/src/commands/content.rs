//! Content search, installation and portable instance manifests.

use std::sync::Arc;

use mnelys_application::ports::ContentSearchRequest;
use mnelys_application::{ExportedInstance, InstalledContent, InstanceSummary, LogLevel};
use mnelys_domain::{
    ContentKind, ContentProviderId, ContentSearchPage, ContentVersion, CoreError, CoreResult,
    TaskKind,
};
use serde::Deserialize;
use tauri::State;

use crate::CoreState;

/// Any content provider that can run a search request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchArgs {
    pub provider: ContentProviderId,
    pub query: String,
    pub kind: ContentKind,
    #[serde(default)]
    pub game_version: Option<String>,
    #[serde(default)]
    pub loader: Option<String>,
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

const fn default_limit() -> u32 {
    20
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersionsArgs {
    pub provider: ContentProviderId,
    pub project_id: String,
    #[serde(default)]
    pub game_version: Option<String>,
    #[serde(default)]
    pub loader: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallContentArgs {
    pub instance_id: String,
    pub provider: ContentProviderId,
    pub project_id: String,
    #[serde(default)]
    pub version_id: Option<String>,
    pub kind: ContentKind,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportArgs {
    pub json: String,
}

#[tauri::command]
pub fn content_providers(state: State<'_, CoreState>) -> Vec<ContentProviderId> {
    state.content().available_providers()
}

#[tauri::command]
pub async fn search_content(
    state: State<'_, CoreState>,
    args: SearchArgs,
) -> CoreResult<ContentSearchPage> {
    state
        .content()
        .search(
            args.provider,
            ContentSearchRequest {
                query: args.query,
                kind: args.kind,
                game_version: args.game_version,
                loader: args.loader,
                offset: args.offset,
                limit: args.limit,
            },
        )
        .await
}

#[tauri::command]
pub async fn content_versions(
    state: State<'_, CoreState>,
    args: ContentVersionsArgs,
) -> CoreResult<Vec<ContentVersion>> {
    state
        .content()
        .versions(
            args.provider,
            &args.project_id,
            args.game_version.as_deref(),
            args.loader.as_deref(),
        )
        .await
}

#[tauri::command]
pub async fn installed_content(
    state: State<'_, CoreState>,
    instance_id: String,
) -> CoreResult<Vec<InstalledContent>> {
    state.content().installed(&instance_id).await
}

/// Installs a project and its required dependencies, returning a task id.
#[tauri::command]
pub async fn install_content(
    state: State<'_, CoreState>,
    args: InstallContentArgs,
) -> CoreResult<String> {
    let summary = state.instances().get(&args.instance_id).await?;
    let content = Arc::clone(state.content());
    let project_id = args.project_id.clone();
    let label = format!("Add {project_id} to {}", summary.name);
    let task = state
        .tasks()
        .spawn(TaskKind::Content, label, move |handle| async move {
            content
                .install(
                    &args.instance_id,
                    args.provider,
                    &args.project_id,
                    args.version_id.as_deref(),
                    args.kind,
                    &handle,
                )
                .await
                .map(|_| ())
        });
    state.logs().record(
        LogLevel::Info,
        "core.content",
        format!("install of {project_id} started as task {task}"),
    );
    Ok(task)
}

#[tauri::command]
pub async fn remove_content(
    state: State<'_, CoreState>,
    instance_id: String,
    project_id: String,
) -> CoreResult<bool> {
    let removed = state.content().remove(&instance_id, &project_id).await?;
    if removed {
        state.logs().record(
            LogLevel::Info,
            "core.content",
            format!("removed {project_id} from `{instance_id}`"),
        );
    }
    Ok(removed)
}

#[tauri::command]
pub async fn export_instance(
    state: State<'_, CoreState>,
    instance_id: String,
) -> CoreResult<ExportedInstance> {
    let exported = state.content().export(&instance_id).await?;
    state.logs().record(
        LogLevel::Info,
        "core.content",
        format!("exported instance `{instance_id}` to {}", exported.path),
    );
    Ok(exported)
}

#[tauri::command]
pub async fn import_instance(
    state: State<'_, CoreState>,
    args: ImportArgs,
) -> CoreResult<InstanceSummary> {
    if args.json.len() > 2 * 1024 * 1024 {
        return Err(CoreError::invalid_request(
            "the import file is larger than 2 MiB",
        ));
    }
    let summary = state.content().import(&args.json).await?;
    state.logs().record(
        LogLevel::Info,
        "core.content",
        format!("imported instance `{}`", summary.id),
    );
    Ok(summary)
}
