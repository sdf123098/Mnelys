//! Where Mnelys keeps its data on this machine.

use std::path::PathBuf;

/// The environment variable that overrides the data root.
///
/// Portables builds and tests use it; nothing else should have to know the layout.
pub const HOME_ENV: &str = "MNELYS_HOME";

/// The default data root.
///
/// `MNELYS_HOME` wins, then the platform data directory (`%APPDATA%\Mnelys`,
/// `~/Library/Application Support/Mnelys`, `~/.local/share/Mnelys`). The last fallback
/// keeps the launcher usable on a stripped-down system where `dirs` returns nothing.
#[must_use]
pub fn default_root() -> PathBuf {
    if let Some(value) = std::env::var_os(HOME_ENV) {
        let path = PathBuf::from(value);
        if !path.as_os_str().is_empty() {
            return path;
        }
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Mnelys")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_root_ends_with_the_product_name() {
        let root = default_root();
        assert_eq!(
            root.file_name()
                .map(|name| name.to_string_lossy().into_owned()),
            Some(String::from("Mnelys"))
        );
    }
}
