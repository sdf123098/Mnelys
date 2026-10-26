//! Long-running work the task centre displays.
//!
//! A task is a snapshot, not a future: the core owns the work and the frontend only
//! polls the snapshot. That keeps progress reporting independent of the webview and
//! makes a closed window harmless.

use serde::{Deserialize, Serialize};

/// What a task is doing. The frontend uses this for the row icon and grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskKind {
    Metadata,
    Download,
    Install,
    Launch,
    Java,
    Account,
    Content,
    Import,
    Export,
}

impl TaskKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Metadata => "metadata",
            Self::Download => "download",
            Self::Install => "install",
            Self::Launch => "launch",
            Self::Java => "java",
            Self::Account => "account",
            Self::Content => "content",
            Self::Import => "import",
            Self::Export => "export",
        }
    }
}

/// Lifecycle of one task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl TaskState {
    /// Whether the task will change again.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// How far along a task is.
///
/// `total` is optional because metadata negotiation and dependency resolution have no
/// honest denominator. The UI draws an indeterminate bar in that case instead of
/// inventing one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgress {
    pub current: u64,
    pub total: Option<u64>,
    pub message: String,
}

impl TaskProgress {
    #[must_use]
    pub fn new(current: u64, total: u64, message: impl Into<String>) -> Self {
        Self {
            current,
            total: Some(total),
            message: message.into(),
        }
    }

    #[must_use]
    pub fn indeterminate(current: u64, message: impl Into<String>) -> Self {
        Self {
            current,
            total: None,
            message: message.into(),
        }
    }

    /// Completion between 0 and 1, or `None` when the task has no denominator.
    #[must_use]
    pub fn fraction(&self) -> Option<f64> {
        let total = self.total?;
        if total == 0 {
            return Some(1.0);
        }
        Some((self.current.min(total) as f64) / (total as f64))
    }
}

/// One task as the frontend sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSnapshot {
    pub id: String,
    pub kind: TaskKind,
    pub state: TaskState,
    pub label: String,
    pub progress: TaskProgress,
    pub started_at_ms: u64,
    #[serde(default)]
    pub finished_at_ms: Option<u64>,
    #[serde(default)]
    pub error: Option<String>,
}

impl TaskSnapshot {
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fraction_never_exceeds_one() {
        let progress = TaskProgress::new(12, 10, "downloading");
        assert_eq!(progress.fraction(), Some(1.0));
    }

    #[test]
    fn an_indeterminate_progress_has_no_fraction() {
        assert_eq!(TaskProgress::indeterminate(3, "resolving").fraction(), None);
    }

    #[test]
    fn terminal_states_are_exactly_the_three_final_ones() {
        assert!(!TaskState::Queued.is_terminal());
        assert!(!TaskState::Running.is_terminal());
        assert!(TaskState::Succeeded.is_terminal());
        assert!(TaskState::Failed.is_terminal());
        assert!(TaskState::Cancelled.is_terminal());
    }
}
