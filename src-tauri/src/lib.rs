//! The Mnelys desktop core.
//!
//! The core is deliberately thin today: it answers the two commands the shell needs and
//! nothing else. Everything the launcher will eventually do — resolving, downloading,
//! verifying, launching — belongs behind this boundary, not in the webview (ADR 0005).

pub mod error;

mod commands;

pub use commands::{core_version, runtime_info, COMMAND_NAMES, CONTRACT_VERSION};
pub use error::{CoreError, CoreErrorCode, CoreResult};

/// Builds and runs the desktop application.
///
/// # Panics
///
/// Panics if the Tauri runtime cannot be initialized or if the event loop exits with an
/// error. There is no meaningful recovery for a shell that cannot start.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::core_version,
            commands::runtime_info
        ])
        .run(tauri::generate_context!())
        .expect("failed to run the Mnelys desktop shell");
}
