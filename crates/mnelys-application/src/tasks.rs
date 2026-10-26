//! The task registry: one place that starts background work and reports it.
//!
//! Commands that take longer than a round trip return a task id immediately and the UI
//! polls [`TaskRegistry::list`]. A task keeps running when the window closes, and its
//! snapshot is the only thing the frontend ever sees.

use std::collections::BTreeMap;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use mnelys_domain::{CoreError, CoreResult, TaskKind, TaskProgress, TaskSnapshot, TaskState};
use uuid::Uuid;

use crate::ports::Clock;

/// How many finished tasks are kept for display.
const RETAINED_TERMINAL_TASKS: usize = 64;

/// Progress reporting handed to a running job.
///
/// Cancellation is cooperative: a job must call [`TaskReporter::check_cancelled`]
/// between units of work. The registry still marks the task cancelled once the job
/// returns.
pub trait TaskReporter: Send + Sync {
    fn progress(&self, current: u64, total: Option<u64>, message: &str);
    fn is_cancelled(&self) -> bool;

    /// Fails with a conflict error once cancellation was requested.
    fn check_cancelled(&self) -> CoreResult<()> {
        if self.is_cancelled() {
            Err(CoreError::conflict("task cancelled"))
        } else {
            Ok(())
        }
    }
}

struct RegistryInner {
    tasks: BTreeMap<String, TaskEntry>,
}

struct TaskEntry {
    snapshot: TaskSnapshot,
    cancelled: Arc<AtomicBool>,
}

/// Owns every background task.
#[derive(Clone)]
pub struct TaskRegistry {
    inner: Arc<Mutex<RegistryInner>>,
    clock: Arc<dyn Clock>,
}

impl TaskRegistry {
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RegistryInner {
                tasks: BTreeMap::new(),
            })),
            clock,
        }
    }

    /// Starts a job on the current Tokio runtime and returns its task id.
    ///
    /// # Panics
    ///
    /// Panics when called outside a Tokio runtime. Every caller is a Tauri command,
    /// which always runs inside the runtime.
    pub fn spawn<F, Fut>(&self, kind: TaskKind, label: impl Into<String>, job: F) -> String
    where
        F: FnOnce(TaskHandle) -> Fut + Send + 'static,
        Fut: Future<Output = CoreResult<()>> + Send + 'static,
    {
        let id = Uuid::new_v4().to_string();
        let started_at_ms = self.clock.now_ms();
        let cancelled = Arc::new(AtomicBool::new(false));
        let snapshot = TaskSnapshot {
            id: id.clone(),
            kind,
            state: TaskState::Running,
            label: label.into(),
            progress: TaskProgress::indeterminate(0, ""),
            started_at_ms,
            finished_at_ms: None,
            error: None,
        };
        {
            let mut inner = self.lock();
            inner.tasks.insert(
                id.clone(),
                TaskEntry {
                    snapshot,
                    cancelled: Arc::clone(&cancelled),
                },
            );
            prune(&mut inner);
        }

        let registry = self.clone();
        let handle = TaskHandle {
            id: id.clone(),
            registry: registry.clone(),
            cancelled,
        };
        let finished_id = id.clone();
        tokio::spawn(async move {
            let result = job(handle).await;
            registry.finish(&finished_id, result);
        });
        id
    }

    /// Every task, newest first.
    #[must_use]
    pub fn list(&self) -> Vec<TaskSnapshot> {
        let inner = self.lock();
        let mut tasks: Vec<TaskSnapshot> = inner
            .tasks
            .values()
            .map(|entry| entry.snapshot.clone())
            .collect();
        tasks.sort_by(|left, right| {
            right
                .started_at_ms
                .cmp(&left.started_at_ms)
                .then_with(|| left.id.cmp(&right.id))
        });
        tasks
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<TaskSnapshot> {
        self.lock()
            .tasks
            .get(id)
            .map(|entry| entry.snapshot.clone())
    }

    /// Requests cancellation. Returns false when the task is unknown or already done.
    #[must_use]
    pub fn cancel(&self, id: &str) -> bool {
        let inner = self.lock();
        match inner.tasks.get(id) {
            Some(entry) if !entry.snapshot.state.is_terminal() => {
                entry.cancelled.store(true, Ordering::SeqCst);
                true
            }
            _ => false,
        }
    }

    fn finish(&self, id: &str, result: CoreResult<()>) {
        let finished_at_ms = self.clock.now_ms();
        let mut inner = self.lock();
        let Some(entry) = inner.tasks.get_mut(id) else {
            return;
        };
        let cancelled = entry.cancelled.load(Ordering::SeqCst);
        entry.snapshot.finished_at_ms = Some(finished_at_ms);
        if cancelled {
            entry.snapshot.state = TaskState::Cancelled;
            entry.snapshot.progress.message = String::from("cancelled");
        } else {
            match result {
                Ok(()) => {
                    entry.snapshot.state = TaskState::Succeeded;
                    if let Some(total) = entry.snapshot.progress.total {
                        entry.snapshot.progress.current = total;
                    }
                }
                Err(error) => {
                    entry.snapshot.state = TaskState::Failed;
                    entry.snapshot.error = Some(error.to_string());
                }
            }
        }
        prune(&mut inner);
    }

    fn lock(&self) -> MutexGuard<'_, RegistryInner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn prune(inner: &mut RegistryInner) {
    let terminal: Vec<String> = inner
        .tasks
        .iter()
        .filter(|(_, entry)| entry.snapshot.state.is_terminal())
        .map(|(id, _)| id.clone())
        .collect();
    if terminal.len() <= RETAINED_TERMINAL_TASKS {
        return;
    }
    let mut ordered = terminal;
    ordered.sort_by_key(|id| {
        inner
            .tasks
            .get(id)
            .and_then(|entry| entry.snapshot.finished_at_ms)
            .unwrap_or(0)
    });
    let remove = ordered.len() - RETAINED_TERMINAL_TASKS;
    for id in ordered.into_iter().take(remove) {
        inner.tasks.remove(&id);
    }
}

/// The reporter a running job receives.
#[derive(Clone)]
pub struct TaskHandle {
    id: String,
    registry: TaskRegistry,
    cancelled: Arc<AtomicBool>,
}

impl TaskHandle {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Adds progress and fails if the task was cancelled.
    pub fn advance(&self, current: u64, total: u64, message: &str) -> CoreResult<()> {
        self.check_cancelled()?;
        self.progress(current, Some(total), message);
        Ok(())
    }
}

impl TaskReporter for TaskHandle {
    fn progress(&self, current: u64, total: Option<u64>, message: &str) {
        let mut inner = self.registry.lock();
        if let Some(entry) = inner.tasks.get_mut(&self.id) {
            if !entry.snapshot.state.is_terminal() {
                entry.snapshot.progress = TaskProgress {
                    current,
                    total,
                    message: message.to_owned(),
                };
            }
        }
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::ports::SystemClock;

    fn registry() -> TaskRegistry {
        TaskRegistry::new(Arc::new(SystemClock))
    }

    #[tokio::test]
    async fn a_successful_job_ends_as_succeeded() {
        let registry = registry();
        let id = registry.spawn(TaskKind::Download, "download", |handle| async move {
            handle.progress(1, Some(2), "half");
            Ok(())
        });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let snapshot = registry.get(&id).expect("task exists");
        assert_eq!(snapshot.state, TaskState::Succeeded);
        assert_eq!(snapshot.progress.current, 2);
    }

    #[tokio::test]
    async fn a_failing_job_records_the_error() {
        let registry = registry();
        let id = registry.spawn(TaskKind::Install, "install", |_handle| async move {
            Err(CoreError::network("offline"))
        });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let snapshot = registry.get(&id).expect("task exists");
        assert_eq!(snapshot.state, TaskState::Failed);
        assert!(snapshot.error.unwrap_or_default().contains("offline"));
    }

    #[tokio::test]
    async fn cancellation_is_cooperative_and_final_state_is_cancelled() {
        let registry = registry();
        let id = registry.spawn(TaskKind::Content, "content", |handle| async move {
            for index in 0..100 {
                handle.check_cancelled()?;
                handle.progress(index, Some(100), "working");
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Ok(())
        });
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert!(registry.cancel(&id));
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        assert_eq!(
            registry.get(&id).map(|task| task.state),
            Some(TaskState::Cancelled)
        );
        assert!(!registry.cancel(&id), "a finished task cannot be cancelled");
    }
}
