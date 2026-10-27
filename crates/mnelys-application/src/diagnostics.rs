//! Local, deterministic diagnosis of a failed launch (plan §19.1, M9 stage one).
//!
//! [`analyze`] is pure: it takes the lines the core already captured and returns the rules
//! that fired, so every rule is reproducible in a unit test against a real crash log.
//! [`DiagnosticService`] only assembles that input from the instance's newest session, its
//! newest crash report and the stored record, then renders the exportable bundle.
//!
//! Every excerpt leaves this module redacted, because the same excerpt is what a bundle
//! carries out of the machine.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use mnelys_domain::{
    CoreResult, DiagnosisReport, DiagnosticEvidence, DiagnosticFinding, DiagnosticId,
    DiagnosticSeverity, DiagnosticsBundle, EvidenceSource, LaunchSession, LaunchState,
};

use crate::accounts::AccountService;
use crate::instances::{InstanceService, InstanceSummary};
use crate::launch::LaunchService;
use crate::paths::GamePaths;
use crate::ports::{Clock, FileStore};
use crate::redact::redact;
use crate::workspace::WorkspaceService;

/// Lines one diagnosis reads from the captured process output. This is the whole buffer
/// the runner keeps, so a full window means older output was already dropped.
const CAPTURED_LOG_LINES: usize = 4000;
/// Excerpts one finding carries.
const MAX_EVIDENCE_PER_FINDING: usize = 3;
/// Longest an excerpt gets before the rest is dropped.
const EXCERPT_CHARS: usize = 360;
/// Lines of game log a bundle repeats below the findings.
const BUNDLE_LOG_LINES: usize = 200;

/// One line the rules may read, with where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLine {
    pub source: EvidenceSource,
    /// Slash separated path below the instance, when the line came from a file.
    pub path: Option<String>,
    /// Position inside the window the core read, not inside the original file.
    pub position: Option<u32>,
    pub text: String,
}

impl SourceLine {
    /// A line of captured process output, counted from zero.
    #[must_use]
    pub fn log(position: usize, text: impl Into<String>) -> Self {
        Self {
            source: EvidenceSource::LaunchLog,
            path: None,
            position: Some(u32::try_from(position).unwrap_or(u32::MAX)),
            text: text.into(),
        }
    }

    /// A line read out of a crash report.
    #[must_use]
    pub fn crash(path: &str, position: usize, text: impl Into<String>) -> Self {
        Self {
            source: EvidenceSource::CrashReport,
            path: Some(path.to_owned()),
            position: Some(u32::try_from(position).unwrap_or(u32::MAX)),
            text: text.into(),
        }
    }
}

/// What one diagnosis looks at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosisInput {
    pub instance: InstanceSummary,
    pub session: Option<LaunchSession>,
    pub lines: Vec<SourceLine>,
    /// Newest crash report the core could read, by its instance-relative path.
    pub crash_report: Option<String>,
    /// True when the captured buffer filled the window.
    pub log_truncated: bool,
}

impl DiagnosisInput {
    /// Whether this run ended badly: the exit code says so, or the runner could not tell.
    #[must_use]
    pub fn failed(&self) -> bool {
        self.session.as_ref().is_some_and(|session| {
            session.state == LaunchState::Failed || session.exit_code.is_some_and(|code| code != 0)
        })
    }
}

/// One text-matching rule.
struct Rule {
    id: DiagnosticId,
    severity: DiagnosticSeverity,
    /// Lowercase fragments; any one of them fires the rule.
    patterns: &'static [&'static str],
    /// Fragments the line must also carry, when a pattern alone is too generic.
    requires: &'static [&'static str],
}

/// The rule table behind plan §19.1. Ordered by how directly a message names its cause,
/// which is the order the findings list explains when two rules fire on one line.
const RULES: &[Rule] = &[
    Rule {
        id: DiagnosticId::JavaVersionMismatch,
        severity: DiagnosticSeverity::Critical,
        patterns: &[
            "unsupportedclassversionerror",
            "has been compiled by a more recent version of the java runtime",
            "requires java",
            "requires at least java",
            "only recognizes class file versions",
        ],
        requires: &[],
    },
    Rule {
        id: DiagnosticId::OutOfMemory,
        severity: DiagnosticSeverity::Critical,
        patterns: &[
            "java.lang.outofmemoryerror",
            "gc overhead limit exceeded",
            "page file is too small",
            "insufficient memory",
        ],
        requires: &[],
    },
    Rule {
        id: DiagnosticId::LoaderInstallIncomplete,
        severity: DiagnosticSeverity::Critical,
        patterns: &[
            "could not find or load main class",
            "no main manifest attribute",
            "filenotfoundexception",
            "zip end of central directory record signature not found",
        ],
        // A `FileNotFoundException` only means a broken install when it names a file the
        // launcher owns. Any other missing file is the game's own business.
        requires: &[".jar", ".json", ".dll", ".so", "natives", "versions/"],
    },
    Rule {
        id: DiagnosticId::MissingDependency,
        severity: DiagnosticSeverity::Critical,
        patterns: &[
            "missing mandatory mod",
            "mandatory dependencies",
            "unresolved dependency",
            "missing dependency",
            "but it is missing",
            "expected range",
        ],
        requires: &[],
    },
    Rule {
        id: DiagnosticId::MixinFailure,
        severity: DiagnosticSeverity::Critical,
        patterns: &[
            "mixin apply failed",
            "mixintransformererror",
            "invalidinjectionexception",
            "mixintransactionerror",
            "an unexpected critical error was encountered",
        ],
        requires: &[],
    },
    Rule {
        id: DiagnosticId::ClassLoadingFailure,
        severity: DiagnosticSeverity::Critical,
        patterns: &[
            "java.lang.noclassdeffounderror",
            "java.lang.classnotfoundexception",
            "could not initialize class",
        ],
        requires: &[],
    },
    Rule {
        id: DiagnosticId::DuplicateMod,
        severity: DiagnosticSeverity::Warning,
        patterns: &[
            "duplicate mods",
            "duplicate mod",
            "duplicatemodsfoundexception",
            "duplicate entries",
        ],
        requires: &[],
    },
    Rule {
        id: DiagnosticId::DiskFull,
        severity: DiagnosticSeverity::Critical,
        patterns: &[
            "no space left on device",
            "not enough space",
            "insufficient space",
            "disk quota exceeded",
            "os error 112",
        ],
        requires: &[],
    },
    Rule {
        id: DiagnosticId::PermissionDenied,
        severity: DiagnosticSeverity::Warning,
        patterns: &[
            "permission denied",
            "access is denied",
            "access denied",
            "operation not permitted",
            "read-only file system",
            "os error 5",
        ],
        requires: &[],
    },
    Rule {
        id: DiagnosticId::NetworkFailure,
        severity: DiagnosticSeverity::Warning,
        patterns: &[
            "connection refused",
            "connection reset",
            "connection timed out",
            "unknownhostexception",
            "no route to host",
            "network is unreachable",
            "name or service not known",
            "certificate_expired",
            "valid certification path",
            "pkix path building failed",
        ],
        requires: &[],
    },
];

/// Runs every rule over one input, then the structural findings the record and the exit
/// status explain.
#[must_use]
pub fn analyze(input: &DiagnosisInput, secrets: &[&str]) -> Vec<DiagnosticFinding> {
    let lowered: Vec<String> = input
        .lines
        .iter()
        .map(|line| line.text.to_lowercase())
        .collect();
    let mut findings: Vec<DiagnosticFinding> = Vec::new();

    for rule in RULES {
        let mut evidence: Vec<DiagnosticEvidence> = Vec::new();
        let mut params = BTreeMap::new();
        for (index, haystack) in lowered.iter().enumerate() {
            if !line_matches(rule, haystack) {
                continue;
            }
            if params.is_empty() {
                params = params_for(rule.id, haystack);
            }
            if evidence.len() == MAX_EVIDENCE_PER_FINDING {
                break;
            }
            let source = &input.lines[index];
            evidence.push(DiagnosticEvidence {
                source: source.source,
                path: source.path.as_ref().map(|path| redact(path, secrets)),
                line: source.position.map(|position| position + 1),
                excerpt: excerpt(&source.text, secrets),
            });
        }
        if !evidence.is_empty() {
            findings.push(DiagnosticFinding {
                id: rule.id,
                severity: rule.severity,
                params,
                evidence,
            });
        }
    }

    findings.extend(config_findings(input, secrets));
    findings.extend(exit_findings(input, secrets));
    if input.failed() && findings.iter().all(|finding| is_structural(finding.id)) {
        findings.push(DiagnosticFinding {
            id: DiagnosticId::UnexplainedFailure,
            severity: DiagnosticSeverity::Notice,
            params: BTreeMap::new(),
            evidence: vec![DiagnosticEvidence {
                source: EvidenceSource::LaunchLog,
                path: None,
                line: None,
                excerpt: excerpt(&format!("lines={} matched=0", input.lines.len()), secrets),
            }],
        });
    }

    findings.sort_by(|left, right| {
        left.severity
            .rank()
            .cmp(&right.severity.rank())
            .then_with(|| left.id.as_str().cmp(right.id.as_str()))
    });
    findings
}

/// Whether only the run's shape, not its text, produced this finding. Used to decide
/// whether anything actually explained the failure.
const fn is_structural(id: DiagnosticId) -> bool {
    matches!(
        id,
        DiagnosticId::AbnormalExit | DiagnosticId::CrashReportPresent
    )
}

/// Whether a lowercased line fires a rule.
fn line_matches(rule: &Rule, haystack: &str) -> bool {
    rule.patterns
        .iter()
        .any(|pattern| haystack.contains(pattern))
        && (rule.requires.is_empty() || rule.requires.iter().any(|need| haystack.contains(need)))
}

/// Values the interface interpolates into its translation.
fn params_for(id: DiagnosticId, haystack: &str) -> BTreeMap<String, String> {
    let mut params = BTreeMap::new();
    match id {
        DiagnosticId::JavaVersionMismatch => {
            for (fragment, key) in [
                ("class file version", "required"),
                ("only recognizes class file versions up to", "supported"),
                ("requires at least java", "required"),
                ("requires java", "required"),
            ] {
                let Some(value) = number_after(haystack, fragment) else {
                    continue;
                };
                let rendered = if fragment.contains("class file version") {
                    class_file_to_java(value).to_string()
                } else {
                    value.to_string()
                };
                params.insert(String::from(key), rendered);
            }
        }
        DiagnosticId::OutOfMemory => {
            if let Some(subtype) = after(haystack, "outofmemoryerror") {
                params.insert(String::from("subtype"), subtype);
            }
        }
        DiagnosticId::ClassLoadingFailure | DiagnosticId::LoaderInstallIncomplete => {
            for fragment in [
                "noclassdeffounderror",
                "classnotfoundexception",
                "could not initialize class",
                "main class",
            ] {
                if let Some(target) = after(haystack, fragment) {
                    params.insert(String::from("target"), target);
                    break;
                }
            }
        }
        DiagnosticId::DuplicateMod | DiagnosticId::MissingDependency => {
            if let Some(name) = quoted_token(haystack) {
                params.insert(String::from("mod"), name);
            }
        }
        _ => {}
    }
    params
}

/// Findings the stored record explains without a log line.
fn config_findings(input: &DiagnosisInput, secrets: &[&str]) -> Vec<DiagnosticFinding> {
    let mut findings = Vec::new();
    if input.session.is_some() && !input.instance.installed {
        findings.push(DiagnosticFinding {
            id: DiagnosticId::LoaderInstallIncomplete,
            severity: DiagnosticSeverity::Critical,
            params: BTreeMap::from([(String::from("target"), input.instance.version_id.clone())]),
            evidence: vec![DiagnosticEvidence {
                source: EvidenceSource::InstanceConfig,
                path: None,
                line: None,
                excerpt: excerpt(
                    &format!("installed=false version_id={}", input.instance.version_id),
                    secrets,
                ),
            }],
        });
    }
    if let Some(path) = &input.crash_report {
        findings.push(DiagnosticFinding {
            id: DiagnosticId::CrashReportPresent,
            severity: DiagnosticSeverity::Warning,
            params: BTreeMap::new(),
            evidence: vec![DiagnosticEvidence {
                source: EvidenceSource::CrashReport,
                path: Some(redact(path, secrets)),
                line: None,
                excerpt: excerpt(path, secrets),
            }],
        });
    }
    findings
}

/// Findings from how the process ended rather than what it printed.
fn exit_findings(input: &DiagnosisInput, secrets: &[&str]) -> Vec<DiagnosticFinding> {
    let Some(session) = input.session.as_ref() else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    if session.state == LaunchState::Failed {
        findings.push(structural_finding(
            DiagnosticId::AbnormalExit,
            DiagnosticSeverity::Warning,
            BTreeMap::new(),
            "state=failed code=unknown",
            secrets,
        ));
        return findings;
    }
    let Some(code) = session.exit_code.filter(|code| *code != 0) else {
        return findings;
    };
    let mut params = BTreeMap::new();
    params.insert(String::from("code"), code.to_string());
    findings.push(structural_finding(
        DiagnosticId::AbnormalExit,
        DiagnosticSeverity::Warning,
        params,
        &format!("state=exited code={code}"),
        secrets,
    ));
    // 137 is SIGKILL, which the Linux and macOS memory limit sends first.
    if code == 137 {
        let mut subtype = BTreeMap::new();
        subtype.insert(String::from("subtype"), String::from("SIGKILL"));
        findings.push(DiagnosticFinding {
            id: DiagnosticId::OutOfMemory,
            severity: DiagnosticSeverity::Warning,
            params: subtype,
            evidence: vec![DiagnosticEvidence {
                source: EvidenceSource::ExitCode,
                path: None,
                line: None,
                excerpt: String::from("state=exited code=137 signal=SIGKILL"),
            }],
        });
    }
    findings
}

/// A finding whose only evidence is how the run ended.
fn structural_finding(
    id: DiagnosticId,
    severity: DiagnosticSeverity,
    params: BTreeMap<String, String>,
    message: &str,
    secrets: &[&str],
) -> DiagnosticFinding {
    DiagnosticFinding {
        id,
        severity,
        params,
        evidence: vec![DiagnosticEvidence {
            source: EvidenceSource::ExitCode,
            path: None,
            line: None,
            excerpt: excerpt(message, secrets),
        }],
    }
}

/// A single-line, redacted excerpt within the display budget.
fn excerpt(text: &str, secrets: &[&str]) -> String {
    let flat = text.trim().replace(['\r', '\n'], " ");
    clip(&redact(&flat, secrets), EXCERPT_CHARS)
}

/// The first `max` characters, with a marker when more was dropped.
fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let kept: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

/// The digits within a few characters after a fragment.
fn number_after(haystack: &str, needle: &str) -> Option<u32> {
    let rest = haystack.split_once(needle)?.1.trim_start();
    let digits: String = rest
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// The trimmed rest of the line after a fragment.
fn after(haystack: &str, needle: &str) -> Option<String> {
    let rest = haystack
        .split_once(needle)?
        .1
        .trim_start_matches([':', '=', ' ']);
    let value: String = rest
        .chars()
        .take_while(|character| !matches!(character, '\r' | '\n' | ',' | ';' | '"'))
        .collect();
    let value = value.trim().to_owned();
    (!value.is_empty()).then(|| clip(&value, 120))
}

/// The first quoted token, which is where the loaders put a mod name.
fn quoted_token(haystack: &str) -> Option<String> {
    for (open, close) in [('\'', '\''), ('"', '"'), ('`', '`')] {
        let Some(start) = haystack.find(open) else {
            continue;
        };
        let Some(end) = haystack[start + 1..].find(close) else {
            continue;
        };
        let token = haystack[start + 1..start + 1 + end].trim();
        if !token.is_empty() && token.len() <= 64 && !token.contains(' ') {
            return Some(token.to_owned());
        }
    }
    None
}

/// A class file major version is 44 below the Java platform it came from.
fn class_file_to_java(class_file: u32) -> u32 {
    class_file.saturating_sub(44)
}

/// Everything the bundle renderer prints, so the text is testable without services.
#[derive(Debug, Clone)]
pub struct BundleContext<'a> {
    pub report: &'a DiagnosisReport,
    pub instance: &'a InstanceSummary,
    pub lines: &'a [SourceLine],
    pub secrets: &'a [&'a str],
    pub os: &'a str,
    pub arch: &'a str,
}

/// Renders the redacted markdown a player can hand to whoever is helping them.
#[must_use]
pub fn render_bundle(context: &BundleContext<'_>) -> String {
    let mut out = String::from("# Mnelys diagnosis bundle\n\n");
    out.push_str("Generated by Mnelys. Credentials, home directories, IP addresses, UUIDs\nand account names are replaced by placeholders before this file is written.\n\n");
    out.push_str(&format!("- platform: {} ({})\n", context.os, context.arch));
    out.push_str(&format!(
        "- generated_at_ms: {}\n\n",
        context.report.generated_at_ms
    ));

    out.push_str("## Instance\n\n");
    out.push_str(&format!("- id: {}\n", context.instance.id));
    out.push_str(&format!(
        "- name: {}\n",
        redact(&context.instance.name, context.secrets)
    ));
    out.push_str(&format!(
        "- gameVersion: {}\n",
        context.instance.game_version
    ));
    out.push_str(&format!(
        "- loader: {} {}\n",
        context.instance.loader.as_str(),
        context.instance.loader_version.as_deref().unwrap_or("-")
    ));
    out.push_str(&format!("- versionId: {}\n", context.instance.version_id));
    out.push_str(&format!("- installed: {}\n", context.instance.installed));
    out.push_str(&format!(
        "- javaPath: {}\n\n",
        context.instance.java_path.as_deref().map_or_else(
            || "(automatic)".to_owned(),
            |path| redact(path, context.secrets)
        )
    ));

    out.push_str("## Last session\n\n");
    match &context.report.session {
        Some(session) => {
            out.push_str(&format!("- state: {}\n", state_label(session.state)));
            out.push_str(&format!(
                "- exitCode: {}\n",
                session.exit_code.map_or_else(
                    || "(still running or unknown)".to_owned(),
                    |code| code.to_string()
                )
            ));
            out.push_str(&format!("- startedAtMs: {}\n", session.started_at_ms));
            out.push_str(&format!(
                "- finishedAtMs: {}\n",
                session
                    .finished_at_ms
                    .map_or_else(|| "-".to_owned(), |value| value.to_string())
            ));
            out.push_str(&format!("- pid: {}\n\n", session.pid));
        }
        None => {
            out.push_str("No launch session is still recorded. The core keeps the last 16.\n\n")
        }
    }

    out.push_str("## Findings\n\n");
    if context.report.findings.is_empty() {
        out.push_str("No rule matched.\n\n");
    }
    for (index, finding) in context.report.findings.iter().enumerate() {
        out.push_str(&format!(
            "{}. `{}` — {}\n",
            index + 1,
            finding.id,
            severity_label(finding.severity)
        ));
        for (key, value) in &finding.params {
            out.push_str(&format!("   - {key}: {value}\n"));
        }
        for evidence in &finding.evidence {
            let location = evidence
                .line
                .map_or_else(String::new, |line| format!(" line {line}"));
            let path = evidence.path.as_deref().unwrap_or("");
            out.push_str(&format!(
                "   - {}{}{}: {}\n",
                source_label(evidence.source),
                if path.is_empty() {
                    String::new()
                } else {
                    format!(" `{path}`")
                },
                location,
                evidence.excerpt
            ));
        }
    }
    out.push('\n');

    out.push_str("## Game log\n\n");
    if context.report.log_truncated {
        out.push_str("Older lines were dropped by the launcher's log buffer.\n\n");
    }
    let log_lines: Vec<&SourceLine> = context
        .lines
        .iter()
        .filter(|line| line.source == EvidenceSource::LaunchLog)
        .rev()
        .take(BUNDLE_LOG_LINES)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    out.push_str("```text\n");
    if log_lines.is_empty() {
        out.push_str("(nothing captured)\n");
    }
    for line in log_lines {
        out.push_str(&redact(&line.text, context.secrets));
        out.push('\n');
    }
    out.push_str("```\n");

    if let Some(path) = &context.report.crash_report {
        out.push_str("\n## Crash report\n\n");
        out.push_str(&format!("`{}`\n\n", redact(path, context.secrets)));
        let crash_lines: Vec<&SourceLine> = context
            .lines
            .iter()
            .filter(|line| line.source == EvidenceSource::CrashReport)
            .collect();
        out.push_str("```text\n");
        if crash_lines.is_empty() {
            out.push_str("(the file could not be read)\n");
        }
        for line in crash_lines {
            out.push_str(&redact(&line.text, context.secrets));
            out.push('\n');
        }
        out.push_str("```\n");
    }
    out
}

/// The lowercase word the bundle prints next to a finding.
const fn severity_label(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Critical => "critical",
        DiagnosticSeverity::Warning => "warning",
        DiagnosticSeverity::Notice => "notice",
    }
}

/// The human-readable origin of one piece of evidence.
const fn source_label(source: EvidenceSource) -> &'static str {
    match source {
        EvidenceSource::LaunchLog => "game log",
        EvidenceSource::CrashReport => "crash report",
        EvidenceSource::ExitCode => "exit status",
        EvidenceSource::InstanceConfig => "instance settings",
    }
}

/// The lowercase name of a session state, for the bundle.
const fn state_label(state: LaunchState) -> &'static str {
    match state {
        LaunchState::Running => "running",
        LaunchState::Exited => "exited",
        LaunchState::Failed => "failed",
        LaunchState::Stopped => "stopped",
    }
}

/// Reads the last launch of an instance and explains it.
pub struct DiagnosticService {
    instances: Arc<InstanceService>,
    launch: Arc<LaunchService>,
    workspace: Arc<WorkspaceService>,
    accounts: Arc<AccountService>,
    files: Arc<dyn FileStore>,
    paths: GamePaths,
    clock: Arc<dyn Clock>,
}

impl DiagnosticService {
    #[must_use]
    pub fn new(
        instances: Arc<InstanceService>,
        launch: Arc<LaunchService>,
        workspace: Arc<WorkspaceService>,
        accounts: Arc<AccountService>,
        files: Arc<dyn FileStore>,
        paths: GamePaths,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            instances,
            launch,
            workspace,
            accounts,
            files,
            paths,
            clock,
        }
    }

    /// Diagnoses the most recent launch attempt of one instance.
    pub async fn diagnose(&self, instance_id: &str) -> CoreResult<DiagnosisReport> {
        let input = self.gather(instance_id).await?;
        let secrets = self.secrets().await;
        let literals: Vec<&str> = secrets.iter().map(String::as_str).collect();
        Ok(DiagnosisReport {
            instance_id: input.instance.id.clone(),
            generated_at_ms: self.clock.now_ms(),
            session: input.session.clone(),
            findings: analyze(&input, &literals),
            analyzed_lines: input.lines.len(),
            log_truncated: input.log_truncated,
            crash_report: input.crash_report.clone(),
        })
    }

    /// Writes a redacted bundle below `<data root>/diagnostics/` and says where it went.
    pub async fn export_bundle(&self, instance_id: &str) -> CoreResult<DiagnosticsBundle> {
        let input = self.gather(instance_id).await?;
        let secrets = self.secrets().await;
        let literals: Vec<&str> = secrets.iter().map(String::as_str).collect();
        let report = DiagnosisReport {
            instance_id: input.instance.id.clone(),
            generated_at_ms: self.clock.now_ms(),
            session: input.session.clone(),
            findings: analyze(&input, &literals),
            analyzed_lines: input.lines.len(),
            log_truncated: input.log_truncated,
            crash_report: input.crash_report.clone(),
        };
        let markdown = render_bundle(&BundleContext {
            report: &report,
            instance: &input.instance,
            lines: &input.lines,
            secrets: &literals,
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
        });
        let path = self.bundle_path(&report);
        self.files.write_atomic(&path, markdown.as_bytes()).await?;
        Ok(DiagnosticsBundle {
            path: path.display().to_string(),
            size_bytes: u64::try_from(markdown.len()).unwrap_or(u64::MAX),
            findings: report.findings.len(),
        })
    }

    /// Collects everything a diagnosis reads. Each part degrades to nothing rather than
    /// failing: an instance that never launched has no session, and a crash report the
    /// game left half-written is still worth showing.
    async fn gather(&self, instance_id: &str) -> CoreResult<DiagnosisInput> {
        let instance = self.instances.get(instance_id).await?;
        let session = self.newest_session(instance_id).await;
        let crash_report = self.newest_crash_report(instance_id).await;
        let mut lines = self.capture_log_lines(session.as_ref()).await;
        let log_truncated = lines.len() >= CAPTURED_LOG_LINES;
        if let Some(path) = &crash_report {
            lines.extend(self.capture_crash_lines(instance_id, path).await);
        }
        Ok(DiagnosisInput {
            instance,
            session,
            lines,
            crash_report,
            log_truncated,
        })
    }

    /// The sessions the runner still holds are newest first, so the first match is the
    /// most recent attempt.
    async fn newest_session(&self, instance_id: &str) -> Option<LaunchSession> {
        self.launch
            .sessions()
            .await
            .ok()?
            .into_iter()
            .find(|session| session.instance_id == instance_id)
    }

    async fn newest_crash_report(&self, instance_id: &str) -> Option<String> {
        self.workspace
            .logs(instance_id)
            .await
            .ok()
            .and_then(|listing| {
                listing
                    .crash_reports
                    .first()
                    .map(|entry| entry.path.clone())
            })
    }

    async fn capture_log_lines(&self, session: Option<&LaunchSession>) -> Vec<SourceLine> {
        let Some(session) = session else {
            return Vec::new();
        };
        let Ok(captured) = self.launch.logs(&session.id, CAPTURED_LOG_LINES).await else {
            return Vec::new();
        };
        captured
            .iter()
            .enumerate()
            .map(|(position, text)| SourceLine::log(position, text.clone()))
            .collect()
    }

    async fn capture_crash_lines(&self, instance_id: &str, path: &str) -> Vec<SourceLine> {
        // `read_log` already enforces the instance sandbox and the tail window.
        let Ok(tail) = self.workspace.read_log(instance_id, path).await else {
            return Vec::new();
        };
        tail.text
            .lines()
            .enumerate()
            .map(|(position, text)| SourceLine::crash(path, position, text))
            .collect()
    }

    fn bundle_path(&self, report: &DiagnosisReport) -> PathBuf {
        self.paths.diagnostics_dir().join(format!(
            "{}-{}.md",
            report.instance_id, report.generated_at_ms
        ))
    }

    /// The selected account's name and id: logs print the player name, and a bundle meant
    /// for someone else should not carry it.
    async fn secrets(&self) -> Vec<String> {
        let mut literals = Vec::new();
        if let Ok(accounts) = self.accounts.list().await {
            for account in accounts.iter().filter(|account| account.selected) {
                literals.push(account.name.clone());
                literals.push(account.uuid.clone());
            }
        }
        literals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(lines: Vec<SourceLine>) -> DiagnosisInput {
        DiagnosisInput {
            instance: InstanceSummary {
                id: String::from("main"),
                name: String::from("Main"),
                game_version: String::from("1.21.4"),
                loader: mnelys_domain::LoaderKind::Fabric,
                loader_version: Some(String::from("0.16.9")),
                version_id: String::from("fabric-loader-0.16.9-1.21.4"),
                created_at_ms: 1,
                last_launched_ms: Some(2),
                java_path: None,
                installed: true,
                directory: String::from("/data/instances/main/game"),
            },
            session: Some(LaunchSession {
                id: String::from("session-1"),
                instance_id: String::from("main"),
                version_id: String::from("fabric-loader-0.16.9-1.21.4"),
                pid: 4242,
                state: LaunchState::Exited,
                started_at_ms: 10,
                finished_at_ms: Some(20),
                exit_code: Some(1),
            }),
            lines,
            crash_report: None,
            log_truncated: false,
        }
    }

    fn ids(findings: &[DiagnosticFinding]) -> Vec<&'static str> {
        findings.iter().map(|finding| finding.id.as_str()).collect()
    }

    /// The one finding a rule produced, so an assertion does not depend on the sort.
    fn only(findings: &[DiagnosticFinding], id: DiagnosticId) -> &DiagnosticFinding {
        findings
            .iter()
            .find(|finding| finding.id == id)
            .unwrap_or_else(|| panic!("no `{}` in {:?}", id.as_str(), ids(findings)))
    }

    #[test]
    fn an_old_java_against_a_newer_class_file_names_both_versions() {
        let found = analyze(
            &input(vec![SourceLine::log(
                7,
                "Exception in thread \"main\" java.lang.UnsupportedClassVersionError: net/minecraft/client/main/Main has been compiled by a more recent version of the Java Runtime (class file version 65.0), this version of the Java Runtime only recognizes class file versions up to 61.0",
            )]),
            &[],
        );
        assert_eq!(ids(&found), ["java-version-mismatch", "abnormal-exit"]);
        let mismatch = only(&found, DiagnosticId::JavaVersionMismatch);
        assert_eq!(mismatch.severity, DiagnosticSeverity::Critical);
        assert_eq!(mismatch.params["required"], "21");
        assert_eq!(mismatch.params["supported"], "17");
        assert_eq!(mismatch.evidence[0].line, Some(8));
    }

    #[test]
    fn a_mod_that_states_its_java_requirement_is_the_same_rule() {
        let found = analyze(
            &input(vec![SourceLine::log(0, "Sodium requires Java 21 or newer")]),
            &[],
        );
        assert_eq!(
            only(&found, DiagnosticId::JavaVersionMismatch).params["required"],
            "21"
        );
    }

    #[test]
    fn an_out_of_memory_print_carries_its_subtype() {
        let found = analyze(
            &input(vec![SourceLine::log(
                3,
                "Exception in thread \"main\" java.lang.OutOfMemoryError: Java heap space",
            )]),
            &[],
        );
        let out_of_memory = only(&found, DiagnosticId::OutOfMemory);
        assert_eq!(out_of_memory.severity, DiagnosticSeverity::Critical);
        assert_eq!(out_of_memory.params["subtype"], "java heap space");
    }

    #[test]
    fn a_system_kill_reads_as_a_memory_suspect_and_an_exit_code() {
        let mut diagnosis = input(Vec::new());
        if let Some(session) = &mut diagnosis.session {
            session.exit_code = Some(137);
        }
        let found = analyze(&diagnosis, &[]);
        assert_eq!(ids(&found), ["abnormal-exit", "out-of-memory"]);
        assert_eq!(
            only(&found, DiagnosticId::OutOfMemory).params["subtype"],
            "SIGKILL"
        );
    }

    #[test]
    fn duplicate_and_missing_mods_quote_their_subject() {
        let duplicated = analyze(
            &input(vec![SourceLine::log(
                0,
                "net.minecraftforge.fml.DuplicateModsFoundException: Duplicate mod 'jei' found",
            )]),
            &[],
        );
        assert_eq!(
            only(&duplicated, DiagnosticId::DuplicateMod).params["mod"],
            "jei"
        );

        let missing = analyze(
            &input(vec![SourceLine::log(
                0,
                "Missing mandatory mods: Mod 'sodium' requires 'fabricloader' but it is missing!",
            )]),
            &[],
        );
        assert_eq!(
            only(&missing, DiagnosticId::MissingDependency).params["mod"],
            "sodium"
        );
    }

    #[test]
    fn a_mixin_and_a_class_loading_failure_are_told_apart() {
        let found = analyze(
            &input(vec![
                SourceLine::log(
                    0,
                    "Mixin apply failed mixins.common.core.json:world.Foo -> Bar",
                ),
                SourceLine::crash(
                    "crash-reports/crash-1.txt",
                    0,
                    "Caused by: java.lang.NoClassDefFoundError: com/example/Missing",
                ),
            ]),
            &[],
        );
        assert!(ids(&found).contains(&"mixin-failure"));
        let loading = only(&found, DiagnosticId::ClassLoadingFailure);
        assert_eq!(loading.params["target"], "com/example/missing");
        assert_eq!(loading.evidence[0].source, EvidenceSource::CrashReport);
        assert_eq!(
            loading.evidence[0].path.as_deref(),
            Some("crash-reports/crash-1.txt")
        );
    }

    #[test]
    fn a_missing_file_only_breaks_the_install_when_it_names_a_launcher_file() {
        let broken = analyze(
            &input(vec![SourceLine::log(
                0,
                "java.io.FileNotFoundException: /data/versions/1.21.4/1.21.4.jar",
            )]),
            &[],
        );
        assert!(ids(&broken).contains(&"loader-install-incomplete"));

        let unrelated = analyze(
            &input(vec![SourceLine::log(
                0,
                "java.io.FileNotFoundException: options.txt",
            )]),
            &[],
        );
        assert!(!ids(&unrelated).contains(&"loader-install-incomplete"));
    }

    #[test]
    fn disk_permission_and_network_lines_each_get_their_own_rule() {
        for (line, expected) in [
            ("java.io.IOException: No space left on device", "disk-full"),
            ("Could not write: Access is denied", "permission-denied"),
            (
                "java.net.ConnectException: Connection refused",
                "network-failure",
            ),
        ] {
            let found = analyze(&input(vec![SourceLine::log(0, line)]), &[]);
            assert!(
                ids(&found).contains(&expected),
                "`{line}` should report {expected}, got {:?}",
                ids(&found)
            );
        }
    }

    #[test]
    fn an_uninstalled_record_is_reported_even_when_the_log_says_nothing() {
        let mut diagnosis = input(Vec::new());
        diagnosis.instance.installed = false;
        let found = analyze(&diagnosis, &[]);
        assert_eq!(found[0].id, DiagnosticId::LoaderInstallIncomplete);
        assert_eq!(found[0].evidence[0].source, EvidenceSource::InstanceConfig);
    }

    #[test]
    fn a_crash_report_is_listed_even_after_a_clean_exit() {
        let mut diagnosis = input(Vec::new());
        if let Some(session) = &mut diagnosis.session {
            session.exit_code = Some(0);
        }
        diagnosis.crash_report = Some(String::from("crash-reports/crash-2.txt"));
        let found = analyze(&diagnosis, &[]);
        assert_eq!(ids(&found), ["crash-report-present"]);
    }

    #[test]
    fn nothing_is_invented_for_a_run_that_succeeded_and_printed_nothing() {
        let mut diagnosis = input(Vec::new());
        if let Some(session) = &mut diagnosis.session {
            session.exit_code = Some(0);
        }
        assert!(analyze(&diagnosis, &[]).is_empty());
    }

    #[test]
    fn a_failure_no_rule_explains_reports_itself_as_unexplained() {
        let found = analyze(
            &input(vec![SourceLine::log(0, "some opaque vendor message")]),
            &[],
        );
        assert!(ids(&found).contains(&"unexplained-failure"));
        assert_eq!(
            found
                .iter()
                .find(|finding| finding.id == DiagnosticId::UnexplainedFailure)
                .expect("finding")
                .severity,
            DiagnosticSeverity::Notice
        );
    }

    #[test]
    fn excerpts_are_redacted_where_the_rule_found_them() {
        let found = analyze(
            &input(vec![SourceLine::log(
                0,
                "java.net.ConnectException: Connection refused reaching /home/steve/.minecraft with token abc123 for steve at 192.168.0.9",
            )]),
            &["steve"],
        );
        let line = &only(&found, DiagnosticId::NetworkFailure).evidence[0];
        assert!(!line.excerpt.contains("abc123"), "{}", line.excerpt);
        assert!(!line.excerpt.contains("steve"), "{}", line.excerpt);
        assert!(
            line.excerpt.contains("<HOME>") && line.excerpt.contains("<IP>"),
            "{}",
            line.excerpt
        );
    }

    #[test]
    fn a_long_line_is_clipped_with_a_marker() {
        let clipped = clip(&"x".repeat(500), EXCERPT_CHARS);
        assert_eq!(clipped.chars().count(), EXCERPT_CHARS);
        assert!(clipped.ends_with('…'));
    }

    #[test]
    fn three_excerpts_are_enough_for_one_finding() {
        let lines = (0..10)
            .map(|index| SourceLine::log(index, "java.lang.OutOfMemoryError: Java heap space"))
            .collect();
        let found = analyze(&input(lines), &[]);
        assert_eq!(found[0].evidence.len(), MAX_EVIDENCE_PER_FINDING);
    }

    #[test]
    fn a_bundle_names_the_platform_the_rules_and_the_log_and_no_secrets() {
        let report = DiagnosisReport {
            instance_id: String::from("main"),
            generated_at_ms: 1_700_000_000_000,
            session: Some(input(Vec::new()).session.clone().expect("session")),
            findings: analyze(
                &input(vec![SourceLine::log(0, "no space left on device")]),
                &[],
            ),
            analyzed_lines: 1,
            log_truncated: false,
            crash_report: Some(String::from("crash-reports/crash-1.txt")),
        };
        let diagnosis = input(vec![SourceLine::log(
            0,
            "Secretive text --accessToken abcdef for Steve in /home/steve/.minecraft",
        )]);
        let markdown = render_bundle(&BundleContext {
            report: &report,
            instance: &diagnosis.instance,
            lines: &diagnosis.lines,
            secrets: &["Steve"],
            os: "windows",
            arch: "x86_64",
        });
        assert!(markdown.contains("## Findings"), "{markdown}");
        assert!(markdown.contains("disk-full"), "{markdown}");
        assert!(markdown.contains("## Crash report"), "{markdown}");
        assert!(markdown.contains("<HOME>"), "{markdown}");
        assert!(!markdown.contains("abcdef"), "{markdown}");
        assert!(!markdown.contains("Steve"), "{markdown}");
    }

    #[test]
    fn a_bundle_says_so_when_nothing_matched() {
        let mut diagnosis = input(Vec::new());
        if let Some(session) = &mut diagnosis.session {
            session.exit_code = Some(0);
        }
        let report = DiagnosisReport {
            instance_id: String::from("main"),
            generated_at_ms: 1,
            session: diagnosis.session.clone(),
            findings: analyze(&diagnosis, &[]),
            analyzed_lines: 0,
            log_truncated: false,
            crash_report: None,
        };
        let markdown = render_bundle(&BundleContext {
            report: &report,
            instance: &diagnosis.instance,
            lines: &diagnosis.lines,
            secrets: &[],
            os: "linux",
            arch: "x86_64",
        });
        assert!(markdown.contains("No rule matched."), "{markdown}");
        assert!(markdown.contains("(nothing captured)"), "{markdown}");
    }

    #[test]
    fn a_class_file_version_maps_to_the_java_that_shipped_it() {
        assert_eq!(class_file_to_java(65), 21);
        assert_eq!(class_file_to_java(61), 17);
        assert_eq!(class_file_to_java(44), 0);
    }
}
