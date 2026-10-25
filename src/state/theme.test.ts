import { afterEach, describe, expect, it } from 'vitest';

import { BRAND_ACCENT } from './preferences-store';
import {
  applyAccent,
  applyTheme,
  contrastRatio,
  deriveAccent,
  parseHexColor,
  toHex,
} from './theme';

afterEach(() => {
  const root = document.documentElement;
  delete root.dataset['theme'];
  delete root.dataset['accent'];
  root.removeAttribute('style');
});

describe('applyTheme', () => {
  it('lets the system preference win by removing the override', () => {
    document.documentElement.dataset['theme'] = 'dark';

    applyTheme('system');

    expect(document.documentElement).not.toHaveAttribute('data-theme');
  });

  it('writes an explicit palette to the root element', () => {
    applyTheme('light');
    expect(document.documentElement).toHaveAttribute('data-theme', 'light');

    applyTheme('dark');
    expect(document.documentElement).toHaveAttribute('data-theme', 'dark');
  });
});

describe('parseHexColor', () => {
  it('expands short hex and normalises output', () => {
    const parsed = parseHexColor('#c18');
    expect(parsed).not.toBeNull();
    expect(parsed === null ? '' : toHex(parsed)).toBe('#CC1188');
  });

  it('rejects values that are not colours', () => {
    expect(parseHexColor('pink')).toBeNull();
    expect(parseHexColor('#12345')).toBeNull();
    expect(parseHexColor('')).toBeNull();
  });
});

describe('deriveAccent', () => {
  it('rejects an invalid colour', () => {
    expect(deriveAccent('not-a-colour', 'light')).toBeNull();
  });

  it.each([
    ['light', '#C2185B'],
    ['dark', '#F48FB1'],
  ] as const)('keeps %s accent text readable', (theme, input) => {
    const tokens = deriveAccent(input, theme);
    expect(tokens).not.toBeNull();
    if (tokens === null) {
      return;
    }

    const accent = parseHexColor(tokens.accent);
    const onAccent = parseHexColor(tokens.onAccent);
    const soft = parseHexColor(tokens.soft);
    expect(accent).not.toBeNull();
    expect(onAccent).not.toBeNull();
    expect(soft).not.toBeNull();
    if (accent === null || onAccent === null || soft === null) {
      return;
    }

    expect(contrastRatio(accent, onAccent)).toBeGreaterThanOrEqual(4.5);
    expect(contrastRatio(accent, soft)).toBeGreaterThanOrEqual(4.5);
    expect(tokens.adjusted).toBe(false);
  });

  it('moves lightness when the chosen colour would fail contrast', () => {
    const tokens = deriveAccent('#FFFF00', 'light');
    expect(tokens).not.toBeNull();
    expect(tokens?.adjusted).toBe(true);
  });
});

describe('applyAccent', () => {
  it('leaves the palettes in charge for the brand accent', () => {
    const root = document.documentElement;
    root.style.setProperty('--mnelys-color-accent', '#123456');
    root.dataset['accent'] = 'custom';

    applyAccent(BRAND_ACCENT, 'light');

    expect(root.style.getPropertyValue('--mnelys-color-accent')).toBe('');
    expect(root).not.toHaveAttribute('data-accent');
  });

  it('writes derived tokens for a custom accent', () => {
    const root = document.documentElement;

    applyAccent('#1565C0', 'dark');

    expect(root).toHaveAttribute('data-accent', 'custom');
    expect(root.style.getPropertyValue('--mnelys-color-accent')).not.toBe('');
    expect(root.style.getPropertyValue('--mnelys-color-on-accent')).not.toBe('');
  });

  it('falls back to the palette when the stored accent is invalid', () => {
    const root = document.documentElement;

    applyAccent('not-a-colour', 'light');

    expect(root).not.toHaveAttribute('data-accent');
    expect(root.style.getPropertyValue('--mnelys-color-accent')).toBe('');
  });
});
