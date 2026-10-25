//! Application layer: use-cases plus the ports they need.
//!
//! Nothing here knows about Tauri, the filesystem or a concrete storage engine. The
//! composition root supplies implementations; presentation only calls these services.

pub mod logs;

pub use logs::{LogEntry, LogLevel, LogService, LogStore};
