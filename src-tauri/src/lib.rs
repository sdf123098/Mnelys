//! The Mnelys desktop composition root.
//!
//! This crate owns wiring only: it chooses the concrete log store, installs the state
//! the command layer reads, and starts the Tauri runtime. The layers underneath —
//! domain vocabulary, application use-cases, infrastructure adapters and the Tauri
//! command surface — are separate crates so the dependency direction stays visible.

use std::sync::Arc;

use mnelys_application::{LogLevel, LogService};
use mnelys_infrastructure::MemoryLogStore;
use mnelys_presentation::CoreState;

pub use mnelys_presentation::{
    CoreError, CoreErrorCode, CoreResult, COMMAND_NAMES, CONTRACT_VERSION,
};

/// Builds and runs the desktop application.
///
/// # Panics
///
/// Panics if the Tauri runtime cannot be initialized or if the event loop exits with an
/// error. There is no meaningful recovery for a shell that cannot start.
pub fn run() {
    let logs = LogService::new(Arc::new(MemoryLogStore::default()));
    logs.record(
        LogLevel::Info,
        "core",
        format!("Mnelys core {} starting", env!("CARGO_PKG_VERSION")),
    );
    logs.record(
        LogLevel::Debug,
        "core.runtime",
        format!(
            "{} / {} · contract v{}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            CONTRACT_VERSION
        ),
    );
    logs.record(
        LogLevel::Info,
        "core.commands",
        format!("{} commands registered", COMMAND_NAMES.len()),
    );

    let builder = tauri::Builder::default().manage(CoreState::new(logs));

    mnelys_presentation::attach(builder)
        .run(tauri::generate_context!())
        .expect("failed to run the Mnelys desktop shell");
}
