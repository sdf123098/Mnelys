//! The core log: a bounded, in-memory record of what this session did.
//!
//! The log is operational data, not product data. It is intentionally memory-only:
//! nothing here is written to disk, so a crash cannot leave a half-written file and the
//! UI never promises persistence it does not have.

use std::sync::Arc;

use mnelys_domain::{CoreError, CoreResult};
use serde::Serialize;

/// Severity of a core log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// The lowercase wire name the frontend switches on.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

/// One entry as the task center displays it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    /// Monotonic within one process, so the UI can key rows without guessing.
    pub id: u64,
    /// Milliseconds since the Unix epoch. Formatting stays a presentation concern.
    pub timestamp_ms: u64,
    pub level: LogLevel,
    /// Subsystem that emitted the entry, for example `core.logs`.
    pub target: String,
    pub message: String,
}

/// Storage port for the core log.
///
/// Implementations must be safe to share across Tauri command threads. `recent` returns
/// newest first because that is the order the task center draws.
pub trait LogStore: Send + Sync {
    fn append(&self, level: LogLevel, target: &str, message: &str) -> CoreResult<LogEntry>;
    fn recent(&self, limit: usize) -> Vec<LogEntry>;
    fn clear(&self) -> CoreResult<usize>;
    fn capacity(&self) -> usize;
}

/// Use-cases over a [`LogStore`].
#[derive(Clone)]
pub struct LogService {
    store: Arc<dyn LogStore>,
}

impl LogService {
    #[must_use]
    pub fn new(store: Arc<dyn LogStore>) -> Self {
        Self { store }
    }

    /// Records an entry and deliberately ignores storage failures.
    ///
    /// Logging must never turn a working command into a failing one; a full or broken
    /// log is a diagnostics problem, not a product error.
    pub fn record(&self, level: LogLevel, target: &str, message: impl AsRef<str>) {
        let _ = self.store.append(level, target, message.as_ref());
    }

    /// Returns the newest entries, capped at the store's capacity.
    ///
    /// `None` means "everything the store keeps". An explicit zero is a caller bug and
    /// is reported instead of silently returning an empty list.
    pub fn recent(&self, limit: Option<usize>) -> CoreResult<Vec<LogEntry>> {
        let capacity = self.store.capacity();
        if capacity == 0 {
            return Err(CoreError::internal("the log store has no capacity"));
        }

        let limit = match limit {
            Some(0) => return Err(CoreError::invalid_request("log limit must be at least 1")),
            Some(requested) => requested.min(capacity),
            None => capacity,
        };

        Ok(self.store.recent(limit))
    }

    /// Drops every entry and reports how many were removed.
    pub fn clear(&self) -> CoreResult<usize> {
        self.store.clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeStore {
        capacity: usize,
    }

    impl LogStore for FakeStore {
        fn append(&self, _: LogLevel, _: &str, _: &str) -> CoreResult<LogEntry> {
            unreachable!("the fake store only exercises queries")
        }

        fn recent(&self, limit: usize) -> Vec<LogEntry> {
            assert!(limit <= self.capacity, "the service must cap the request");
            Vec::new()
        }

        fn clear(&self) -> CoreResult<usize> {
            Ok(0)
        }

        fn capacity(&self) -> usize {
            self.capacity
        }
    }

    fn service(capacity: usize) -> LogService {
        LogService::new(Arc::new(FakeStore { capacity }))
    }

    #[test]
    fn an_absent_limit_reads_the_whole_store() {
        let entries = service(10).recent(None).expect("valid query");
        assert!(entries.is_empty());
    }

    #[test]
    fn a_limit_larger_than_capacity_is_capped() {
        let entries = service(10).recent(Some(50)).expect("valid query");
        assert!(entries.is_empty());
    }

    #[test]
    fn a_zero_limit_is_rejected() {
        let error = service(10).recent(Some(0)).expect_err("zero is invalid");
        assert_eq!(error.code, mnelys_domain::CoreErrorCode::InvalidRequest);
    }

    #[test]
    fn a_store_without_capacity_is_an_internal_error() {
        let error = service(0).recent(None).expect_err("capacity zero");
        assert_eq!(error.code, mnelys_domain::CoreErrorCode::Internal);
    }
}
