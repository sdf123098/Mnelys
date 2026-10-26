//! The Modrinth content provider.
//!
//! Modrinth needs no key. Searching and browsing versions use the public v2 API, and
//! every file carries a SHA-1 the installer verifies before writing.

use std::sync::Arc;

use async_trait::async_trait;
use mnelys_application::ports::{
    fetch_json, ContentProvider, ContentSearchRequest, ContentVersionFilter, HttpClient,
};
use mnelys_domain::{
    ContentDependency, ContentFile, ContentKind, ContentProject, ContentProviderId,
    ContentSearchPage, ContentVersion, CoreError, CoreResult, DependencyKind,
};
use serde::Deserialize;
use serde_json::json;

const API: &str = "https://api.modrinth.com/v2";

/// Talks to Modrinth's public API.
pub struct ModrinthProvider {
    client: Arc<dyn HttpClient>,
}

impl ModrinthProvider {
    #[must_use]
    pub fn new(client: Arc<dyn HttpClient>) -> Self {
        Self { client }
    }
}

fn project_type(kind: ContentKind) -> CoreResult<&'static str> {
    match kind {
        ContentKind::Mod => Ok("mod"),
        ContentKind::ResourcePack => Ok("resourcepack"),
        ContentKind::Shader => Ok("shader"),
        ContentKind::Datapack => Ok("datapack"),
        ContentKind::World => Err(CoreError::unavailable(
            "Modrinth does not host world downloads",
        )),
    }
}

fn facets(request: &ContentSearchRequest) -> CoreResult<String> {
    let mut facets = vec![vec![format!(
        "project_type:{}",
        project_type(request.kind)?
    )]];
    if let Some(game_version) = request.game_version.as_deref() {
        facets.push(vec![format!("versions:{game_version}")]);
    }
    if let (Some(loader), ContentKind::Mod) = (request.loader.as_deref(), request.kind) {
        facets.push(vec![format!("categories:{loader}")]);
    }
    Ok(json!(facets).to_string())
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    hits: Vec<SearchHit>,
    offset: u32,
    limit: u32,
    total_hits: u64,
}

#[derive(Debug, Deserialize)]
struct SearchHit {
    project_id: String,
    slug: String,
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    icon_url: Option<String>,
    #[serde(default)]
    downloads: u64,
    #[serde(default)]
    author: String,
    #[serde(default)]
    date_modified: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    id: String,
    project_id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    version_number: String,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    loaders: Vec<String>,
    #[serde(default)]
    files: Vec<FileResponse>,
    #[serde(default)]
    dependencies: Vec<DependencyResponse>,
}

#[derive(Debug, Deserialize)]
struct FileResponse {
    url: String,
    filename: String,
    #[serde(default)]
    hashes: FileHashes,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    primary: bool,
}

#[derive(Debug, Default, Deserialize)]
struct FileHashes {
    #[serde(default)]
    sha1: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DependencyResponse {
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    version_id: Option<String>,
    #[serde(default)]
    dependency_type: String,
}

fn dependency_kind(value: &str) -> DependencyKind {
    match value {
        "required" => DependencyKind::Required,
        "incompatible" => DependencyKind::Incompatible,
        "embedded" => DependencyKind::Embedded,
        _ => DependencyKind::Optional,
    }
}

#[async_trait]
impl ContentProvider for ModrinthProvider {
    fn id(&self) -> ContentProviderId {
        ContentProviderId::Modrinth
    }

    async fn search(&self, request: ContentSearchRequest) -> CoreResult<ContentSearchPage> {
        let mut url = reqwest::Url::parse(&format!("{API}/search")).map_err(|error| {
            CoreError::internal("the Modrinth search URL is invalid")
                .with_details(error.to_string())
        })?;
        let facets = facets(&request)?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("query", &request.query);
            query.append_pair("facets", &facets);
            query.append_pair("limit", &request.limit.to_string());
            query.append_pair("offset", &request.offset.to_string());
        }
        let response: SearchResponse = fetch_json(self.client.as_ref(), url.as_str()).await?;
        Ok(ContentSearchPage {
            provider: ContentProviderId::Modrinth,
            hits: response
                .hits
                .into_iter()
                .map(|hit| ContentProject {
                    id: hit.project_id,
                    provider: ContentProviderId::Modrinth,
                    kind: request.kind,
                    slug: hit.slug,
                    name: hit.title,
                    summary: hit.description,
                    icon_url: hit.icon_url,
                    downloads: hit.downloads,
                    author: hit.author,
                    updated_at: hit.date_modified,
                })
                .collect(),
            offset: response.offset,
            limit: response.limit,
            total: response.total_hits,
        })
    }

    async fn versions(
        &self,
        project_id: &str,
        filter: &ContentVersionFilter,
    ) -> CoreResult<Vec<ContentVersion>> {
        if project_id.trim().is_empty() {
            return Err(CoreError::invalid_request(
                "a Modrinth project id is required",
            ));
        }
        let mut url = reqwest::Url::parse(&format!("{API}/project/{project_id}/version")).map_err(
            |error| {
                CoreError::invalid_request("the Modrinth project id is not valid")
                    .with_details(error.to_string())
            },
        )?;
        {
            let mut query = url.query_pairs_mut();
            if let Some(game_version) = filter.game_version.as_deref() {
                query.append_pair("game_versions", &json!([game_version]).to_string());
            }
            if let Some(loader) = filter.loader.as_deref() {
                query.append_pair("loaders", &json!([loader]).to_string());
            }
        }
        let versions: Vec<VersionResponse> = fetch_json(self.client.as_ref(), url.as_str()).await?;
        Ok(versions
            .into_iter()
            .map(|version| ContentVersion {
                id: version.id,
                project_id: version.project_id,
                provider: ContentProviderId::Modrinth,
                name: if version.name.is_empty() {
                    version.version_number.clone()
                } else {
                    version.name
                },
                version_number: version.version_number,
                game_versions: version.game_versions,
                loaders: version.loaders,
                files: version
                    .files
                    .into_iter()
                    .map(|file| ContentFile {
                        url: file.url,
                        filename: file.filename,
                        sha1: file.hashes.sha1,
                        size: file.size,
                        primary: file.primary,
                    })
                    .collect(),
                dependencies: version
                    .dependencies
                    .into_iter()
                    .map(|dependency| ContentDependency {
                        project_id: dependency.project_id,
                        version_id: dependency.version_id,
                        kind: dependency_kind(&dependency.dependency_type),
                    })
                    .collect(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(kind: ContentKind) -> ContentSearchRequest {
        ContentSearchRequest {
            query: String::from("sodium"),
            kind,
            game_version: Some(String::from("1.21.4")),
            loader: Some(String::from("fabric")),
            offset: 0,
            limit: 20,
        }
    }

    #[test]
    fn facets_carry_type_version_and_loader() {
        let facets = facets(&request(ContentKind::Mod)).expect("facets");
        let parsed: serde_json::Value = serde_json::from_str(&facets).expect("json");
        let arrays = parsed.as_array().expect("array");
        assert!(arrays
            .iter()
            .any(|entry| entry[0].as_str() == Some("project_type:mod")));
        assert!(arrays
            .iter()
            .any(|entry| entry[0].as_str() == Some("versions:1.21.4")));
        assert!(arrays
            .iter()
            .any(|entry| entry[0].as_str() == Some("categories:fabric")));
    }

    #[test]
    fn a_world_search_is_rejected() {
        assert!(facets(&request(ContentKind::World)).is_err());
    }

    #[test]
    fn resource_packs_do_not_filter_by_loader() {
        let facets = facets(&request(ContentKind::ResourcePack)).expect("facets");
        assert!(!facets.contains("categories"));
    }

    #[test]
    fn dependency_types_map_to_their_vocabulary() {
        assert_eq!(dependency_kind("required"), DependencyKind::Required);
        assert_eq!(
            dependency_kind("incompatible"),
            DependencyKind::Incompatible
        );
        assert_eq!(dependency_kind("embedded"), DependencyKind::Embedded);
        assert_eq!(dependency_kind("optional"), DependencyKind::Optional);
        assert_eq!(dependency_kind("unknown"), DependencyKind::Optional);
    }
}
