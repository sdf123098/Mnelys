//! Content platform vocabulary: projects, versions, files and dependencies.

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// A content source Mnelys can talk to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentProviderId {
    Modrinth,
    #[serde(rename = "curseforge")]
    CurseForge,
}

impl ContentProviderId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Modrinth => "modrinth",
            Self::CurseForge => "curseforge",
        }
    }

    pub fn parse(value: &str) -> Result<Self, CoreError> {
        match value {
            "modrinth" => Ok(Self::Modrinth),
            "curseforge" => Ok(Self::CurseForge),
            other => Err(CoreError::invalid_request(format!(
                "`{other}` is not a supported content provider"
            ))),
        }
    }
}

/// What kind of content a project carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentKind {
    Mod,
    ResourcePack,
    Shader,
    World,
    Datapack,
}

impl ContentKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mod => "mod",
            Self::ResourcePack => "resourcepack",
            Self::Shader => "shader",
            Self::World => "world",
            Self::Datapack => "datapack",
        }
    }

    pub fn parse(value: &str) -> Result<Self, CoreError> {
        match value {
            "mod" => Ok(Self::Mod),
            "resourcepack" | "resource-pack" => Ok(Self::ResourcePack),
            "shader" => Ok(Self::Shader),
            "world" => Ok(Self::World),
            "datapack" | "data-pack" => Ok(Self::Datapack),
            other => Err(CoreError::invalid_request(format!(
                "`{other}` is not a supported content kind"
            ))),
        }
    }

    /// The directory inside an instance the content belongs in.
    #[must_use]
    pub const fn directory(self) -> &'static str {
        match self {
            Self::Mod => "mods",
            Self::ResourcePack => "resourcepacks",
            Self::Shader => "shaderpacks",
            Self::World => "saves",
            Self::Datapack => "datapacks",
        }
    }
}

/// A search result row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentProject {
    pub id: String,
    pub provider: ContentProviderId,
    pub kind: ContentKind,
    pub slug: String,
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub author: String,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// One page of search results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchPage {
    pub provider: ContentProviderId,
    pub hits: Vec<ContentProject>,
    pub offset: u32,
    pub limit: u32,
    pub total: u64,
}

/// A downloadable file attached to a version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentFile {
    pub url: String,
    pub filename: String,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    pub primary: bool,
}

impl ContentFile {
    /// Rejects file names that would escape the instance content directory.
    pub fn validate(&self) -> Result<(), CoreError> {
        let name = &self.filename;
        let valid = !name.is_empty()
            && name.len() <= 255
            && !name.contains('/')
            && !name.contains('\\')
            && name != "."
            && name != ".."
            && !name.contains(':')
            && !name.chars().any(char::is_control);
        if valid {
            Ok(())
        } else {
            Err(CoreError::invalid_request(format!(
                "content file name `{name}` is not a plain file name"
            )))
        }
    }
}

/// One published version of a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersion {
    pub id: String,
    pub project_id: String,
    pub provider: ContentProviderId,
    pub name: String,
    pub version_number: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub files: Vec<ContentFile>,
    pub dependencies: Vec<ContentDependency>,
}

impl ContentVersion {
    /// The file an installer should use: the platform-marked primary, else the first.
    #[must_use]
    pub fn primary_file(&self) -> Option<&ContentFile> {
        self.files
            .iter()
            .find(|file| file.primary)
            .or_else(|| self.files.first())
    }
}

/// How strongly one project depends on another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyKind {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

/// A dependency edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDependency {
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub version_id: Option<String>,
    pub kind: DependencyKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_directories_are_the_expected_ones() {
        assert_eq!(ContentKind::Mod.directory(), "mods");
        assert_eq!(ContentKind::Shader.directory(), "shaderpacks");
    }

    #[test]
    fn a_file_name_cannot_escape_its_directory() {
        let mut file = ContentFile {
            url: String::from("https://example.test/a.jar"),
            filename: String::from("../a.jar"),
            sha1: None,
            size: None,
            primary: true,
        };
        assert!(file.validate().is_err());
        file.filename = String::from("a.jar");
        assert!(file.validate().is_ok());
    }

    #[test]
    fn the_primary_file_wins_over_the_first_file() {
        let version = ContentVersion {
            id: String::from("v"),
            project_id: String::from("p"),
            provider: ContentProviderId::Modrinth,
            name: String::from("1.0"),
            version_number: String::from("1.0"),
            game_versions: vec![String::from("1.21.4")],
            loaders: vec![String::from("fabric")],
            files: vec![
                ContentFile {
                    url: String::from("https://example.test/a.jar"),
                    filename: String::from("a.jar"),
                    sha1: None,
                    size: None,
                    primary: false,
                },
                ContentFile {
                    url: String::from("https://example.test/b.jar"),
                    filename: String::from("b.jar"),
                    sha1: None,
                    size: None,
                    primary: true,
                },
            ],
            dependencies: Vec::new(),
        };
        assert_eq!(
            version.primary_file().map(|file| file.filename.as_str()),
            Some("b.jar")
        );
    }
}
