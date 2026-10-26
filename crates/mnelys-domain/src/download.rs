//! Download plans: the complete, validated list of files an install needs.
//!
//! A plan is built before a single byte is fetched, so the UI can show a real total,
//! duplicate destinations fail before they can corrupt a cache, and every path is
//! proven to stay inside the game directory.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// Why a file is being downloaded. Natives are extracted after transfer; everything
/// else is stored at its planned path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadKind {
    Client,
    Library,
    Native,
    Asset,
    Logging,
    Java,
    Installer,
    Content,
    Other,
}

/// One file in a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItem {
    pub url: String,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    /// Forward-slash path relative to the root the plan is applied to.
    pub path: String,
    pub kind: DownloadKind,
}

impl DownloadItem {
    #[must_use]
    pub fn new(
        url: impl Into<String>,
        path: impl Into<String>,
        kind: DownloadKind,
        sha1: Option<String>,
        size: Option<u64>,
    ) -> Self {
        Self {
            url: url.into(),
            sha1,
            size,
            path: path.into(),
            kind,
        }
    }
}

/// The validated list of files one operation needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadPlan {
    items: Vec<DownloadItem>,
}

impl DownloadPlan {
    /// Validates and de-duplicates a plan.
    ///
    /// The same destination may appear more than once only when URL and hash agree:
    /// some loader profiles list a library twice. A genuine conflict is an error, not a
    /// silent last-writer-wins.
    pub fn new(items: Vec<DownloadItem>) -> Result<Self, CoreError> {
        let mut by_path: BTreeMap<String, DownloadItem> = BTreeMap::new();
        for item in items {
            validate_relative_path(&item.path)?;
            if !(item.url.starts_with("https://") || item.url.starts_with("http://")) {
                return Err(CoreError::invalid_request(format!(
                    "download `{}` does not use http or https",
                    item.path
                )));
            }
            if let Some(sha1) = &item.sha1 {
                if sha1.len() != 40 || !sha1.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(CoreError::invalid_request(format!(
                        "download `{}` has an invalid SHA-1",
                        item.path
                    )));
                }
            }
            match by_path.get(&item.path) {
                Some(existing) => {
                    let same = existing.url == item.url && existing.sha1 == item.sha1;
                    if !same {
                        return Err(CoreError::conflict(format!(
                            "two downloads target `{}`",
                            item.path
                        )));
                    }
                }
                None => {
                    by_path.insert(item.path.clone(), item);
                }
            }
        }
        Ok(Self {
            items: by_path.into_values().collect(),
        })
    }

    #[must_use]
    pub fn items(&self) -> &[DownloadItem] {
        &self.items
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Sum of the sizes Mojang published. Unknown sizes count as zero, so this is a
    /// lower bound the UI labels as such.
    #[must_use]
    pub fn known_bytes(&self) -> u64 {
        self.items.iter().filter_map(|item| item.size).sum()
    }

    /// How many files have no published size.
    #[must_use]
    pub fn unknown_size_count(&self) -> usize {
        self.items.iter().filter(|item| item.size.is_none()).count()
    }

    /// Splits a plan into files of the given kind.
    #[must_use]
    pub fn of_kind(&self, kind: DownloadKind) -> Vec<&DownloadItem> {
        self.items.iter().filter(|item| item.kind == kind).collect()
    }
}

/// Rejects absolute paths, parent traversal, backslashes and empty segments.
///
/// Plans are built from network metadata, so this is a boundary check, not a
/// convenience.
pub fn validate_relative_path(path: &str) -> Result<(), CoreError> {
    if path.is_empty() || path.len() > 512 {
        return Err(CoreError::invalid_request(
            "download path must be between 1 and 512 characters",
        ));
    }
    if path.contains('\\') || path.contains(':') || path.contains('\0') {
        return Err(CoreError::invalid_request(format!(
            "download path `{path}` contains a forbidden character"
        )));
    }
    if path.starts_with('/') {
        return Err(CoreError::invalid_request(format!(
            "download path `{path}` must be relative"
        )));
    }
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(CoreError::invalid_request(format!(
                "download path `{path}` contains an unsafe segment"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(path: &str, url: &str) -> DownloadItem {
        DownloadItem::new(url, path, DownloadKind::Library, None, Some(10))
    }

    #[test]
    fn identical_duplicates_collapse() {
        let plan = DownloadPlan::new(vec![
            item("libraries/a.jar", "https://example.test/a.jar"),
            item("libraries/a.jar", "https://example.test/a.jar"),
        ])
        .expect("valid plan");
        assert_eq!(plan.len(), 1);
        assert_eq!(plan.known_bytes(), 10);
    }

    #[test]
    fn conflicting_destinations_are_rejected() {
        let error = DownloadPlan::new(vec![
            item("libraries/a.jar", "https://example.test/a.jar"),
            item("libraries/a.jar", "https://example.test/b.jar"),
        ])
        .expect_err("conflict");
        assert_eq!(error.code, crate::CoreErrorCode::Conflict);
    }

    #[test]
    fn traversal_and_absolute_paths_are_rejected() {
        for path in [
            "../escape.jar",
            "/etc/passwd",
            "a//b.jar",
            "C:/x.jar",
            "a\\b.jar",
        ] {
            assert!(validate_relative_path(path).is_err(), "{path}");
        }
        assert!(validate_relative_path("libraries/org/example/a.jar").is_ok());
    }

    #[test]
    fn an_invalid_hash_is_rejected() {
        let item = DownloadItem::new(
            "https://example.test/a.jar",
            "a.jar",
            DownloadKind::Client,
            Some(String::from("nope")),
            None,
        );
        assert!(DownloadPlan::new(vec![item]).is_err());
    }

    #[test]
    fn unknown_sizes_are_reported() {
        let plan = DownloadPlan::new(vec![DownloadItem::new(
            "https://example.test/a.jar",
            "a.jar",
            DownloadKind::Client,
            None,
            None,
        )])
        .expect("valid");
        assert_eq!(plan.known_bytes(), 0);
        assert_eq!(plan.unknown_size_count(), 1);
    }
}
