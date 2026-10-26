//! Instances: the isolated game directories the launcher owns.
//!
//! An instance is identified by a stable slug, never by its display name, so renaming
//! never moves files. Loader selection is part of the spec because every install and
//! launch plan depends on it.

use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

use crate::CoreError;

/// The mod loader an instance runs on.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum LoaderKind {
    #[default]
    Vanilla,
    Fabric,
    Quilt,
    #[serde(rename = "legacy-fabric")]
    LegacyFabric,
    Forge,
    #[serde(rename = "neoforge")]
    NeoForge,
}

impl LoaderKind {
    /// The wire and storage name. Never change a string without a schema migration.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::LegacyFabric => "legacy-fabric",
            Self::Forge => "forge",
            Self::NeoForge => "neoforge",
        }
    }

    /// Parses a stored loader name.
    pub fn parse(value: &str) -> Result<Self, CoreError> {
        match value {
            "vanilla" => Ok(Self::Vanilla),
            "fabric" => Ok(Self::Fabric),
            "quilt" => Ok(Self::Quilt),
            "legacy-fabric" => Ok(Self::LegacyFabric),
            "forge" => Ok(Self::Forge),
            "neoforge" => Ok(Self::NeoForge),
            other => Err(CoreError::invalid_request(format!(
                "`{other}` is not a supported loader"
            ))),
        }
    }

    /// Whether this loader needs an explicit loader version.
    #[must_use]
    pub const fn needs_version(self) -> bool {
        !matches!(self, Self::Vanilla)
    }

    /// Whether the loader ships a merged profile instead of running its own installer.
    #[must_use]
    pub const fn uses_metadata_profile(self) -> bool {
        matches!(self, Self::Fabric | Self::Quilt | Self::LegacyFabric)
    }

    /// Whether the loader installs by executing an installer jar.
    #[must_use]
    pub const fn uses_installer(self) -> bool {
        matches!(self, Self::Forge | Self::NeoForge)
    }
}

/// What the user asked for when creating an instance.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSpec {
    pub id: String,
    pub name: String,
    pub game_version: String,
    #[serde(default)]
    pub loader: LoaderKind,
    #[serde(default)]
    pub loader_version: Option<String>,
}

impl InstanceSpec {
    /// Validates the parts of the spec that must be unambiguous before any I/O.
    pub fn validate(&self) -> Result<(), CoreError> {
        validate_slug(&self.id)?;
        let name = self.name.trim();
        if name.is_empty() || name.chars().count() > 64 {
            return Err(CoreError::invalid_request(
                "instance name must be between 1 and 64 characters",
            ));
        }
        if name.chars().any(char::is_control) {
            return Err(CoreError::invalid_request(
                "instance name must not contain control characters",
            ));
        }
        validate_version_token(&self.game_version, "game version")?;
        if self.loader.needs_version() {
            match self.loader_version.as_deref() {
                Some(version) => validate_version_token(version, "loader version")?,
                None => {
                    return Err(CoreError::invalid_request(
                        "a loader version is required for this loader",
                    ))
                }
            }
        }
        Ok(())
    }

    /// The identifier of the executable version this spec resolves to.
    ///
    /// These match the ids the loader providers publish, so a merged profile keeps the
    /// same name users see in other launchers.
    #[must_use]
    pub fn version_id(&self) -> String {
        match self.loader {
            LoaderKind::Vanilla => self.game_version.clone(),
            LoaderKind::Fabric => format!(
                "fabric-loader-{}-{}",
                self.loader_version.as_deref().unwrap_or("unknown"),
                self.game_version
            ),
            LoaderKind::Quilt => format!(
                "quilt-loader-{}-{}",
                self.loader_version.as_deref().unwrap_or("unknown"),
                self.game_version
            ),
            LoaderKind::LegacyFabric => format!(
                "legacy-fabric-loader-{}-{}",
                self.loader_version.as_deref().unwrap_or("unknown"),
                self.game_version
            ),
            LoaderKind::Forge => format!(
                "forge-{}-{}",
                self.game_version,
                self.loader_version.as_deref().unwrap_or("unknown")
            ),
            LoaderKind::NeoForge => format!(
                "neoforge-{}",
                self.loader_version.as_deref().unwrap_or("unknown")
            ),
        }
    }
}

fn validate_slug(value: &str) -> Result<(), CoreError> {
    let valid = !value.is_empty()
        && value.len() <= 48
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        && value
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_lowercase() || first.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err(CoreError::invalid_request(
            "instance id must be a lowercase slug of letters, digits and dashes",
        ))
    }
}

fn validate_version_token(value: &str, label: &str) -> Result<(), CoreError> {
    let valid = !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._+-".contains(character));
    if valid {
        Ok(())
    } else {
        Err(CoreError::invalid_request(format!(
            "{label} contains characters Mnelys does not accept"
        )))
    }
}

/// The persisted record. `schema_version` is checked before the record is used.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstanceRecord {
    pub schema_version: u32,
    pub spec: InstanceSpec,
    pub created_at_ms: u64,
    #[serde(default)]
    pub last_launched_ms: Option<u64>,
    #[serde(default)]
    pub java_path: Option<String>,
    /// The merged executable version id (base id for Vanilla, profile id for loaders).
    pub version_id: String,
}

impl InstanceRecord {
    pub const SCHEMA_VERSION: u32 = 1;

    /// Creates a record for a spec at a point in time.
    #[must_use]
    pub fn new(spec: InstanceSpec, created_at_ms: u64, version_id: String) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            spec,
            created_at_ms,
            last_launched_ms: None,
            java_path: None,
            version_id,
        }
    }

    /// Rejects a record written by a newer Mnelys.
    pub fn check_schema(&self) -> Result<(), CoreError> {
        if self.schema_version > Self::SCHEMA_VERSION {
            return Err(CoreError::internal(format!(
                "instance `{}` uses schema {} but this build understands {}",
                self.spec.id,
                self.schema_version,
                Self::SCHEMA_VERSION
            )));
        }
        Ok(())
    }
}

/// The offline UUID Minecraft uses for a username.
///
/// This is `UUID.nameUUIDFromBytes(("OfflinePlayer:" + name).getBytes(UTF_8))`, which is
/// an MD5 name UUID with version 3 and the RFC 4122 variant. Servers derive the same
/// value, so offline skins and player data match.
#[must_use]
pub fn offline_uuid(username: &str) -> String {
    let mut digest = Md5::digest(format!("OfflinePlayer:{username}").as_bytes());
    digest[6] = (digest[6] & 0x0f) | 0x30;
    digest[8] = (digest[8] & 0x3f) | 0x80;
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// Validates a Minecraft username for an offline account.
pub fn validate_offline_username(username: &str) -> Result<(), CoreError> {
    let valid = (3..=16).contains(&username.chars().count())
        && username
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_');
    if valid {
        Ok(())
    } else {
        Err(CoreError::invalid_request(
            "username must be 3-16 characters of letters, digits or underscore",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_uuids_match_the_java_algorithm() {
        // Known values produced by java.util.UUID.nameUUIDFromBytes for the same names.
        assert_eq!(
            offline_uuid("Notch"),
            "b50ad385-829d-3141-a216-7e7d7539ba7f"
        );
        assert_eq!(offline_uuid("jeb_"), "a762f560-4fce-3236-812a-b80efff0b62b");
    }

    #[test]
    fn a_valid_spec_passes_validation() {
        let spec = InstanceSpec {
            id: "1-21-4-fabric".to_owned(),
            name: "Fabric 1.21.4".to_owned(),
            game_version: "1.21.4".to_owned(),
            loader: LoaderKind::Fabric,
            loader_version: Some("0.16.9".to_owned()),
        };
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn a_loader_without_a_version_is_rejected() {
        let spec = InstanceSpec {
            id: "fabric".to_owned(),
            name: "Fabric".to_owned(),
            game_version: "1.21.4".to_owned(),
            loader: LoaderKind::Fabric,
            loader_version: None,
        };
        assert!(spec.validate().is_err());
    }

    #[test]
    fn unsafe_slugs_and_names_are_rejected() {
        let base = InstanceSpec {
            id: "ok".to_owned(),
            name: "Ok".to_owned(),
            game_version: "1.21.4".to_owned(),
            loader: LoaderKind::Vanilla,
            loader_version: None,
        };
        for id in ["../escape", "Upper", "", "-leading"] {
            let mut spec = base.clone();
            spec.id = id.to_owned();
            assert!(spec.validate().is_err(), "{id} must be rejected");
        }
        let mut spec = base;
        spec.name = "bad\nname".to_owned();
        assert!(spec.validate().is_err());
    }

    #[test]
    fn an_offline_username_is_checked() {
        assert!(validate_offline_username("Player_1").is_ok());
        assert!(validate_offline_username("ab").is_err());
        assert!(validate_offline_username("has space").is_err());
    }
}
