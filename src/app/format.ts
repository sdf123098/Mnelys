const SIZE_UNITS = ['B', 'KiB', 'MiB', 'GiB', 'TiB'] as const;

/**
 * Binary units, because a mod folder is judged in MiB and a world in GiB. Units are
 * never localized: they are the ones the file manager shows on this machine.
 */
export function formatBytes(value: number): string {
  let size = Math.max(value, 0);
  let unit = 0;
  while (size >= 1024 && unit < SIZE_UNITS.length - 1) {
    size /= 1024;
    unit += 1;
  }
  const digits = unit === 0 || size >= 100 ? 0 : 1;
  return `${size.toFixed(digits)} ${SIZE_UNITS[unit] ?? 'B'}`;
}

/**
 * Day plus minute, in the language the shell is showing rather than the OS locale. Seconds
 * are left out on purpose: in a file listing they are noise, and dropping them is what
 * keeps the size and the stamp on one line.
 */
export function formatDateTime(locale: string, milliseconds: number): string {
  return new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(milliseconds);
}
