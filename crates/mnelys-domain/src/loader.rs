//! Loader versions a user can choose from.

use serde::{Deserialize, Serialize};

use crate::LoaderKind;

/// One installable loader build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderVersionInfo {
    pub version: String,
    pub game_version: String,
    pub loader: LoaderKind,
    /// Whether the provider marks this build as stable.
    pub stable: bool,
}

impl LoaderVersionInfo {
    #[must_use]
    pub fn new(
        loader: LoaderKind,
        game_version: impl Into<String>,
        version: impl Into<String>,
        stable: bool,
    ) -> Self {
        Self {
            version: version.into(),
            game_version: game_version.into(),
            loader,
            stable,
        }
    }
}
