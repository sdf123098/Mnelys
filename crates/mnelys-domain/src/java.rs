//! Java runtimes the launcher can use.

use serde::{Deserialize, Serialize};

/// Where a runtime came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JavaSource {
    /// A runtime Mnelys downloaded and owns.
    Managed,
    /// `JAVA_HOME`, `PATH` or a well-known install directory.
    System,
}

/// One local Java installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaRuntime {
    /// Absolute path to the `java` executable.
    pub path: String,
    pub major: u32,
    /// Full version string as the JVM reports it, for example `21.0.4+7-LTS`.
    pub version: String,
    pub vendor: String,
    pub source: JavaSource,
}

impl JavaRuntime {
    /// Whether this runtime can run a version that asks for `required`.
    ///
    /// Newer majors are accepted: Minecraft runs on newer JDKs far more often than it
    /// fails on them, and refusing would strand users whose only JDK is current.
    #[must_use]
    pub fn satisfies(&self, required: u32) -> bool {
        self.major >= required
    }

    /// Lower ranks are preferred when two runtimes satisfy the same requirement.
    #[must_use]
    pub const fn rank(&self) -> u8 {
        match self.source {
            JavaSource::Managed => 0,
            JavaSource::System => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_newer_jdk_satisfies_an_older_requirement() {
        let runtime = JavaRuntime {
            path: String::from("/usr/bin/java"),
            major: 21,
            version: String::from("21.0.4"),
            vendor: String::from("Eclipse Adoptium"),
            source: JavaSource::System,
        };
        assert!(runtime.satisfies(17));
        assert!(runtime.satisfies(21));
        assert!(!runtime.satisfies(22));
    }
}
