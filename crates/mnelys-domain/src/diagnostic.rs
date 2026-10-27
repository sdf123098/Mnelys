//! The vocabulary for local, deterministic failure diagnosis (M9 stage one).
//!
//! A diagnosis is a [`DiagnosticFinding`]: a stable rule id, a severity and the evidence
//! that fired it. The core never ships diagnosis prose — the frontend translates the id
//! through the ten language catalogs, exactly like the loader and content kinds. That is
//! what keeps a rule change from becoming a translation change, and the other way round.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Serialize, Serializer};

/// Every diagnosis rule the core knows, in the order `src/ipc/commands.ts` mirrors.
///
/// The wire slug is a contract: adding one is fine, renaming one is not, because a
/// renamed id silently loses its translation. `src-tauri/tests/contract.rs` fails when
/// the frontend stops declaring a slug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticId {
    JavaVersionMismatch,
    OutOfMemory,
    DuplicateMod,
    MissingDependency,
    MixinFailure,
    ClassLoadingFailure,
    LoaderInstallIncomplete,
    DiskFull,
    PermissionDenied,
    NetworkFailure,
    AbnormalExit,
    CrashReportPresent,
    UnexplainedFailure,
}

impl DiagnosticId {
    /// Every id, for the contract test and for the frontend's exhaustive label map.
    pub const ALL: [Self; 13] = [
        Self::JavaVersionMismatch,
        Self::OutOfMemory,
        Self::DuplicateMod,
        Self::MissingDependency,
        Self::MixinFailure,
        Self::ClassLoadingFailure,
        Self::LoaderInstallIncomplete,
        Self::DiskFull,
        Self::PermissionDenied,
        Self::NetworkFailure,
        Self::AbnormalExit,
        Self::CrashReportPresent,
        Self::UnexplainedFailure,
    ];

    /// The wire representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::JavaVersionMismatch => "java-version-mismatch",
            Self::OutOfMemory => "out-of-memory",
            Self::DuplicateMod => "duplicate-mod",
            Self::MissingDependency => "missing-dependency",
            Self::MixinFailure => "mixin-failure",
            Self::ClassLoadingFailure => "class-loading-failure",
            Self::LoaderInstallIncomplete => "loader-install-incomplete",
            Self::DiskFull => "disk-full",
            Self::PermissionDenied => "permission-denied",
            Self::NetworkFailure => "network-failure",
            Self::AbnormalExit => "abnormal-exit",
            Self::CrashReportPresent => "crash-report-present",
            Self::UnexplainedFailure => "unexplained-failure",
        }
    }
}

impl fmt::Display for DiagnosticId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for DiagnosticId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// How much attention a finding deserves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticSeverity {
    /// The run cannot succeed while this holds.
    Critical,
    /// A likely cause, or a consequence worth showing next to the cause.
    Warning,
    /// Context only; the interface must not present it as an error.
    Notice,
}

impl DiagnosticSeverity {
    /// Sorts findings so the blocking cause comes first.
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Critical => 0,
            Self::Warning => 1,
            Self::Notice => 2,
        }
    }
}

/// Where one piece of evidence came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceSource {
    /// Output the core captured from the game process.
    LaunchLog,
    /// A file below `crash-reports/`.
    CrashReport,
    /// The exit status of the process.
    ExitCode,
    /// The stored instance record, not a runtime message.
    InstanceConfig,
}

/// One line, file or value that fired a rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticEvidence {
    pub source: EvidenceSource,
    /// Slash separated path below the instance game directory, when the evidence is a file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Redacted before it crosses the command boundary, because this same excerpt is what
    /// a diagnosis bundle carries out of the machine.
    pub excerpt: String,
}

/// One rule that fired.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticFinding {
    pub id: DiagnosticId,
    pub severity: DiagnosticSeverity,
    /// Values the interface interpolates into its translation, keyed by stable names
    /// such as `required` and `supported`. Never carries a secret.
    pub params: BTreeMap<String, String>,
    pub evidence: Vec<DiagnosticEvidence>,
}

/// The diagnosis of one launch attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosisReport {
    pub instance_id: String,
    pub generated_at_ms: u64,
    /// The session the report is about; absent when nothing has been launched yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<crate::LaunchSession>,
    pub findings: Vec<DiagnosticFinding>,
    /// Lines the rules actually read, so the interface can say how much it saw.
    pub analyzed_lines: usize,
    /// True when the captured buffer is longer than the window the core read.
    pub log_truncated: bool,
    /// The newest crash report the core could find, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crash_report: Option<String>,
}

/// Where the redacted bundle was written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsBundle {
    pub path: String,
    pub size_bytes: u64,
    pub findings: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_id_has_a_unique_slug() {
        let mut slugs: Vec<&str> = DiagnosticId::ALL.iter().map(|id| id.as_str()).collect();
        slugs.sort_unstable();
        let count = slugs.len();
        slugs.dedup();
        assert_eq!(slugs.len(), count, "two diagnosis rules share a wire slug");
    }

    #[test]
    fn an_id_serializes_to_its_slug_and_a_finding_stays_camel_case() {
        let json = serde_json::to_value(DiagnosticFinding {
            id: DiagnosticId::OutOfMemory,
            severity: DiagnosticSeverity::Critical,
            params: BTreeMap::from([(String::from("subtype"), String::from("Java heap space"))]),
            evidence: vec![DiagnosticEvidence {
                source: EvidenceSource::LaunchLog,
                path: None,
                line: Some(12),
                excerpt: String::from("java.lang.OutOfMemoryError: Java heap space"),
            }],
        })
        .expect("serializes");
        assert_eq!(json["id"], "out-of-memory");
        assert_eq!(json["severity"], "critical");
        assert_eq!(json["evidence"][0]["source"], "launchLog");
        assert_eq!(json["evidence"][0]["line"], 12);
    }

    #[test]
    fn critical_sorts_before_notice() {
        let mut ranks = [
            DiagnosticSeverity::Notice,
            DiagnosticSeverity::Critical,
            DiagnosticSeverity::Warning,
        ]
        .iter()
        .map(|severity| severity.rank())
        .collect::<Vec<_>>();
        ranks.sort_unstable();
        assert_eq!(ranks, vec![0, 1, 2]);
    }

    #[test]
    fn absent_optionals_are_not_serialized() {
        let json = serde_json::to_value(DiagnosisReport {
            instance_id: String::from("main"),
            generated_at_ms: 1_700_000_000_000,
            session: None,
            findings: Vec::new(),
            analyzed_lines: 0,
            log_truncated: false,
            crash_report: None,
        })
        .expect("serializes");
        assert!(json.get("session").is_none());
        assert!(json.get("crashReport").is_none());
    }
}
