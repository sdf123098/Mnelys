//! Domain vocabulary shared by every core layer.
//!
//! The domain crate deliberately has no I/O and no Tauri dependency: it defines the
//! stable failure vocabulary that infrastructure and presentation translate into.

pub mod error;

pub use error::{CoreError, CoreErrorCode, CoreResult};
