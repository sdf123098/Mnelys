//! Settings and mirror diagnostics.

use mnelys_application::mirrors::{probe_all, MirrorKind, MirrorPreference, MirrorProbe};
use mnelys_application::{AppSettings, LogLevel};
use mnelys_domain::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::CoreState;

/// What the settings page reads back.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub mirror: MirrorPreference,
    pub active_mirror: MirrorKind,
    pub memory_mb: u32,
    /// The data directory, shown so users can find their instances.
    pub root: String,
}

/// A partial settings update. Missing fields keep their stored value.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsUpdate {
    #[serde(default)]
    pub mirror: Option<MirrorPreference>,
    #[serde(default)]
    pub memory_mb: Option<u32>,
}

#[tauri::command]
pub async fn get_settings(state: State<'_, CoreState>) -> CoreResult<SettingsView> {
    let settings = state.settings().load().await?;
    Ok(view(
        &settings,
        state.mirrors().active(),
        &state.paths().root().to_string_lossy(),
    ))
}

/// Saves the patch and applies the mirror choice to the running core immediately.
#[tauri::command]
pub async fn update_settings(
    state: State<'_, CoreState>,
    update: SettingsUpdate,
) -> CoreResult<SettingsView> {
    let mut settings = state.settings().load().await?;
    if let Some(memory_mb) = update.memory_mb {
        if memory_mb != 0 && !(512..=65536).contains(&memory_mb) {
            return Err(CoreError::invalid_request(
                "memory must be 0 (automatic) or between 512 MB and 65536 MB",
            ));
        }
        settings.memory_mb = memory_mb;
    }
    if let Some(mirror) = update.mirror {
        settings.mirror = mirror;
    }
    state.settings().save(&settings).await?;
    state.mirrors().set_preference(settings.mirror);
    state.logs().record(
        LogLevel::Info,
        "core.settings",
        format!(
            "settings saved: mirror={:?}, memory={} MB",
            settings.mirror, settings.memory_mb
        ),
    );
    Ok(view(
        &settings,
        state.mirrors().active(),
        &state.paths().root().to_string_lossy(),
    ))
}

/// Times both Mojang and BMCLAPI so the user can choose with evidence.
#[tauri::command]
pub async fn probe_mirrors(state: State<'_, CoreState>) -> CoreResult<Vec<MirrorProbe>> {
    Ok(probe_all(state.client().as_ref()).await)
}

fn view(settings: &AppSettings, active: MirrorKind, root: &str) -> SettingsView {
    SettingsView {
        mirror: settings.mirror,
        active_mirror: active,
        memory_mb: settings.memory_mb,
        root: root.to_owned(),
    }
}
