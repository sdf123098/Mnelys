//! Local diagnosis of a failed launch (plan §19.1, M9 stage one).
//!
//! Both commands read the instance's most recent session; nothing is sent anywhere. The
//! bundle the second one writes is redacted by the core before it touches the disk.

use mnelys_application::LogLevel;
use mnelys_domain::{CoreResult, DiagnosisReport, DiagnosticsBundle};
use tauri::State;

use crate::CoreState;

#[tauri::command]
pub async fn diagnose_instance(
    state: State<'_, CoreState>,
    instance_id: String,
) -> CoreResult<DiagnosisReport> {
    state.diagnostics().diagnose(&instance_id).await
}

/// Writes a redacted bundle and returns where it went, so the interface shows a real path.
#[tauri::command]
pub async fn export_instance_diagnostics(
    state: State<'_, CoreState>,
    instance_id: String,
) -> CoreResult<DiagnosticsBundle> {
    let bundle = state.diagnostics().export_bundle(&instance_id).await?;
    state.logs().record(
        LogLevel::Info,
        "core.diagnostics",
        format!(
            "wrote a diagnosis bundle for instance `{instance_id}` to {}",
            bundle.path
        ),
    );
    Ok(bundle)
}
