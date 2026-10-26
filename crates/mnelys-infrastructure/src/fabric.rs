//! Fabric, Quilt and Legacy Fabric loader metadata.
//!
//! The three projects publish the same profile format, so one provider serves all of
//! them and only the base URL changes.

use std::sync::Arc;

use async_trait::async_trait;
use mnelys_application::ports::{fetch_json, HttpClient, LoaderProvider};
use mnelys_domain::{CoreError, CoreResult, LoaderKind, LoaderVersionInfo, VersionDocument};
use serde::Deserialize;

const FABRIC: &str = "https://meta.fabricmc.net/v2";
const QUILT: &str = "https://meta.quiltmc.org/v3";
const LEGACY_FABRIC: &str = "https://meta.legacyfabric.net/v2";

/// Serves loader versions and profiles for the Fabric family.
pub struct FabricLikeProvider {
    client: Arc<dyn HttpClient>,
}

impl FabricLikeProvider {
    #[must_use]
    pub fn new(client: Arc<dyn HttpClient>) -> Self {
        Self { client }
    }
}

fn base_url(loader: LoaderKind) -> CoreResult<&'static str> {
    match loader {
        LoaderKind::Fabric => Ok(FABRIC),
        LoaderKind::Quilt => Ok(QUILT),
        LoaderKind::LegacyFabric => Ok(LEGACY_FABRIC),
        other => Err(CoreError::not_implemented(format!(
            "{} does not publish Fabric-style metadata",
            other.as_str()
        ))),
    }
}

fn check_token(value: &str, label: &str) -> CoreResult<()> {
    let valid = !value.is_empty()
        && value.len() <= 64
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_' | '+')
        });
    if valid {
        Ok(())
    } else {
        Err(CoreError::invalid_request(format!(
            "`{value}` is not a valid {label}"
        )))
    }
}

#[derive(Debug, Deserialize)]
struct LoaderEntry {
    #[serde(default)]
    loader: Option<LoaderSummary>,
    /// Quilt's per-game listing puts the version at the top level.
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    stable: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct LoaderSummary {
    version: String,
    #[serde(default)]
    stable: bool,
}

impl LoaderEntry {
    fn into_info(self, game_version: &str, loader: LoaderKind) -> Option<LoaderVersionInfo> {
        let (version, stable) = match self.loader {
            Some(summary) => (summary.version, summary.stable),
            None => (self.version?, self.stable.unwrap_or(true)),
        };
        Some(LoaderVersionInfo {
            version,
            game_version: game_version.to_owned(),
            loader,
            stable,
        })
    }
}

fn version_key(version: &str) -> Vec<u64> {
    version
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect()
}

#[async_trait]
impl LoaderProvider for FabricLikeProvider {
    async fn versions(
        &self,
        loader: LoaderKind,
        game_version: &str,
    ) -> CoreResult<Vec<LoaderVersionInfo>> {
        check_token(game_version, "game version")?;
        let base = base_url(loader)?;
        let url = format!("{base}/versions/loader/{game_version}");
        let entries: Vec<LoaderEntry> = fetch_json(self.client.as_ref(), &url).await?;
        let mut versions: Vec<LoaderVersionInfo> = entries
            .into_iter()
            .filter_map(|entry| entry.into_info(game_version, loader))
            .collect();
        versions.sort_by_key(|entry| std::cmp::Reverse(version_key(&entry.version)));
        versions.dedup_by(|left, right| left.version == right.version);
        Ok(versions)
    }

    async fn profile(
        &self,
        loader: LoaderKind,
        game_version: &str,
        loader_version: &str,
    ) -> CoreResult<VersionDocument> {
        check_token(game_version, "game version")?;
        check_token(loader_version, "loader version")?;
        let base = base_url(loader)?;
        let url = format!("{base}/versions/loader/{game_version}/{loader_version}/profile/json");
        let mut document: VersionDocument = fetch_json(self.client.as_ref(), &url).await?;
        match document.inherits_from.as_deref() {
            Some(parent) if parent != game_version => {
                return Err(CoreError::network(format!(
                    "the {} profile for {loader_version} inherits from {parent}, not {game_version}",
                    loader.as_str()
                )));
            }
            None => document.inherits_from = Some(game_version.to_owned()),
            Some(_) => {}
        }
        if document
            .main_class
            .as_deref()
            .unwrap_or_default()
            .is_empty()
        {
            return Err(CoreError::network(format!(
                "the {} profile for {loader_version} has no main class",
                loader.as_str()
            )));
        }
        Ok(document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_fabric_style_loader_has_a_base() {
        assert!(base_url(LoaderKind::Fabric).is_ok());
        assert!(base_url(LoaderKind::Quilt).is_ok());
        assert!(base_url(LoaderKind::LegacyFabric).is_ok());
        assert!(base_url(LoaderKind::Forge).is_err());
        assert!(base_url(LoaderKind::Vanilla).is_err());
    }

    #[test]
    fn a_fabric_entry_is_read_from_the_nested_shape() {
        let entry: LoaderEntry = serde_json::from_str(
            r#"{"loader":{"version":"0.16.9","stable":true},"intermediary":{"version":"1.21.4"}}"#,
        )
        .expect("parses");
        let info = entry
            .into_info("1.21.4", LoaderKind::Fabric)
            .expect("has a version");
        assert_eq!(info.version, "0.16.9");
        assert!(info.stable);
    }

    #[test]
    fn a_flat_entry_is_read_too() {
        let entry: LoaderEntry =
            serde_json::from_str(r#"{"version":"0.27.0","build":33}"#).expect("parses");
        let info = entry
            .into_info("1.21.4", LoaderKind::Quilt)
            .expect("has a version");
        assert_eq!(info.version, "0.27.0");
    }

    #[test]
    fn loader_versions_sort_numerically() {
        let mut keys: Vec<Vec<u64>> = vec![version_key("0.9.0"), version_key("0.16.9")];
        keys.sort();
        assert_eq!(keys[0], vec![0, 9, 0]);
        assert_eq!(keys[1], vec![0, 16, 9]);
    }

    #[test]
    fn a_traversal_token_is_rejected() {
        assert!(check_token("../secrets", "game version").is_err());
        assert!(check_token("1.21.4", "game version").is_ok());
    }
}
