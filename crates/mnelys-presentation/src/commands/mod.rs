//! The command surface. Each submodule owns one area of the core.

mod accounts;
mod content;
mod diagnostics;
mod instances;
mod java;
mod launch;
mod loaders;
mod logs;
mod runtime;
mod settings;
mod tasks;
mod versions;
mod workspace;

/// Version of the command contract.
///
/// Bump this whenever a request or response shape changes. `src/ipc/commands.ts` mirrors
/// the value, and `src-tauri/tests/contract.rs` fails when the two disagree (ADR 0006).
pub const CONTRACT_VERSION: u32 = 5;

/// Every command name this core exposes, in the order the frontend declares them.
pub const COMMAND_NAMES: [&str; 48] = [
    "core_version",
    "runtime_info",
    "app_logs",
    "clear_app_logs",
    "get_settings",
    "update_settings",
    "probe_mirrors",
    "list_versions",
    "list_loaders",
    "list_java",
    "install_java",
    "list_instances",
    "get_instance",
    "create_instance",
    "update_instance",
    "delete_instance",
    "install_instance",
    "launch_instance",
    "stop_instance",
    "launch_sessions",
    "launch_logs",
    "list_accounts",
    "add_offline_account",
    "add_yggdrasil_account",
    "begin_microsoft_login",
    "complete_microsoft_login",
    "select_account",
    "remove_account",
    "refresh_account",
    "content_providers",
    "search_content",
    "content_versions",
    "installed_content",
    "install_content",
    "remove_content",
    "export_instance",
    "import_instance",
    "list_instance_files",
    "list_instance_logs",
    "read_instance_log",
    "list_instance_worlds",
    "list_instance_schematics",
    "delete_instance_entry",
    "reveal_instance_entry",
    "diagnose_instance",
    "export_instance_diagnostics",
    "list_tasks",
    "cancel_task",
];

/// Registers every command on a Tauri builder.
///
/// `generate_handler!` expands here, next to the submodules that declare the commands,
/// so Tauri's hidden per-command macros resolve inside this crate.
#[must_use]
pub fn attach<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        runtime::core_version,
        runtime::runtime_info,
        logs::app_logs,
        logs::clear_app_logs,
        settings::get_settings,
        settings::update_settings,
        settings::probe_mirrors,
        versions::list_versions,
        loaders::list_loaders,
        java::list_java,
        java::install_java,
        instances::list_instances,
        instances::get_instance,
        instances::create_instance,
        instances::update_instance,
        instances::delete_instance,
        instances::install_instance,
        launch::launch_instance,
        launch::stop_instance,
        launch::launch_sessions,
        launch::launch_logs,
        accounts::list_accounts,
        accounts::add_offline_account,
        accounts::add_yggdrasil_account,
        accounts::begin_microsoft_login,
        accounts::complete_microsoft_login,
        accounts::select_account,
        accounts::remove_account,
        accounts::refresh_account,
        content::content_providers,
        content::search_content,
        content::content_versions,
        content::installed_content,
        content::install_content,
        content::remove_content,
        content::export_instance,
        content::import_instance,
        workspace::list_instance_files,
        workspace::list_instance_logs,
        workspace::read_instance_log,
        workspace::list_instance_worlds,
        workspace::list_instance_schematics,
        workspace::delete_instance_entry,
        workspace::reveal_instance_entry,
        diagnostics::diagnose_instance,
        diagnostics::export_instance_diagnostics,
        tasks::list_tasks,
        tasks::cancel_task
    ])
}
