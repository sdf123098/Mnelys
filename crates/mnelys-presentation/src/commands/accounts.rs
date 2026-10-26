//! Account management. Tokens stay behind the application service.

use mnelys_application::LogLevel;
use mnelys_domain::{AccountSummary, CoreResult, DeviceCodeChallenge};
use serde::Deserialize;
use tauri::State;

use crate::CoreState;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineAccountArgs {
    pub username: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YggdrasilAccountArgs {
    pub auth_url: String,
    pub username: String,
    pub password: String,
}

#[tauri::command]
pub async fn list_accounts(state: State<'_, CoreState>) -> CoreResult<Vec<AccountSummary>> {
    state.accounts().list().await
}

#[tauri::command]
pub async fn add_offline_account(
    state: State<'_, CoreState>,
    args: OfflineAccountArgs,
) -> CoreResult<AccountSummary> {
    let summary = state.accounts().add_offline(&args.username).await?;
    state.logs().record(
        LogLevel::Info,
        "core.accounts",
        format!("added offline account `{}`", summary.name),
    );
    Ok(summary)
}

#[tauri::command]
pub async fn add_yggdrasil_account(
    state: State<'_, CoreState>,
    args: YggdrasilAccountArgs,
) -> CoreResult<AccountSummary> {
    let summary = state
        .accounts()
        .add_yggdrasil(&args.auth_url, &args.username, &args.password)
        .await?;
    state.logs().record(
        LogLevel::Info,
        "core.accounts",
        format!("signed in to Yggdrasil as `{}`", summary.name),
    );
    Ok(summary)
}

#[tauri::command]
pub async fn begin_microsoft_login(state: State<'_, CoreState>) -> CoreResult<DeviceCodeChallenge> {
    let challenge = state.accounts().begin_microsoft().await?;
    state.logs().record(
        LogLevel::Info,
        "core.accounts",
        "Microsoft device-code sign-in started",
    );
    Ok(challenge)
}

/// Polls Microsoft until the code is approved. Long-running by design.
#[tauri::command]
pub async fn complete_microsoft_login(
    state: State<'_, CoreState>,
    device_code: String,
    interval: Option<u64>,
) -> CoreResult<AccountSummary> {
    let summary = state
        .accounts()
        .complete_microsoft(&device_code, interval.unwrap_or(5))
        .await?;
    state.logs().record(
        LogLevel::Info,
        "core.accounts",
        format!("signed in to Microsoft as `{}`", summary.name),
    );
    Ok(summary)
}

#[tauri::command]
pub async fn select_account(state: State<'_, CoreState>, id: String) -> CoreResult<AccountSummary> {
    state.accounts().select(&id).await
}

#[tauri::command]
pub async fn remove_account(state: State<'_, CoreState>, id: String) -> CoreResult<()> {
    state.accounts().remove(&id).await?;
    state.logs().record(
        LogLevel::Info,
        "core.accounts",
        format!("removed account `{id}`"),
    );
    Ok(())
}

#[tauri::command]
pub async fn refresh_account(
    state: State<'_, CoreState>,
    id: String,
) -> CoreResult<AccountSummary> {
    state.accounts().refresh(&id).await
}
