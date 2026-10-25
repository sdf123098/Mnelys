//! The failure surface the core exposes to the frontend.
//!
//! Every failure that crosses the Tauri boundary is a [`CoreError`]: a stable,
//! machine-readable code plus a human-readable message. The frontend mirrors the code
//! list in `src/ipc/errors.ts`, and `tests/contract.rs` fails when the two sides drift
//! (ADR 0006).

use std::fmt;

use serde::{Serialize, Serializer};

/// Stable, machine-readable failure codes.
///
/// The wire form (`core/...`) is a contract: adding a variant is fine, renaming one is
/// not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreErrorCode {
    Unavailable,
    NotImplemented,
    InvalidRequest,
    Io,
    Network,
    Unauthorized,
    Conflict,
    Internal,
}

impl CoreErrorCode {
    /// Every code, in the order the frontend declares them.
    pub const ALL: [Self; 8] = [
        Self::Unavailable,
        Self::NotImplemented,
        Self::InvalidRequest,
        Self::Io,
        Self::Network,
        Self::Unauthorized,
        Self::Conflict,
        Self::Internal,
    ];

    /// The wire representation. Never change a string here without bumping the contract.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "core/unavailable",
            Self::NotImplemented => "core/not-implemented",
            Self::InvalidRequest => "core/invalid-request",
            Self::Io => "core/io",
            Self::Network => "core/network",
            Self::Unauthorized => "core/unauthorized",
            Self::Conflict => "core/conflict",
            Self::Internal => "core/internal",
        }
    }
}

impl fmt::Display for CoreErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for CoreErrorCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// A core failure, shaped exactly as the frontend's `CoreError` expects it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreError {
    pub code: CoreErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl CoreError {
    #[must_use]
    pub fn new(code: CoreErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(CoreErrorCode::Unavailable, message)
    }

    #[must_use]
    pub fn not_implemented(message: impl Into<String>) -> Self {
        Self::new(CoreErrorCode::NotImplemented, message)
    }

    #[must_use]
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(CoreErrorCode::InvalidRequest, message)
    }

    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(CoreErrorCode::Internal, message)
    }

    /// Attaches diagnostic context. Callers must not put secrets in here.
    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for CoreError {}

/// The result type every command returns.
pub type CoreResult<T> = Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_has_a_unique_wire_name() {
        let mut names: Vec<&str> = CoreErrorCode::ALL
            .iter()
            .map(|code| code.as_str())
            .collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "two error codes share a wire name");
    }

    #[test]
    fn a_code_serializes_to_its_wire_name() {
        let json = serde_json::to_string(&CoreErrorCode::NotImplemented).expect("serializes");
        assert_eq!(json, "\"core/not-implemented\"");
    }

    #[test]
    fn an_error_serializes_without_an_empty_details_field() {
        let json = serde_json::to_value(CoreError::unavailable("core is not running"))
            .expect("serializes");
        assert_eq!(
            json,
            serde_json::json!({ "code": "core/unavailable", "message": "core is not running" })
        );
    }

    #[test]
    fn details_are_carried_when_present() {
        let json = serde_json::to_value(CoreError::internal("boom").with_details("thread 3"))
            .expect("serializes");
        assert_eq!(json["details"], "thread 3");
    }
}
