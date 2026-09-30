//! Guards the command contract against drift between the Rust core and the frontend.
//!
//! The two sides are separate toolchains and neither can import the other, so these
//! tests read the TypeScript sources directly. A rename on one side without the other
//! fails the build rather than the app (ADR 0006).

use std::fs;
use std::path::PathBuf;

fn frontend_source(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

#[test]
fn the_contract_version_matches_the_frontend() {
    let source = frontend_source("src/ipc/commands.ts");
    let expected = format!("CONTRACT_VERSION = {}", mnelys_lib::CONTRACT_VERSION);
    assert!(
        source.contains(&expected),
        "src/ipc/commands.ts does not declare `{expected}`"
    );
}

#[test]
fn every_command_is_declared_by_the_frontend() {
    let source = frontend_source("src/ipc/commands.ts");
    for name in mnelys_lib::COMMAND_NAMES {
        assert!(
            source.contains(&format!("'{name}'")),
            "src/ipc/commands.ts does not declare command `{name}`"
        );
    }
}

#[test]
fn every_error_code_is_declared_by_the_frontend() {
    let source = frontend_source("src/ipc/errors.ts");
    for code in mnelys_lib::CoreErrorCode::ALL {
        assert!(
            source.contains(&format!("'{}'", code.as_str())),
            "src/ipc/errors.ts does not declare `{}`",
            code.as_str()
        );
    }
}
