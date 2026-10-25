import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { BRAND_ACCENT, usePreferences } from '@/state/preferences-store';
import {
  ACCENT_PRESETS,
  BRAND_ACCENT_HEX,
  deriveAccent,
  parseHexColor,
  toHex,
  useResolvedTheme,
  type AccentPresetId,
} from '@/state/theme';

import { Button } from './Button';
import styles from './AccentPicker.module.css';

const PRESET_LABELS = {
  brand: 'settings.accentBrand',
  pink: 'settings.accentPink',
  red: 'settings.accentRed',
  orange: 'settings.accentOrange',
  amber: 'settings.accentAmber',
  green: 'settings.accentGreen',
  teal: 'settings.accentTeal',
  blue: 'settings.accentBlue',
  indigo: 'settings.accentIndigo',
  violet: 'settings.accentViolet',
} as const satisfies Record<AccentPresetId, string>;

function normalizeHex(input: string): string | null {
  const rgb = parseHexColor(input);
  return rgb === null ? null : toHex(rgb);
}

/**
 * Accent selection: brand default, a small preset row, and one free colour.
 *
 * The stored value is either the brand sentinel or a hex colour. Nothing here writes
 * CSS tokens — the provider derives them for the active theme, which is also what keeps
 * a light-theme choice readable after the user switches to dark.
 */
export function AccentPicker() {
  const { t } = useTranslation();
  const accent = usePreferences((state) => state.accent);
  const setAccent = usePreferences((state) => state.setAccent);
  const theme = usePreferences((state) => state.theme);
  const resolvedTheme = useResolvedTheme(theme);

  const brandSelected = accent === BRAND_ACCENT;
  const storedText = brandSelected ? BRAND_ACCENT_HEX : accent;
  const [draft, setDraft] = useState({ accent, text: storedText });
  const [invalid, setInvalid] = useState(false);
  // A preset change updates the stored accent; the text field follows unless the user
  // is actively editing a value that was already based on the new accent.
  const text = draft.accent === accent ? draft.text : storedText;

  const tokens = brandSelected ? null : deriveAccent(accent, resolvedTheme);

  function commit(value: string): void {
    const hex = normalizeHex(value);
    if (hex === null) {
      setInvalid(true);
      return;
    }
    setDraft({ accent: hex, text: hex });
    setInvalid(false);
    setAccent(hex);
  }

  function choose(presetHex: string | null): void {
    setInvalid(false);
    setAccent(presetHex ?? BRAND_ACCENT);
  }

  return (
    <fieldset className={styles.root}>
      <legend className={styles.legend}>{t('settings.accent')}</legend>

      <div className={styles.presets} role="group" aria-label={t('settings.accentPresets')}>
        {ACCENT_PRESETS.map((preset) => {
          const isBrand = preset.hex === null;
          const hex = preset.hex ?? BRAND_ACCENT_HEX;
          const selected = isBrand
            ? brandSelected
            : !brandSelected && accent.toLowerCase() === hex.toLowerCase();
          const label = t(PRESET_LABELS[preset.id]);
          return (
            <button
              key={preset.id}
              type="button"
              className={selected ? `${styles.swatch} ${styles.swatchSelected}` : styles.swatch}
              style={{ backgroundColor: hex }}
              title={label}
              aria-label={label}
              aria-pressed={selected}
              onClick={() => {
                choose(preset.hex);
              }}
            />
          );
        })}
      </div>

      <div className={styles.custom}>
        <input
          type="color"
          className={styles.colorInput}
          value={normalizeHex(text) ?? storedText}
          aria-label={t('settings.accentCustom')}
          onChange={(event) => {
            commit(event.target.value);
          }}
        />
        <input
          type="text"
          className={styles.hexInput}
          value={text}
          spellCheck={false}
          autoComplete="off"
          aria-label={t('settings.accentHex')}
          aria-invalid={invalid}
          onChange={(event) => {
            setDraft({ accent, text: event.target.value });
            setInvalid(false);
          }}
          onBlur={() => {
            commit(text);
          }}
          onKeyDown={(event) => {
            if (event.key === 'Enter') {
              commit(text);
            } else if (event.key === 'Escape') {
              setDraft({ accent, text: storedText });
              setInvalid(false);
            }
          }}
        />
        <Button
          variant="secondary"
          size="sm"
          onClick={() => {
            choose(null);
          }}
          disabled={brandSelected}
        >
          {t('settings.accentReset')}
        </Button>
      </div>

      {invalid ? (
        <p className={styles.error} role="alert">
          {t('settings.accentInvalid')}
        </p>
      ) : null}
      {tokens?.adjusted === true ? (
        <p className={styles.hint}>{t('settings.accentAdjusted')}</p>
      ) : null}
    </fieldset>
  );
}
