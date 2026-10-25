//! The command surface. Each submodule owns one area of the core.

mod logs;
mod runtime;

pub use runtime::{COMMAND_NAMES, CONTRACT_VERSION};

/// Registers every command on a Tauri builder.
///
/// `generate_handler!` expands here, next to the submodules that declare the commands,
/// so Tauri's hidden per-command macros resolve inside this crate.
#[must_use]
pub fn attach<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        runtime::core_version,
        runtime::runtime_info,
        logs::app_logs,
        logs::clear_app_logs
    ])
}
