//! Presentation layer: the only place that knows about Tauri.
//!
//! Commands translate a transport request into an application call and translate the
//! result back. They own no state themselves; the composition root installs
//! [`CoreState`] and calls [`attach`] to register this surface.

mod commands;
mod state;

pub use commands::{attach, COMMAND_NAMES, CONTRACT_VERSION};
pub use mnelys_domain::{CoreError, CoreErrorCode, CoreResult};
pub use state::{CoreServices, CoreState};
