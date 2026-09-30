/**
 * Error shape returned by the Rust core.
 *
 * ADR 0006: failures cross the command boundary as typed results carrying a stable
 * code, never as thrown strings. The UI maps `code` to a localized message in
 * `src/i18n/locales/*.json` under the `error` namespace.
 */
export const CORE_ERROR_CODES = [
  'core/unavailable',
  'core/not-implemented',
  'core/invalid-request',
  'core/io',
  'core/network',
  'core/unauthorized',
  'core/conflict',
  'core/internal',
] as const;

export type CoreErrorCode = (typeof CORE_ERROR_CODES)[number];

export interface CoreError {
  readonly code: CoreErrorCode;
  readonly message: string;
  /** Free-form diagnostic context from the core. Never contains secrets. */
  readonly details?: string;
}

export function isCoreErrorCode(value: unknown): value is CoreErrorCode {
  return typeof value === 'string' && (CORE_ERROR_CODES as readonly string[]).includes(value);
}

/** Narrows an unknown rejection payload to the structured error the core promises. */
export function isCoreError(value: unknown): value is CoreError {
  if (typeof value !== 'object' || value === null) {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  return (
    isCoreErrorCode(candidate['code']) &&
    typeof candidate['message'] === 'string' &&
    (candidate['details'] === undefined || typeof candidate['details'] === 'string')
  );
}
