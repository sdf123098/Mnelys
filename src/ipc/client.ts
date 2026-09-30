import { invoke } from '@tauri-apps/api/core';

import type { CommandArgs, CommandName, CommandResult } from './commands';
import { isCoreError, type CoreError, type CoreErrorCode } from './errors';

/**
 * Thrown by every wrapper in this module so callers never have to inspect raw
 * rejection payloads. `code` is stable and safe to branch on.
 */
export class CoreCommandError extends Error {
  readonly code: CoreErrorCode;
  readonly details: string | undefined;

  constructor(error: CoreError) {
    super(error.message);
    this.name = 'CoreCommandError';
    this.code = error.code;
    this.details = error.details;
  }
}

async function call<K extends CommandName>(
  name: K,
  args: CommandArgs<K>,
): Promise<CommandResult<K>> {
  try {
    return await invoke<CommandResult<K>>(name, args);
  } catch (cause) {
    if (isCoreError(cause)) {
      throw new CoreCommandError(cause);
    }
    throw new CoreCommandError({
      code: 'core/internal',
      message:
        typeof cause === 'string' && cause.length > 0
          ? cause
          : 'The Mnelys core failed without returning a structured error.',
    });
  }
}

/**
 * The only door between the frontend and the Rust core (ADR 0006).
 *
 * Components import these named wrappers. Importing `invoke` anywhere else is an
 * ESLint error; importing a Tauri plugin directly is one too, so that the native
 * capability surface stays reviewable (ADR 0005).
 */
export const core = {
  /** Start-up handshake: proves the boundary is live and versions agree. */
  version: (): Promise<CommandResult<'core_version'>> => call('core_version', {}),
  /** Host facts the UI adapts to. Never used to branch a security decision. */
  runtimeInfo: (): Promise<CommandResult<'runtime_info'>> => call('runtime_info', {}),
} as const;

export type MnelysCore = typeof core;
