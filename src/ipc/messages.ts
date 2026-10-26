import type { TFunction } from 'i18next';

import { CoreCommandError } from './client';
import type { CoreErrorCode } from './errors';

/**
 * Stable code to localized sentence. The core's own message is diagnostic English and
 * stays in the log; the interface never mixes languages, so only the code is shown.
 */
const ERROR_KEYS = {
  'core/unavailable': 'error.coreUnavailable',
  'core/not-implemented': 'error.coreNotImplemented',
  'core/invalid-request': 'error.coreInvalidRequest',
  'core/io': 'error.coreIo',
  'core/network': 'error.coreNetwork',
  'core/unauthorized': 'error.coreUnauthorized',
  'core/conflict': 'error.coreConflict',
  'core/internal': 'error.coreInternal',
} as const satisfies Readonly<Record<CoreErrorCode, string>>;

/** Turns any rejection into one localized sentence a page can render as-is. */
export function describeError(error: unknown, t: TFunction): string {
  if (error instanceof CoreCommandError) {
    return t(ERROR_KEYS[error.code]);
  }
  if (error instanceof Error && error.message.length > 0) {
    return error.message;
  }
  return t('error.unknown');
}
