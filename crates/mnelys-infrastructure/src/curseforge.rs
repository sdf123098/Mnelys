//! The CurseForge content provider.
//!
//! CurseForge requires an API key. Mnelys reads it from `MNELYS_CURSEFORGE_API_KEY`;
//! when the variable is absent the provider is not registered at all, so the UI never
//! offers a platform that cannot answer.

use std::sync::Arc;

use async_trait::async_trait;
use mnelys_application::ports::{
    request_json, ContentProvider, ContentSearchRequest, ContentVersionFilter, HttpClient,
    HttpRequest,
};
use mnelys_domain::{
    ContentDependency, ContentFile, ContentKind, ContentProject, ContentProviderId,
    ContentSearchPage, ContentVersion, CoreError, CoreResult, DependencyKind,
};
use serde::Deserialize;

const API: &str = "https://api.curseforge.com/v1";
const MINECRAFT_GAME_ID: u32 = 432;
const PAGE_SIZE: u32 = 50;

/// Reads the API key from the environment.
pub const API_KEY_ENV: &str = "MNELYS_CURSEFORGE_API_KEY";

/// Talks to the CurseForge v1 API.
pub struct CurseForgeProvider {
    client: Arc<dyn HttpClient>,
    api_key: String,
}

impl CurseForgeProvider {
    /// Creates the provider, returning `None` when no key is configured.
    #[must_use]
    pub fn from_env(client: Arc<dyn HttpClient>) -> Option<Self> {
        let key = std::env::var(API_KEY_ENV)
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())?;
        Some(Self {
            client,
            api_key: key,
        })
    }

    #[must_use]
    pub fn new(client: Arc<dyn HttpClient>, api_key: impl Into<String>) -> Self {
        Self {
            client,
            api_key: api_key.into(),
        }
    }

    fn request(&self, url: &str) -> HttpRequest {
        HttpRequest::get(url)
            .with_header("x-api-key", self.api_key.clone())
            .with_header("Accept", "application/json")
    }
}

fn class_id(kind: ContentKind) -> CoreResult<u32> {
    match kind {
        ContentKind::Mod => Ok(6),
        ContentKind::ResourcePack => Ok(12),
        ContentKind::World => Ok(17),
        ContentKind::Shader => Ok(6552),
        ContentKind::Datapack => Ok(6945),
    }
}

fn loader_type(loader: &str) -> u32 {
    match loader {
        "forge" => 1,
        "fabric" => 4,
        "quilt" => 5,
        "neoforge" => 6,
        _ => 0,
    }
}

fn dependency_kind(relation: u32) -> DependencyKind {
    match relation {
        3 => DependencyKind::Required,
        2 => DependencyKind::Optional,
        5 => DependencyKind::Incompatible,
        1 | 6 => DependencyKind::Embedded,
        _ => DependencyKind::Optional,
    }
}

fn is_game_version(token: &str) -> bool {
    token
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
}

fn split_game_versions(tokens: &[String]) -> (Vec<String>, Vec<String>) {
    let mut game_versions = Vec::new();
    let mut loaders = Vec::new();
    for token in tokens {
        if is_game_version(token) {
            game_versions.push(token.clone());
        } else {
            loaders.push(token.to_lowercase());
        }
    }
    (game_versions, loaders)
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    data: Vec<CfMod>,
    #[serde(default)]
    pagination: Pagination,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pagination {
    #[serde(default)]
    index: u32,
    #[serde(default)]
    page_size: u32,
    #[serde(default)]
    total_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfMod {
    id: u64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    slug: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    download_count: u64,
    #[serde(default)]
    authors: Vec<CfAuthor>,
    #[serde(default)]
    logo: Option<CfLogo>,
    #[serde(default)]
    date_modified: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CfAuthor {
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfLogo {
    #[serde(default)]
    thumbnail_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FilesResponse {
    #[serde(default)]
    data: Vec<CfFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfFile {
    id: u64,
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    file_name: String,
    #[serde(default)]
    file_length: Option<u64>,
    #[serde(default)]
    download_url: Option<String>,
    #[serde(default)]
    hashes: Vec<CfHash>,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    dependencies: Vec<CfDependency>,
}

#[derive(Debug, Deserialize)]
struct CfHash {
    #[serde(default)]
    value: String,
    /// 1 is SHA-1, 2 is MD5.
    #[serde(default)]
    algo: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfDependency {
    #[serde(default)]
    mod_id: u64,
    #[serde(default)]
    relation_type: u32,
}

fn sha1_of(file: &CfFile) -> Option<String> {
    file.hashes
        .iter()
        .find(|hash| hash.algo == 1)
        .map(|hash| hash.value.clone())
}

#[async_trait]
impl ContentProvider for CurseForgeProvider {
    fn id(&self) -> ContentProviderId {
        ContentProviderId::CurseForge
    }

    async fn search(&self, request: ContentSearchRequest) -> CoreResult<ContentSearchPage> {
        let mut url = reqwest::Url::parse(&format!("{API}/mods/search")).map_err(|error| {
            CoreError::internal("the CurseForge search URL is invalid")
                .with_details(error.to_string())
        })?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("gameId", &MINECRAFT_GAME_ID.to_string());
            query.append_pair("classId", &class_id(request.kind)?.to_string());
            query.append_pair("index", &request.offset.to_string());
            query.append_pair("pageSize", &request.limit.to_string());
            query.append_pair("sortField", "2");
            query.append_pair("sortOrder", "desc");
            if !request.query.trim().is_empty() {
                query.append_pair("searchFilter", request.query.trim());
            }
            if let Some(game_version) = request.game_version.as_deref() {
                query.append_pair("gameVersion", game_version);
            }
            if let Some(loader) = request.loader.as_deref() {
                let loader = loader_type(loader);
                if loader != 0 {
                    query.append_pair("modLoaderType", &loader.to_string());
                } else {
                    return Err(CoreError::unavailable(
                        "CurseForge does not list content for this loader",
                    ));
                }
            }
        }
        let response: SearchResponse =
            request_json(self.client.as_ref(), self.request(url.as_str())).await?;
        Ok(ContentSearchPage {
            provider: ContentProviderId::CurseForge,
            hits: response
                .data
                .into_iter()
                .map(|project| ContentProject {
                    id: project.id.to_string(),
                    provider: ContentProviderId::CurseForge,
                    kind: request.kind,
                    slug: project.slug,
                    name: project.name,
                    summary: project.summary,
                    icon_url: project.logo.and_then(|logo| logo.thumbnail_url),
                    downloads: project.download_count,
                    author: project
                        .authors
                        .into_iter()
                        .map(|author| author.name)
                        .collect::<Vec<_>>()
                        .join(", "),
                    updated_at: project.date_modified,
                })
                .collect(),
            offset: response.pagination.index,
            limit: response.pagination.page_size,
            total: response.pagination.total_count,
        })
    }

    async fn versions(
        &self,
        project_id: &str,
        filter: &ContentVersionFilter,
    ) -> CoreResult<Vec<ContentVersion>> {
        let mod_id: u64 = project_id
            .trim()
            .parse()
            .map_err(|_| CoreError::invalid_request("a CurseForge project id is numeric"))?;
        let mut url =
            reqwest::Url::parse(&format!("{API}/mods/{mod_id}/files")).map_err(|error| {
                CoreError::internal("the CurseForge files URL is invalid")
                    .with_details(error.to_string())
            })?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("index", "0");
            query.append_pair("pageSize", &PAGE_SIZE.to_string());
            if let Some(game_version) = filter.game_version.as_deref() {
                query.append_pair("gameVersion", game_version);
            }
            if let Some(loader) = filter.loader.as_deref() {
                let loader = loader_type(loader);
                if loader != 0 {
                    query.append_pair("modLoaderType", &loader.to_string());
                }
            }
        }
        let response: FilesResponse =
            request_json(self.client.as_ref(), self.request(url.as_str())).await?;

        let mut versions = Vec::new();
        for file in response.data {
            let Some(url) = file.download_url.clone().filter(|value| !value.is_empty()) else {
                // The author disabled third-party downloads; the file cannot be installed.
                continue;
            };
            let (game_versions, loaders) = split_game_versions(&file.game_versions);
            let sha1 = sha1_of(&file);
            versions.push(ContentVersion {
                id: file.id.to_string(),
                project_id: mod_id.to_string(),
                provider: ContentProviderId::CurseForge,
                name: if file.display_name.is_empty() {
                    file.file_name.clone()
                } else {
                    file.display_name.clone()
                },
                version_number: file.file_name.clone(),
                game_versions,
                loaders,
                files: vec![ContentFile {
                    url,
                    filename: file.file_name,
                    sha1,
                    size: file.file_length,
                    primary: true,
                }],
                dependencies: file
                    .dependencies
                    .into_iter()
                    .map(|dependency| ContentDependency {
                        project_id: (dependency.mod_id != 0).then(|| dependency.mod_id.to_string()),
                        version_id: None,
                        kind: dependency_kind(dependency.relation_type),
                    })
                    .collect(),
            });
        }
        Ok(versions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_match_the_curseforge_catalogue() {
        assert_eq!(class_id(ContentKind::Mod).expect("mod"), 6);
        assert_eq!(class_id(ContentKind::ResourcePack).expect("pack"), 12);
        assert_eq!(class_id(ContentKind::World).expect("world"), 17);
        assert_eq!(class_id(ContentKind::Shader).expect("shader"), 6552);
        assert_eq!(class_id(ContentKind::Datapack).expect("datapack"), 6945);
    }

    #[test]
    fn loader_ids_match_the_api_enum() {
        assert_eq!(loader_type("forge"), 1);
        assert_eq!(loader_type("fabric"), 4);
        assert_eq!(loader_type("quilt"), 5);
        assert_eq!(loader_type("neoforge"), 6);
        assert_eq!(loader_type("vanilla"), 0);
    }

    #[test]
    fn dependency_relations_are_translated() {
        assert_eq!(dependency_kind(3), DependencyKind::Required);
        assert_eq!(dependency_kind(2), DependencyKind::Optional);
        assert_eq!(dependency_kind(5), DependencyKind::Incompatible);
        assert_eq!(dependency_kind(1), DependencyKind::Embedded);
        assert_eq!(dependency_kind(9), DependencyKind::Optional);
    }

    #[test]
    fn loader_tokens_are_separated_from_game_versions() {
        let tokens = vec![
            String::from("1.20.1"),
            String::from("Forge"),
            String::from("1.21.4"),
            String::from("NeoForge"),
        ];
        let (games, loaders) = split_game_versions(&tokens);
        assert_eq!(games, vec!["1.20.1", "1.21.4"]);
        assert_eq!(loaders, vec!["forge", "neoforge"]);
    }

    #[test]
    fn only_the_sha1_hash_is_selected() {
        let file = CfFile {
            id: 1,
            display_name: String::new(),
            file_name: String::new(),
            file_length: None,
            download_url: None,
            hashes: vec![
                CfHash {
                    value: String::from("md5"),
                    algo: 2,
                },
                CfHash {
                    value: String::from("sha1"),
                    algo: 1,
                },
            ],
            game_versions: Vec::new(),
            dependencies: Vec::new(),
        };
        assert_eq!(sha1_of(&file).as_deref(), Some("sha1"));
    }
}
