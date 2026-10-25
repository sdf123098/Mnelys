//! A bounded in-memory [`LogStore`].
//!
//! This is the M1 adapter. It is deliberately not a logging framework: the core only
//! needs a stable ring buffer that the task center can read and clear from any thread.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use mnelys_application::{LogEntry, LogLevel, LogStore};
use mnelys_domain::CoreResult;

/// How many entries the session keeps before the oldest are dropped.
pub const DEFAULT_LOG_CAPACITY: usize = 500;

/// A `Mutex<VecDeque>` ring buffer with monotonic ids.
pub struct MemoryLogStore {
    capacity: usize,
    next_id: AtomicU64,
    entries: Mutex<VecDeque<LogEntry>>,
}

impl MemoryLogStore {
    /// Creates a store that keeps at most `capacity` entries.
    ///
    /// # Panics
    ///
    /// Panics when `capacity` is zero: a log that can hold nothing is a wiring bug, and
    /// failing at start-up is clearer than silently discarding every entry.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        assert!(
            capacity > 0,
            "a log store needs room for at least one entry"
        );
        Self {
            capacity,
            next_id: AtomicU64::new(0),
            entries: Mutex::new(VecDeque::with_capacity(capacity.min(64))),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, VecDeque<LogEntry>> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for MemoryLogStore {
    fn default() -> Self {
        Self::new(DEFAULT_LOG_CAPACITY)
    }
}

impl LogStore for MemoryLogStore {
    fn append(&self, level: LogLevel, target: &str, message: &str) -> CoreResult<LogEntry> {
        let entry = LogEntry {
            id: self.next_id.fetch_add(1, Ordering::Relaxed) + 1,
            timestamp_ms: now_ms(),
            level,
            target: target.to_owned(),
            message: message.to_owned(),
        };

        let mut entries = self.lock();
        if entries.len() == self.capacity {
            entries.pop_front();
        }
        entries.push_back(entry.clone());
        Ok(entry)
    }

    fn recent(&self, limit: usize) -> Vec<LogEntry> {
        self.lock().iter().rev().take(limit).cloned().collect()
    }

    fn clear(&self) -> CoreResult<usize> {
        let mut entries = self.lock();
        let removed = entries.len();
        entries.clear();
        Ok(removed)
    }

    fn capacity(&self) -> usize {
        self.capacity
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_are_returned_newest_first() {
        let store = MemoryLogStore::new(4);
        for message in ["first", "second", "third"] {
            store
                .append(LogLevel::Info, "test", message)
                .expect("memory store never fails");
        }

        let entries = store.recent(4);
        let messages: Vec<&str> = entries.iter().map(|entry| entry.message.as_str()).collect();
        assert_eq!(messages, ["third", "second", "first"]);
    }

    #[test]
    fn the_oldest_entry_is_dropped_at_capacity() {
        let store = MemoryLogStore::new(2);
        for message in ["first", "second", "third"] {
            store
                .append(LogLevel::Info, "test", message)
                .expect("memory store never fails");
        }

        let entries = store.recent(2);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].message, "third");
        assert_eq!(entries[1].message, "second");
    }

    #[test]
    fn ids_stay_monotonic_after_clear() {
        let store = MemoryLogStore::new(4);
        let first = store
            .append(LogLevel::Info, "test", "before")
            .expect("memory store never fails");
        assert_eq!(store.clear().expect("clear never fails"), 1);

        let second = store
            .append(LogLevel::Info, "test", "after")
            .expect("memory store never fails");
        assert!(second.id > first.id);
        assert_eq!(store.recent(4).len(), 1);
    }

    #[test]
    fn an_empty_store_reports_nothing_to_clear() {
        let store = MemoryLogStore::new(4);
        assert_eq!(store.clear().expect("clear never fails"), 0);
        assert!(store.recent(4).is_empty());
    }

    #[test]
    fn entries_carry_a_usable_timestamp() {
        let store = MemoryLogStore::new(4);
        let entry = store
            .append(LogLevel::Warn, "core", "hello")
            .expect("memory store never fails");
        assert!(entry.timestamp_ms > 0);
        assert_eq!(entry.target, "core");
        assert_eq!(entry.level, LogLevel::Warn);
    }

    #[test]
    #[should_panic(expected = "at least one entry")]
    fn a_zero_capacity_store_is_rejected() {
        let _ = MemoryLogStore::new(0);
    }
}
