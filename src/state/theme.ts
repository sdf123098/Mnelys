import { useSyncExternalStore } from 'react';

import { BRAND_ACCENT, type ThemePreference } from './preferences-store';

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

export type ResolvedTheme = 'light' | 'dark';

const DARK_QUERY = '(prefers-color-scheme: dark)';

const LIGHT_SURFACE = '#FFFFFF';
const DARK_SURFACE = '#1C1F26';
const LIGHT_ON_ACCENT = '#FFFFFF';
const DARK_ON_ACCENT = '#101216';

/** The pink behind the brand sentinel, exposed so previews match the token default. */
export const BRAND_ACCENT_HEX = '#C2185B';

export type AccentPresetId =
  'brand' | 'pink' | 'red' | 'orange' | 'amber' | 'green' | 'teal' | 'blue' | 'indigo' | 'violet';

export interface AccentPreset {
  readonly id: AccentPresetId;
  /** `null` means the palette default, not a fixed colour. */
  readonly hex: string | null;
}

export const ACCENT_PRESETS: readonly AccentPreset[] = [
  { id: 'brand', hex: null },
  { id: 'pink', hex: '#D81B60' },
  { id: 'red', hex: '#C62828' },
  { id: 'orange', hex: '#E65100' },
  { id: 'amber', hex: '#B26A00' },
  { id: 'green', hex: '#2E7D32' },
  { id: 'teal', hex: '#00796B' },
  { id: 'blue', hex: '#1565C0' },
  { id: 'indigo', hex: '#3949AB' },
  { id: 'violet', hex: '#6A1B9A' },
];

const ACCENT_PROPERTIES = [
  '--mnelys-color-accent',
  '--mnelys-color-accent-hover',
  '--mnelys-color-accent-soft',
  '--mnelys-color-focus-ring',
  '--mnelys-color-on-accent',
] as const;

interface Rgb {
  readonly r: number;
  readonly g: number;
  readonly b: number;
}

interface Hsl {
  readonly h: number;
  readonly s: number;
  readonly l: number;
}

export interface AccentTokens {
  readonly accent: string;
  readonly hover: string;
  readonly soft: string;
  readonly focusRing: string;
  readonly onAccent: string;
  /** True when the input lightness had to move to keep text readable. */
  readonly adjusted: boolean;
}

export function resolveTheme(theme: ThemePreference, prefersDark: boolean): ResolvedTheme {
  if (theme === 'system') {
    return prefersDark ? 'dark' : 'light';
  }
  return theme;
}

export function parseHexColor(input: string): Rgb | null {
  const value = input.trim().replace(/^#/, '');
  if (!/^(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/.test(value)) {
    return null;
  }
  const expanded =
    value.length === 3
      ? value
          .split('')
          .map((part) => part + part)
          .join('')
      : value;
  return {
    r: Number.parseInt(expanded.slice(0, 2), 16),
    g: Number.parseInt(expanded.slice(2, 4), 16),
    b: Number.parseInt(expanded.slice(4, 6), 16),
  };
}

export function toHex({ r, g, b }: Rgb): string {
  const part = (value: number): string =>
    Math.round(Math.min(255, Math.max(0, value)))
      .toString(16)
      .padStart(2, '0');
  return `#${part(r)}${part(g)}${part(b)}`.toUpperCase();
}

function rgbToHsl({ r, g, b }: Rgb): Hsl {
  const red = r / 255;
  const green = g / 255;
  const blue = b / 255;
  const max = Math.max(red, green, blue);
  const min = Math.min(red, green, blue);
  const lightness = (max + min) / 2;
  const delta = max - min;

  if (delta === 0) {
    return { h: 0, s: 0, l: lightness };
  }

  const saturation =
    lightness > 0.5 ? delta / (2 - max - min) : delta / Math.max(max + min, Number.EPSILON);

  let hue: number;
  if (max === red) {
    hue = (green - blue) / delta + (green < blue ? 6 : 0);
  } else if (max === green) {
    hue = (blue - red) / delta + 2;
  } else {
    hue = (red - green) / delta + 4;
  }

  return { h: hue * 60, s: saturation, l: lightness };
}

function hslToRgb({ h, s, l }: Hsl): Rgb {
  const hue = ((h % 360) + 360) % 360;
  const chroma = (1 - Math.abs(2 * l - 1)) * s;
  const section = hue / 60;
  const second = chroma * (1 - Math.abs((section % 2) - 1));
  const offset = l - chroma / 2;

  let rgb: readonly [number, number, number];
  if (section < 1) {
    rgb = [chroma, second, 0];
  } else if (section < 2) {
    rgb = [second, chroma, 0];
  } else if (section < 3) {
    rgb = [0, chroma, second];
  } else if (section < 4) {
    rgb = [0, second, chroma];
  } else if (section < 5) {
    rgb = [second, 0, chroma];
  } else {
    rgb = [chroma, 0, second];
  }

  return {
    r: Math.round((rgb[0] + offset) * 255),
    g: Math.round((rgb[1] + offset) * 255),
    b: Math.round((rgb[2] + offset) * 255),
  };
}

function mix(first: Rgb, second: Rgb, weight: number): Rgb {
  const share = Math.min(1, Math.max(0, weight));
  return {
    r: Math.round(first.r * share + second.r * (1 - share)),
    g: Math.round(first.g * share + second.g * (1 - share)),
    b: Math.round(first.b * share + second.b * (1 - share)),
  };
}

function channelToLinear(value: number): number {
  const channel = value / 255;
  return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
}

export function relativeLuminance({ r, g, b }: Rgb): number {
  return 0.2126 * channelToLinear(r) + 0.7152 * channelToLinear(g) + 0.0722 * channelToLinear(b);
}

export function contrastRatio(first: Rgb, second: Rgb): number {
  const a = relativeLuminance(first);
  const b = relativeLuminance(second);
  const lighter = Math.max(a, b);
  const darker = Math.min(a, b);
  return (lighter + 0.05) / (darker + 0.05);
}

function tokensFor(hsl: Hsl, theme: ResolvedTheme): AccentTokens {
  const accent = hslToRgb(hsl);
  const surface = parseHexColor(theme === 'light' ? LIGHT_SURFACE : DARK_SURFACE);
  const onAccent = parseHexColor(theme === 'light' ? LIGHT_ON_ACCENT : DARK_ON_ACCENT);
  if (surface === null || onAccent === null) {
    throw new Error('accent surfaces must be valid colours');
  }

  const soft = mix(accent, surface, theme === 'light' ? 0.12 : 0.18);
  const hoverLightness = Math.min(1, Math.max(0, hsl.l + (theme === 'light' ? -0.08 : 0.08)));

  return {
    accent: toHex(accent),
    hover: toHex(hslToRgb({ ...hsl, l: hoverLightness })),
    soft: toHex(soft),
    focusRing: toHex(accent),
    onAccent: toHex(onAccent),
    adjusted: false,
  };
}

function keepsContrast(hsl: Hsl, theme: ResolvedTheme): boolean {
  const tokens = tokensFor(hsl, theme);
  const accent = parseHexColor(tokens.accent);
  const soft = parseHexColor(tokens.soft);
  const surface = parseHexColor(theme === 'light' ? LIGHT_SURFACE : DARK_SURFACE);
  const onAccent = parseHexColor(tokens.onAccent);
  if (accent === null || soft === null || surface === null || onAccent === null) {
    return false;
  }
  return (
    contrastRatio(accent, onAccent) >= 4.5 &&
    contrastRatio(accent, soft) >= 4.5 &&
    contrastRatio(accent, surface) >= 4.5
  );
}

/**
 * Derives every accent token from one colour and moves its lightness only as far as
 * needed for AA text contrast. The hue and saturation stay exactly as chosen.
 */
export function deriveAccent(input: string, theme: ResolvedTheme): AccentTokens | null {
  const rgb = parseHexColor(input);
  if (rgb === null) {
    return null;
  }

  const original = rgbToHsl(rgb);
  if (keepsContrast(original, theme)) {
    return tokensFor(original, theme);
  }

  let best: Hsl | null = null;
  let bestDistance = Number.POSITIVE_INFINITY;
  for (let lightness = 0.02; lightness <= 0.98; lightness += 0.01) {
    const candidate: Hsl = { ...original, l: lightness };
    if (!keepsContrast(candidate, theme)) {
      continue;
    }
    const distance = Math.abs(lightness - original.l);
    if (distance < bestDistance) {
      best = candidate;
      bestDistance = distance;
    }
  }

  if (best === null) {
    return null;
  }
  return { ...tokensFor(best, theme), adjusted: bestDistance > 0.01 };
}

/**
 * Applies the accent tokens. The brand choice removes every override so the CSS
 * palettes decide; a custom colour writes the derived tokens for the active theme.
 */
export function applyAccent(
  accent: string,
  theme: ResolvedTheme,
  root: HTMLElement = document.documentElement,
): void {
  for (const property of ACCENT_PROPERTIES) {
    root.style.removeProperty(property);
  }

  if (accent === BRAND_ACCENT) {
    delete root.dataset['accent'];
    return;
  }

  const tokens = deriveAccent(accent, theme);
  if (tokens === null) {
    delete root.dataset['accent'];
    return;
  }

  root.style.setProperty('--mnelys-color-accent', tokens.accent);
  root.style.setProperty('--mnelys-color-accent-hover', tokens.hover);
  root.style.setProperty('--mnelys-color-accent-soft', tokens.soft);
  root.style.setProperty('--mnelys-color-focus-ring', tokens.focusRing);
  root.style.setProperty('--mnelys-color-on-accent', tokens.onAccent);
  root.dataset['accent'] = 'custom';
}

function subscribeToSystemTheme(onChange: () => void): () => void {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return () => undefined;
  }
  const media = window.matchMedia(DARK_QUERY);
  media.addEventListener('change', onChange);
  return () => {
    media.removeEventListener('change', onChange);
  };
}

function systemPrefersDark(): boolean {
  return typeof window !== 'undefined' && typeof window.matchMedia === 'function'
    ? window.matchMedia(DARK_QUERY).matches
    : false;
}

/** Resolves a stored preference against the live OS setting, before React mounts. */
export function resolveStoredTheme(theme: ThemePreference): ResolvedTheme {
  return resolveTheme(theme, systemPrefersDark());
}

export function useSystemPrefersDark(): boolean {
  return useSyncExternalStore(subscribeToSystemTheme, systemPrefersDark, () => false);
}

export function useResolvedTheme(theme: ThemePreference): ResolvedTheme {
  return resolveTheme(theme, useSystemPrefersDark());
}
