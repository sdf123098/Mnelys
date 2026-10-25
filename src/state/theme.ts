import type { ThemePreference } from './preferences-store';

/**
 * The token layer honours the system preference by default. An explicit choice sets
 * `data-theme`, which both palettes key off; `system` removes the attribute instead of
 * copying a snapshot of the current OS theme.
 */
export function applyTheme(
  theme: ThemePreference,
  root: HTMLElement = document.documentElement,
): void {
  if (theme === 'system') {
    delete root.dataset['theme'];
    return;
  }
  root.dataset['theme'] = theme;
}
