//! Showing a file in the desktop's own file manager.
//!
//! One argument vector per platform and no shell, so a file name containing spaces,
//! quotes or `&&` stays a file name. The child process is spawned and forgotten:
//! `explorer` on Windows reports a non-zero status even when it did the job, so waiting
//! for it would turn a successful reveal into an error.

use std::path::{Path, PathBuf};

use mnelys_application::ports::EntryRevealer;
use mnelys_domain::{CoreError, CoreResult};

/// Reveals paths with the platform's own file manager.
#[derive(Debug, Clone, Copy, Default)]
pub struct DesktopRevealer;

impl DesktopRevealer {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl EntryRevealer for DesktopRevealer {
    fn reveal(&self, path: &Path) -> CoreResult<()> {
        let (program, arguments) = reveal_command(path);
        std::process::Command::new(program)
            .args(&arguments)
            .spawn()
            .map_err(|error| {
                CoreError::io(format!("the desktop could not reveal {}", path.display()))
                    .with_details(error.to_string())
            })?;
        Ok(())
    }
}

/// The program and its arguments for one path.
fn reveal_command(path: &Path) -> (&'static str, Vec<String>) {
    if cfg!(target_os = "windows") {
        ("explorer", vec![format!("/select,{}", path.display())])
    } else if cfg!(target_os = "macos") {
        ("open", vec![String::from("-R"), path.display().to_string()])
    } else {
        let parent = path.parent().unwrap_or(path);
        ("xdg-open", vec![parent.display().to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_path_never_becomes_two_arguments() {
        let tricky =
            PathBuf::from("/data/instances/main/game/mods").join("a mod (1.0) && echo hi.jar");
        let (program, arguments) = reveal_command(&tricky);
        assert_eq!(arguments.len(), 1, "the path must stay one argument");
        assert!(program != "sh" && program != "cmd", "no shell is used");
        if cfg!(target_os = "windows") {
            assert_eq!(program, "explorer");
            assert!(arguments[0].starts_with("/select,"));
        }
    }
}
