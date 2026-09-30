//! The command surface. Each submodule owns one area of the core.

mod runtime;

// `#[tauri::command]` emits a hidden macro pair next to each function
// (`__cmd__<name>` and `__tauri_command_name_<name>`), and `generate_handler!` looks them
// up under the same path as the command. Re-exporting them here is what lets `lib.rs`
// write `commands::core_version` while the implementation lives in a submodule.
pub use runtime::{
    __cmd__core_version, __cmd__runtime_info, __tauri_command_name_core_version,
    __tauri_command_name_runtime_info, core_version, runtime_info, COMMAND_NAMES, CONTRACT_VERSION,
};
