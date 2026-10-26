//! Launch plans and live game sessions.
//!
//! A plan is a fully expanded command line plus the environment it runs in. Building
//! it without starting anything makes the hard part — placeholders, classpath and
//! natives — testable on a machine with no Java installed.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A ready-to-run Minecraft invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPlan {
    pub instance_id: String,
    pub version_id: String,
    pub java_path: String,
    pub main_class: String,
    pub classpath: Vec<String>,
    #[serde(default)]
    pub native_path: Option<String>,
    pub game_directory: String,
    pub assets_directory: String,
    pub asset_index: String,
    pub jvm_arguments: Vec<String>,
    pub game_arguments: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub memory_mb: u32,
}

impl LaunchPlan {
    /// The full argument vector, with the executable first. Useful for diagnostics and
    /// for asserting in tests that placeholders were expanded.
    #[must_use]
    pub fn command_line(&self) -> Vec<String> {
        let mut command = vec![self.java_path.clone()];
        if self.memory_mb > 0 {
            command.push(format!("-Xmx{}M", self.memory_mb));
        }
        command.extend(self.jvm_arguments.iter().cloned());
        command.push("-cp".to_owned());
        command.push(self.classpath.join(classpath_separator()));
        command.push(self.main_class.clone());
        command.extend(self.game_arguments.iter().cloned());
        command
    }
}

/// The platform classpath separator. Kept as a function so tests can assert both.
#[must_use]
pub const fn classpath_separator() -> &'static str {
    if cfg!(windows) {
        ";"
    } else {
        ":"
    }
}

/// Expands `${placeholder}` tokens.
///
/// Unknown tokens are left intact on purpose: a loader that introduces a variable
/// Mnelys does not know yet should produce a diagnosable command line, not a silently
/// empty argument.
#[must_use]
pub fn render_arguments(template: &str, variables: &BTreeMap<String, String>) -> String {
    let mut rendered = template.to_owned();
    for (key, value) in variables {
        rendered = rendered.replace(&format!("${{{key}}}"), value);
    }
    rendered
}

/// Applies [`render_arguments`] to a whole argument list.
#[must_use]
pub fn render_argument_list(
    arguments: &[String],
    variables: &BTreeMap<String, String>,
) -> Vec<String> {
    arguments
        .iter()
        .map(|argument| render_arguments(argument, variables))
        .collect()
}

/// Lifecycle of a game process Mnelys started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LaunchState {
    Running,
    Exited,
    Failed,
    Stopped,
}

impl LaunchState {
    #[must_use]
    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }
}

/// One spawned game process as the frontend sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchSession {
    pub id: String,
    pub instance_id: String,
    pub version_id: String,
    pub pid: u32,
    pub state: LaunchState,
    pub started_at_ms: u64,
    #[serde(default)]
    pub finished_at_ms: Option<u64>,
    #[serde(default)]
    pub exit_code: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_placeholder_is_expanded() {
        let variables = BTreeMap::from([
            (String::from("auth_player_name"), String::from("Steve")),
            (String::from("version_name"), String::from("1.21.4")),
        ]);
        assert_eq!(
            render_arguments(
                "--username ${auth_player_name} --version ${version_name}",
                &variables
            ),
            "--username Steve --version 1.21.4"
        );
    }

    #[test]
    fn an_unknown_placeholder_stays_visible() {
        assert_eq!(
            render_arguments("--x ${mystery}", &BTreeMap::new()),
            "--x ${mystery}"
        );
    }

    #[test]
    fn a_plan_renders_memory_classpath_and_arguments_in_order() {
        let plan = LaunchPlan {
            instance_id: String::from("main"),
            version_id: String::from("1.21.4"),
            java_path: String::from("java"),
            main_class: String::from("net.minecraft.client.main.Main"),
            classpath: vec![String::from("a.jar"), String::from("b.jar")],
            native_path: None,
            game_directory: String::from("/game"),
            assets_directory: String::from("/assets"),
            asset_index: String::from("17"),
            jvm_arguments: vec![String::from("-Dfile.encoding=UTF-8")],
            game_arguments: vec![String::from("--username"), String::from("Steve")],
            environment: BTreeMap::new(),
            memory_mb: 2048,
        };
        let command = plan.command_line();
        assert_eq!(command[0], "java");
        assert_eq!(command[1], "-Xmx2048M");
        assert_eq!(command[2], "-Dfile.encoding=UTF-8");
        assert_eq!(command[3], "-cp");
        assert_eq!(command[4], format!("a.jar{}b.jar", classpath_separator()));
        assert_eq!(command[5], "net.minecraft.client.main.Main");
        assert_eq!(command.last().map(String::as_str), Some("Steve"));
    }
}
