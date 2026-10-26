//! Download mirrors.
//!
//! Mojang's CDN is the source of truth. BMCLAPI mirrors the same paths for users whose
//! route to Mojang is slow; the policy rewrites only well-known Mojang hosts, so a
//! provider URL is never sent somewhere unexpected.

use std::sync::{Arc, RwLock};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::ports::{fetch_bytes, HttpClient};

/// What the user asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MirrorPreference {
    /// Always use Mojang. The default.
    #[default]
    Direct,
    /// Always use BMCLAPI.
    Bmclapi,
    /// Probe both once per process and use the faster one.
    Auto,
}

/// Which source is actually used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MirrorKind {
    Direct,
    Bmclapi,
}

impl MirrorKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Direct => "mojang",
            Self::Bmclapi => "bmclapi",
        }
    }

    const fn manifest_url(self) -> &'static str {
        match self {
            Self::Direct => "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
            Self::Bmclapi => "https://bmclapi2.bangbang93.com/mc/game/version_manifest_v2.json",
        }
    }
}

/// One probe result, as the settings page displays it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorProbe {
    pub mirror: MirrorKind,
    pub reachable: bool,
    #[serde(default)]
    pub latency_ms: Option<u64>,
}

/// Chooses and applies a mirror for Mojang-hosted URLs.
#[derive(Clone)]
pub struct MirrorPolicy {
    preference: Arc<RwLock<MirrorPreference>>,
    chosen: Arc<RwLock<Option<MirrorKind>>>,
}

impl MirrorPolicy {
    #[must_use]
    pub fn new(preference: MirrorPreference) -> Self {
        let chosen = match preference {
            MirrorPreference::Direct => Some(MirrorKind::Direct),
            MirrorPreference::Bmclapi => Some(MirrorKind::Bmclapi),
            MirrorPreference::Auto => None,
        };
        Self {
            preference: Arc::new(RwLock::new(preference)),
            chosen: Arc::new(RwLock::new(chosen)),
        }
    }

    #[must_use]
    pub fn preference(&self) -> MirrorPreference {
        *self
            .preference
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Changes the preference. Automatic probing runs again on the next use.
    pub fn set_preference(&self, preference: MirrorPreference) {
        *self
            .preference
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = preference;
        let chosen = match preference {
            MirrorPreference::Direct => Some(MirrorKind::Direct),
            MirrorPreference::Bmclapi => Some(MirrorKind::Bmclapi),
            MirrorPreference::Auto => None,
        };
        *self
            .chosen
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = chosen;
    }

    #[must_use]
    pub fn active(&self) -> MirrorKind {
        self.chosen
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .unwrap_or(MirrorKind::Direct)
    }

    /// Resolves an automatic choice once per process. A no-op for explicit choices.
    pub async fn ensure(&self, client: &dyn HttpClient) {
        if self.preference() != MirrorPreference::Auto || self.active_known() {
            return;
        }
        let direct = probe(client, MirrorKind::Direct).await;
        let bmclapi = probe(client, MirrorKind::Bmclapi).await;
        let chosen = match (direct.latency_ms, bmclapi.latency_ms) {
            (Some(direct), Some(bmclapi)) if bmclapi < direct => MirrorKind::Bmclapi,
            (Some(_), _) => MirrorKind::Direct,
            (None, Some(_)) => MirrorKind::Bmclapi,
            (None, None) => MirrorKind::Direct,
        };
        *self
            .chosen
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(chosen);
    }

    fn active_known(&self) -> bool {
        self.chosen
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    /// Rewrites a Mojang URL for the active mirror. Other hosts are untouched.
    #[must_use]
    pub fn rewrite(&self, url: &str) -> String {
        if self.active() == MirrorKind::Direct {
            return url.to_owned();
        }
        for (source, target) in [
            (
                "https://piston-meta.mojang.com",
                "https://bmclapi2.bangbang93.com",
            ),
            (
                "https://launchermeta.mojang.com",
                "https://bmclapi2.bangbang93.com",
            ),
            (
                "https://launcher.mojang.com",
                "https://bmclapi2.bangbang93.com",
            ),
            (
                "https://resources.download.minecraft.net",
                "https://bmclapi2.bangbang93.com/assets",
            ),
            (
                "https://libraries.minecraft.net",
                "https://bmclapi2.bangbang93.com/maven",
            ),
            (
                "https://maven.minecraftforge.net",
                "https://bmclapi2.bangbang93.com/maven",
            ),
        ] {
            if let Some(rest) = url.strip_prefix(source) {
                return format!("{target}{rest}");
            }
        }
        url.to_owned()
    }
}

/// Times both manifest endpoints.
pub async fn probe_all(client: &dyn HttpClient) -> Vec<MirrorProbe> {
    vec![
        probe(client, MirrorKind::Direct).await,
        probe(client, MirrorKind::Bmclapi).await,
    ]
}

async fn probe(client: &dyn HttpClient, mirror: MirrorKind) -> MirrorProbe {
    let started = Instant::now();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        fetch_bytes(client, mirror.manifest_url()),
    )
    .await;
    let reachable = matches!(result, Ok(Ok(_)));
    MirrorProbe {
        mirror,
        reachable,
        latency_ms: reachable.then(|| started.elapsed().as_millis() as u64),
    }
}

/// Fails with a stable error when a mirror URL cannot be constructed.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_policy_is_direct_and_rewrites_nothing() {
        let policy = MirrorPolicy::new(MirrorPreference::Direct);
        let url = "https://libraries.minecraft.net/org/a.jar";
        assert_eq!(policy.rewrite(url), url);
    }

    #[test]
    fn bmclapi_rewrites_known_hosts_only() {
        let policy = MirrorPolicy::new(MirrorPreference::Bmclapi);
        assert_eq!(
            policy.rewrite("https://piston-meta.mojang.com/mc/x.json"),
            "https://bmclapi2.bangbang93.com/mc/x.json"
        );
        assert_eq!(
            policy.rewrite("https://resources.download.minecraft.net/ab/abcdef"),
            "https://bmclapi2.bangbang93.com/assets/ab/abcdef"
        );
        assert_eq!(
            policy.rewrite("https://maven.fabricmc.net/a.jar"),
            "https://maven.fabricmc.net/a.jar"
        );
    }

    #[test]
    fn the_manifest_path_is_identical_on_both_hosts() {
        let suffix = "version_manifest_v2.json";
        assert!(MirrorKind::Direct.manifest_url().ends_with(suffix));
        assert!(MirrorKind::Bmclapi.manifest_url().ends_with(suffix));
    }
}
