//! Every filesystem location Mnelys owns.
//!
//! One type computes the layout so the core, the adapters and the tests cannot drift
//! on where a version, a library or an instance lives.

use std::path::{Path, PathBuf};

/// The directory layout below one root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamePaths {
    root: PathBuf,
}

impl GamePaths {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn instances_dir(&self) -> PathBuf {
        self.root.join("instances")
    }

    #[must_use]
    pub fn instance_dir(&self, id: &str) -> PathBuf {
        self.instances_dir().join(id)
    }

    #[must_use]
    pub fn instance_record(&self, id: &str) -> PathBuf {
        self.instance_dir(id).join("instance.json")
    }

    /// The game directory handed to Minecraft as `--gameDir`.
    #[must_use]
    pub fn instance_game_dir(&self, id: &str) -> PathBuf {
        self.instance_dir(id).join("game")
    }

    #[must_use]
    pub fn versions_dir(&self) -> PathBuf {
        self.root.join("versions")
    }

    #[must_use]
    pub fn version_dir(&self, id: &str) -> PathBuf {
        self.versions_dir().join(id)
    }

    #[must_use]
    pub fn version_json(&self, id: &str) -> PathBuf {
        self.version_dir(id).join(format!("{id}.json"))
    }

    #[must_use]
    pub fn version_jar(&self, id: &str) -> PathBuf {
        self.version_dir(id).join(format!("{id}.jar"))
    }

    #[must_use]
    pub fn version_natives(&self, id: &str) -> PathBuf {
        self.version_dir(id).join("natives")
    }

    #[must_use]
    pub fn installed_marker(&self, id: &str) -> PathBuf {
        self.version_dir(id).join(".mnelys-installed")
    }

    #[must_use]
    pub fn libraries_dir(&self) -> PathBuf {
        self.root.join("libraries")
    }

    #[must_use]
    pub fn assets_dir(&self) -> PathBuf {
        self.root.join("assets")
    }

    #[must_use]
    pub fn asset_objects_dir(&self) -> PathBuf {
        self.assets_dir().join("objects")
    }

    #[must_use]
    pub fn asset_indexes_dir(&self) -> PathBuf {
        self.assets_dir().join("indexes")
    }

    #[must_use]
    pub fn asset_index_file(&self, id: &str) -> PathBuf {
        self.asset_indexes_dir().join(format!("{id}.json"))
    }

    #[must_use]
    pub fn virtual_assets_dir(&self, id: &str) -> PathBuf {
        self.assets_dir().join("virtual").join(id)
    }

    #[must_use]
    pub fn log_configs_dir(&self) -> PathBuf {
        self.assets_dir().join("log_configs")
    }

    #[must_use]
    pub fn meta_dir(&self) -> PathBuf {
        self.root.join("meta")
    }

    #[must_use]
    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }

    #[must_use]
    pub fn java_dir(&self) -> PathBuf {
        self.root.join("java")
    }

    #[must_use]
    pub fn java_runtime_dir(&self, major: u32) -> PathBuf {
        self.java_dir().join(major.to_string())
    }

    #[must_use]
    pub fn accounts_file(&self) -> PathBuf {
        self.root.join("accounts.json")
    }

    #[must_use]
    pub fn exports_dir(&self) -> PathBuf {
        self.root.join("exports")
    }

    #[must_use]
    pub fn settings_file(&self) -> PathBuf {
        self.root.join("settings.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_files_live_below_the_instance_directory() {
        let paths = GamePaths::new("/data");
        assert_eq!(
            paths.instance_record("main"),
            PathBuf::from("/data/instances/main/instance.json")
        );
        assert_eq!(
            paths.instance_game_dir("main"),
            PathBuf::from("/data/instances/main/game")
        );
    }

    #[test]
    fn a_version_json_is_named_after_its_version() {
        let paths = GamePaths::new("/data");
        assert_eq!(
            paths.version_json("1.21.4"),
            PathBuf::from("/data/versions/1.21.4/1.21.4.json")
        );
    }
}
