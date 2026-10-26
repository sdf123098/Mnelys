//! Core settings that must survive a restart.
//!
//! The frontend persists its own presentation preferences (accent colour, language).
//! What lives here changes how the core behaves: which mirror it downloads from and
//! the heap size new launches start with.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use mnelys_domain::CoreResult;

use crate::mirrors::MirrorPreference;

/// The persisted core settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub schema_version: u32,
    #[serde(default)]
    pub mirror: MirrorPreference,
    /// The default heap size offered when launching. Zero means "core default".
    #[serde(default)]
    pub memory_mb: u32,
}

impl AppSettings {
    pub const SCHEMA_VERSION: u32 = 1;

    /// Rejects a file written by a newer Mnelys.
    pub fn check_schema(&self) -> CoreResult<()> {
        if self.schema_version > Self::SCHEMA_VERSION {
            return Err(mnelys_domain::CoreError::internal(format!(
                "the settings file uses schema {} but this build understands {}",
                self.schema_version,
                Self::SCHEMA_VERSION
            )));
        }
        Ok(())
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            mirror: MirrorPreference::Direct,
            memory_mb: 0,
        }
    }
}

/// Persistence for [`AppSettings`].
#[async_trait]
pub trait SettingsStore: Send + Sync {
    async fn load(&self) -> CoreResult<AppSettings>;
    async fn save(&self, settings: &AppSettings) -> CoreResult<()>;
}
