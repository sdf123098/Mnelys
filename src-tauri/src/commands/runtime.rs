//! Commands that describe the running core.

use serde::Serialize;

use crate::error::CoreResult;

/// Version of the command contract.
///
/// Bump this whenever a request or response shape changes. `src/ipc/commands.ts` mirrors
/// the value, and `tests/contract.rs` fails when the two disagree (ADR 0006).
pub const CONTRACT_VERSION: u32 = 1;

/// Every command name this core exposes, in the order the frontend declares them.
pub const COMMAND_NAMES: [&str; 2] = ["core_version", "runtime_info"];

/// Which engine is rendering the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WebviewFamily {
    #[serde(rename = "webview2")]
    Webview2,
    #[serde(rename = "wkwebview")]
    WkWebview,
    #[serde(rename = "webkitgtk")]
    WebkitGtk,
    #[serde(rename = "unknown")]
    Unknown,
}

/// Response of `core_version`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreVersion {
    pub version: &'static str,
    pub contract_version: u32,
    pub profile: &'static str,
}

/// Response of `runtime_info`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInfo {
    pub os: &'static str,
    pub arch: &'static str,
    pub webview: String,
    pub family: WebviewFamily,
}

/// Reports the core version and the command contract version.
///
/// The shell calls this on start-up, so a contract mismatch is visible instead of
/// silently producing wrong data.
#[tauri::command]
#[must_use]
pub fn core_version() -> CoreVersion {
    CoreVersion {
        version: env!("CARGO_PKG_VERSION"),
        contract_version: CONTRACT_VERSION,
        profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
    }
}

/// Reports the platform and webview facts the runtime panel displays.
#[tauri::command]
pub fn runtime_info() -> CoreResult<RuntimeInfo> {
    Ok(RuntimeInfo {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        webview: tauri::webview_version().unwrap_or_else(|_| String::from("unknown")),
        family: webview_family(),
    })
}

/// The engine Mnelys is compiled for. Resolved at compile time on purpose: the family of
/// the engine cannot change while the binary is running.
const fn webview_family() -> WebviewFamily {
    if cfg!(target_os = "windows") {
        WebviewFamily::Webview2
    } else if cfg!(target_os = "macos") {
        WebviewFamily::WkWebview
    } else if cfg!(target_os = "linux") {
        WebviewFamily::WebkitGtk
    } else {
        WebviewFamily::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_core_reports_its_own_version() {
        let version = core_version();
        assert_eq!(version.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(version.contract_version, CONTRACT_VERSION);
        assert!(version.profile == "debug" || version.profile == "release");
    }

    #[test]
    fn the_command_list_matches_the_generated_handler() {
        assert_eq!(COMMAND_NAMES, ["core_version", "runtime_info"]);
    }

    #[test]
    fn runtime_info_reports_the_platform_it_was_built_for() {
        let info = runtime_info().expect("runtime_info never fails");
        assert_eq!(info.os, std::env::consts::OS);
        assert_eq!(info.arch, std::env::consts::ARCH);
        assert!(!info.webview.is_empty());
    }

    #[test]
    fn core_version_is_serialized_in_camel_case() {
        let json = serde_json::to_value(core_version()).expect("serializes");
        assert!(json.get("contractVersion").is_some());
        assert!(json.get("contract_version").is_none());
    }
}
