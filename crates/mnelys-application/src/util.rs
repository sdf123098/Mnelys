//! Small pure helpers shared by the application services.

use mnelys_domain::CoreError;
use sha1::{Digest, Sha1};

/// Lowercase hex SHA-1 of a byte slice.
#[must_use]
pub fn sha1_hex(bytes: &[u8]) -> String {
    let digest = Sha1::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Whether a cached file matches a published hash.
#[must_use]
pub fn hash_matches(bytes: &[u8], expected: Option<&str>) -> bool {
    match expected {
        Some(expected) => sha1_hex(bytes).eq_ignore_ascii_case(expected),
        None => true,
    }
}

/// Version ids become directory names. Reject anything that could escape.
pub fn validate_version_id(value: &str) -> Result<(), CoreError> {
    let valid = !value.is_empty()
        && value.len() <= 96
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._+-".contains(character))
        && value != "."
        && value != "..";
    if valid {
        Ok(())
    } else {
        Err(CoreError::invalid_request(format!(
            "`{value}` is not a safe version id"
        )))
    }
}

/// Joins a base URL and a relative path without doubling slashes.
#[must_use]
pub fn join_url(base: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

/// Parses JSON with a message that names the document, not the parser.
pub fn parse_document<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    label: &str,
) -> Result<T, CoreError> {
    serde_json::from_slice(bytes).map_err(|error| {
        CoreError::internal(format!("{label} is not valid JSON")).with_details(error.to_string())
    })
}

/// The file name of the last segment of a URL path.
#[must_use]
pub fn url_file_name(url: &str) -> Option<&str> {
    let path = url.split('?').next().unwrap_or(url);
    let name = path.rsplit('/').next()?;
    if name.is_empty() || name == "." || name == ".." {
        None
    } else {
        Some(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_matches_a_known_value() {
        assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
    }

    #[test]
    fn unsafe_version_ids_are_rejected() {
        assert!(validate_version_id("1.21.4").is_ok());
        assert!(validate_version_id("fabric-loader-0.16.9-1.21.4").is_ok());
        for bad in ["", "../x", "a/b", "a\\b"] {
            assert!(validate_version_id(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn urls_join_without_duplicate_slashes() {
        assert_eq!(
            join_url("https://example.test/maven/", "/org/a.jar"),
            "https://example.test/maven/org/a.jar"
        );
    }

    #[test]
    fn a_file_name_is_extracted_from_a_query_url() {
        assert_eq!(
            url_file_name("https://example.test/log4j.xml?x=1"),
            Some("log4j.xml")
        );
        assert_eq!(url_file_name("https://example.test/"), None);
    }
}
