//! Minecraft version metadata, loader profiles and the rules that select artefacts.
//!
//! These types mirror Mojang's metadata documents and the profile documents the loader
//! providers return. They stay here, dependency-free, so the install and launch plans can
//! be unit-tested without a network or a filesystem.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// The Mojang version manifest (`version_manifest_v2.json`).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<ManifestEntry>,
}

/// Newest release and snapshot identifiers.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

/// One entry of the version manifest.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ManifestEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default, rename = "releaseTime")]
    pub release_time: Option<String>,
}

/// One file the version document points at.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DownloadInfo {
    #[serde(default)]
    pub id: Option<String>,
    pub url: String,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub path: Option<String>,
}

/// The asset index pointer in a version document.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub total_size: Option<u64>,
    pub url: String,
}

/// The logging configuration a version ships.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LoggingConfig {
    #[serde(default)]
    pub client: Option<LoggingClient>,
}

/// One logging configuration file and the JVM argument that selects it.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LoggingClient {
    pub argument: String,
    pub file: DownloadInfo,
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
}

/// The Java the version expects.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    #[serde(default)]
    pub component: Option<String>,
    pub major_version: u32,
}

/// A library entry, in either the modern or the legacy shape.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub natives: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub extract: Option<ExtractRule>,
    /// Loader profiles often give a Maven base instead of a download object.
    #[serde(default)]
    pub url: Option<String>,
}

/// The artefacts a modern library entry offers.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<DownloadInfo>,
    #[serde(default)]
    pub classifiers: Option<BTreeMap<String, DownloadInfo>>,
}

/// Native extraction rules.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExtractRule {
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// A rule controlling whether a library or argument applies.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Rule {
    pub action: RuleAction,
    #[serde(default)]
    pub os: Option<OsRule>,
    #[serde(default)]
    pub features: Option<BTreeMap<String, bool>>,
}

/// Whether a matching rule allows or disallows the entry.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Disallow,
}

/// Operating-system conditions on a rule.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

/// An argument, either plain or gated by rules.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ArgumentValue {
    Plain(String),
    Conditional {
        rules: Vec<Rule>,
        value: ArgumentValues,
    },
}

/// One or many strings for a conditional argument.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ArgumentValues {
    One(String),
    Many(Vec<String>),
}

impl ArgumentValues {
    fn for_each(&self, mut visit: impl FnMut(&str)) {
        match self {
            Self::One(value) => visit(value),
            Self::Many(values) => {
                for value in values {
                    visit(value);
                }
            }
        }
    }
}

/// Modern argument blocks.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<ArgumentValue>,
    #[serde(default)]
    pub jvm: Vec<ArgumentValue>,
}

/// The parts of a version document Mnelys consumes.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VersionDocument {
    pub id: String,
    #[serde(default)]
    pub inherits_from: Option<String>,
    #[serde(default)]
    pub main_class: Option<String>,
    #[serde(default)]
    pub assets: Option<String>,
    #[serde(default)]
    pub asset_index: Option<AssetIndex>,
    #[serde(default)]
    pub downloads: BTreeMap<String, DownloadInfo>,
    #[serde(default)]
    pub java_version: Option<JavaVersion>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub logging: Option<LoggingConfig>,
    #[serde(default)]
    pub arguments: Option<Arguments>,
    /// Pre-1.13 versions carry a single argument string.
    #[serde(default)]
    pub minecraft_arguments: Option<String>,
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
    #[serde(default, rename = "releaseTime")]
    pub release_time: Option<String>,
}

impl VersionDocument {
    /// The client jar, when the document names one.
    #[must_use]
    pub fn client_download(&self) -> Option<&DownloadInfo> {
        self.downloads.get("client")
    }

    /// The Java major the version expects. Old versions default to 8.
    #[must_use]
    pub fn required_java_major(&self) -> u32 {
        self.java_version
            .as_ref()
            .map_or(8, |java| java.major_version)
    }

    /// The asset index identifier the game receives on the command line.
    #[must_use]
    pub fn asset_index_id(&self) -> String {
        self.asset_index.as_ref().map_or_else(
            || self.assets.clone().unwrap_or_else(|| "legacy".to_owned()),
            |index| index.id.clone(),
        )
    }

    /// Merges a loader profile onto this base document.
    ///
    /// Loader profiles inherit from the Vanilla version and only add libraries, a main
    /// class and arguments. Later entries win where the profile is authoritative, while
    /// the base keeps the client jar, asset index and Java requirement.
    pub fn merge_profile(&self, profile: &Self) -> Self {
        let mut merged = self.clone();

        if profile.main_class.is_some() {
            merged.main_class.clone_from(&profile.main_class);
        }
        if profile.assets.is_some() {
            merged.assets.clone_from(&profile.assets);
        }
        if profile.asset_index.is_some() {
            merged.asset_index.clone_from(&profile.asset_index);
        }
        if profile.java_version.is_some() {
            merged.java_version.clone_from(&profile.java_version);
        }
        if profile.logging.is_some() {
            merged.logging.clone_from(&profile.logging);
        }
        for (key, value) in &profile.downloads {
            merged.downloads.insert(key.clone(), value.clone());
        }
        merged.libraries.extend(profile.libraries.iter().cloned());

        match (&mut merged.arguments, &profile.arguments) {
            (Some(base), Some(extra)) => {
                base.game.extend(extra.game.iter().cloned());
                base.jvm.extend(extra.jvm.iter().cloned());
            }
            (None, Some(extra)) => {
                // A legacy base carries one argument string; promote it so the loader's
                // modern argument list does not hide the game arguments.
                let mut promoted = Arguments::default();
                if let Some(legacy) = merged.minecraft_arguments.take() {
                    promoted.game = legacy
                        .split_whitespace()
                        .map(|part| ArgumentValue::Plain(part.to_owned()))
                        .collect();
                }
                promoted.game.extend(extra.game.iter().cloned());
                promoted.jvm.extend(extra.jvm.iter().cloned());
                merged.arguments = Some(promoted);
            }
            _ => {}
        }
        if profile.minecraft_arguments.is_some() {
            merged
                .minecraft_arguments
                .clone_from(&profile.minecraft_arguments);
        }

        merged
    }

    /// Flattens the JVM and game arguments that apply on this platform.
    #[must_use]
    pub fn filtered_arguments(&self, platform: Platform) -> (Vec<String>, Vec<String>) {
        match &self.arguments {
            Some(arguments) => (
                filter_arguments(&arguments.jvm, platform),
                filter_arguments(&arguments.game, platform),
            ),
            None => (
                Vec::new(),
                self.minecraft_arguments
                    .as_deref()
                    .map(|legacy| legacy.split_whitespace().map(str::to_owned).collect())
                    .unwrap_or_default(),
            ),
        }
    }
}

fn filter_arguments(values: &[ArgumentValue], platform: Platform) -> Vec<String> {
    let mut selected = Vec::new();
    for value in values {
        match value {
            ArgumentValue::Plain(text) => selected.push(text.clone()),
            ArgumentValue::Conditional { rules, value } => {
                if rules_allow(rules, platform) {
                    value.for_each(|text| selected.push(text.to_owned()));
                }
            }
        }
    }
    selected
}

/// The platform rules are evaluated against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOs,
    Linux,
}

impl Platform {
    /// The platform this binary was compiled for.
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Linux
        }
    }

    /// The name Mojang metadata uses.
    #[must_use]
    pub const fn metadata_name(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::MacOs => "osx",
            Self::Linux => "linux",
        }
    }

    /// The architecture token Mojang metadata uses.
    #[must_use]
    pub const fn metadata_arch(self) -> &'static str {
        if cfg!(target_arch = "aarch64") {
            "arm64"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else {
            "x86_64"
        }
    }

    /// The classifier suffix used by legacy native libraries.
    #[must_use]
    pub const fn native_classifier(self) -> &'static str {
        match self {
            Self::Windows => "natives-windows",
            Self::MacOs => "natives-macos",
            Self::Linux => "natives-linux",
        }
    }
}

/// Evaluates a rule list the way the game does: start disallowed, then let each
/// matching rule decide, so the last matching rule wins. An empty list allows the entry.
#[must_use]
pub fn rules_allow(rules: &[Rule], platform: Platform) -> bool {
    if rules.is_empty() {
        return true;
    }

    let mut allowed = false;
    for rule in rules {
        if rule_matches(rule, platform) {
            allowed = rule.action == RuleAction::Allow;
        }
    }
    allowed
}

fn rule_matches(rule: &Rule, platform: Platform) -> bool {
    if let Some(os) = &rule.os {
        if let Some(name) = &os.name {
            if name != platform.metadata_name() {
                return false;
            }
        }
        if let Some(arch) = &os.arch {
            if arch != platform.metadata_arch() {
                return false;
            }
        }
        if os.version.is_some() {
            // Matching a regex against the OS version is deliberately unsupported; the
            // only rules Mojang ships target old macOS releases.
            return false;
        }
    }

    if let Some(features) = &rule.features {
        // Demo mode, custom resolution and quick play are never active here.
        if features.values().any(|enabled| *enabled) {
            return false;
        }
    }

    true
}

/// A parsed Maven coordinate (`group:artifact:version[:classifier][@extension]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MavenCoordinate {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub classifier: Option<String>,
    pub extension: String,
}

impl MavenCoordinate {
    /// Parses a coordinate as loader profiles write them.
    pub fn parse(value: &str) -> Result<Self, CoreError> {
        let (body, extension) = match value.rsplit_once('@') {
            Some((body, extension)) => (body, extension),
            None => (value, "jar"),
        };
        let parts: Vec<&str> = body.split(':').collect();
        let (group, artifact, version) = match parts.as_slice() {
            [group, artifact, version] => (*group, *artifact, *version),
            [group, artifact, version, _] => (*group, *artifact, *version),
            _ => {
                return Err(CoreError::invalid_request(format!(
                    "`{value}` is not a Maven coordinate"
                )))
            }
        };
        if group.is_empty() || artifact.is_empty() || version.is_empty() {
            return Err(CoreError::invalid_request(format!(
                "`{value}` has an empty Maven segment"
            )));
        }
        let classifier = parts.get(3).map(|value| (*value).to_owned());
        Ok(Self {
            group: group.to_owned(),
            artifact: artifact.to_owned(),
            version: version.to_owned(),
            classifier,
            extension: extension.to_owned(),
        })
    }

    /// The path below a Maven repository root.
    #[must_use]
    pub fn repository_path(&self) -> String {
        let file = match &self.classifier {
            Some(classifier) => {
                format!(
                    "{}-{}-{}.{}",
                    self.artifact, self.version, classifier, self.extension
                )
            }
            None => format!("{}-{}.{}", self.artifact, self.version, self.extension),
        };
        format!(
            "{}/{}/{}/{}",
            self.group.replace('.', "/"),
            self.artifact,
            self.version,
            file
        )
    }

    /// The file name a cached copy uses.
    #[must_use]
    pub fn file_name(&self) -> String {
        self.repository_path()
            .rsplit('/')
            .next()
            .unwrap_or("library.jar")
            .to_owned()
    }
}

/// The asset index document referenced by a version.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AssetIndexDocument {
    #[serde(default)]
    pub objects: BTreeMap<String, AssetObject>,
}

/// One asset in the index.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os_rule(name: &str, action: RuleAction) -> Rule {
        Rule {
            action,
            os: Some(OsRule {
                name: Some(name.to_owned()),
                arch: None,
                version: None,
            }),
            features: None,
        }
    }

    #[test]
    fn an_empty_rule_list_allows_everything() {
        assert!(rules_allow(&[], Platform::Linux));
    }

    #[test]
    fn a_rule_list_that_targets_other_platforms_excludes_this_one() {
        let rules = vec![
            os_rule("osx", RuleAction::Allow),
            os_rule("windows", RuleAction::Disallow),
        ];
        assert!(!rules_allow(&rules, Platform::Linux));
        assert!(!rules_allow(&rules, Platform::Windows));
        assert!(rules_allow(&rules, Platform::MacOs));
    }

    #[test]
    fn the_last_matching_rule_wins() {
        let rules = vec![
            os_rule("linux", RuleAction::Allow),
            os_rule("linux", RuleAction::Disallow),
        ];
        assert!(!rules_allow(&rules, Platform::Linux));
    }

    #[test]
    fn legacy_arguments_are_split_on_whitespace() {
        let version = VersionDocument {
            id: "1.6.4".to_owned(),
            inherits_from: None,
            main_class: None,
            assets: None,
            asset_index: None,
            downloads: BTreeMap::new(),
            java_version: None,
            libraries: Vec::new(),
            logging: None,
            arguments: None,
            minecraft_arguments: Some("--username ${auth_player_name}".to_owned()),
            kind: None,
            release_time: None,
        };
        let (jvm, game) = version.filtered_arguments(Platform::Linux);
        assert!(jvm.is_empty());
        assert_eq!(game, ["--username", "${auth_player_name}"]);
    }

    #[test]
    fn maven_paths_follow_the_repository_layout() {
        let coordinate =
            MavenCoordinate::parse("net.fabricmc:fabric-loader:0.16.9").expect("valid");
        assert_eq!(
            coordinate.repository_path(),
            "net/fabricmc/fabric-loader/0.16.9/fabric-loader-0.16.9.jar"
        );

        let native = MavenCoordinate::parse("org.lwjgl:lwjgl:3.3.3:natives-linux").expect("valid");
        assert!(native
            .repository_path()
            .ends_with("lwjgl-3.3.3-natives-linux.jar"));
    }

    #[test]
    fn a_bad_coordinate_is_rejected() {
        assert!(MavenCoordinate::parse("only:two").is_err());
        assert!(MavenCoordinate::parse("::").is_err());
    }
}
