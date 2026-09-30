/**
 * The typed Rust command surface.
 *
 * ADR 0005 requires the Rust/frontend boundary to be typed and versioned; ADR 0006
 * makes it a single contract with exactly one client. This file is the TypeScript
 * half of the contract implemented in `src-tauri/src/commands/`.
 *
 * `CONTRACT_VERSION` must equal the `contractVersion` the core reports through
 * `core_version`. `src/ipc/contract.test.ts` checks that the command names declared
 * here match the ones the Rust side registers.
 */

export const CONTRACT_VERSION = 1;

/** Result of `core_version` — the start-up handshake that keeps both halves in step. */
export interface CoreVersion {
  readonly version: string;
  readonly contractVersion: number;
  readonly profile: 'debug' | 'release';
}

/** Result of `runtime_info` — host facts the UI adapts to. */
export interface RuntimeInfo {
  readonly os: string;
  readonly arch: string;
  readonly webview: string;
  readonly family: 'webview2' | 'wkwebview' | 'webkitgtk' | 'unknown';
}

/**
 * Command name to argument and result types. Commands that take no arguments
 * declare `Record<string, never>` so the client keeps one call signature.
 */
export interface CommandContract {
  core_version: { args: Record<string, never>; result: CoreVersion };
  runtime_info: { args: Record<string, never>; result: RuntimeInfo };
}

export type CommandName = keyof CommandContract;
export type CommandArgs<K extends CommandName> = CommandContract[K]['args'];
export type CommandResult<K extends CommandName> = CommandContract[K]['result'];

/** Every command name, as a runtime value, for contract tests and diagnostics. */
export const COMMAND_NAMES = [
  'core_version',
  'runtime_info',
] as const satisfies readonly CommandName[];
