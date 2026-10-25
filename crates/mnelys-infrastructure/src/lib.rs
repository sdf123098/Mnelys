//! Infrastructure adapters.
//!
//! Only adapters live here. They depend on the application ports, never the other way
//! around, so the core stays testable without a webview or a real filesystem.

pub mod memory_log;

pub use memory_log::{MemoryLogStore, DEFAULT_LOG_CAPACITY};
