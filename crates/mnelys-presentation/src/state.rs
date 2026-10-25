//! State shared by every command invocation.

use mnelys_application::LogService;

/// What the command layer can reach.
///
/// Cloning is cheap: [`LogService`] holds an `Arc` and the store itself is shared.
#[derive(Clone)]
pub struct CoreState {
    logs: LogService,
}

impl CoreState {
    #[must_use]
    pub fn new(logs: LogService) -> Self {
        Self { logs }
    }

    #[must_use]
    pub fn logs(&self) -> &LogService {
        &self.logs
    }
}
