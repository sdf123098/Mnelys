# Design specification baseline

Status: Accepted for M1
Last updated: 2026-10-26

Mnelys uses a stable, opaque, content-first desktop surface. Windows Mica, Acrylic, desktop sampling, transparent root windows, and undocumented DWM blur APIs are prohibited.

## Brand identity

- International product name: `Mnelys`
- Simplified Chinese product name: `忆涟`
- Canonical source artwork: [`assets/branding/mnelys-icon-source.jpg`](../assets/branding/mnelys-icon-source.jpg)
- The source artwork must remain unchanged. Platform icon generation uses derived assets and must not overwrite the canonical source.
- Derived icons must preserve the recognizable face, hair silhouette, eye mark, flower, and pink/blue palette. Cropping must not remove the face or primary silhouette.
- Platform-specific padding, corner masks, color profiles, and output formats are handled during the packaging PoC.

## Window baseline

- Initial size: 1280×800
- Minimum size: 900×600
- Root background: opaque
- Compact layout: 900–1199 px
- Standard layout: 1200–1599 px
- Expanded developer layout: 1600 px and above

## Shell layout

- Navigation rail: 220 px standard, 64 px icon-only compact rail below 1200 px. Each rail link keeps an accessible name and a tooltip in compact mode.
- Top bar: one visual page title, theme control, task-center control, account entry.
- Status bar: persistent core/runtime health with a non-color indicator.
- Task center: modal drawer from the right, with a focus trap, Escape to close, and focus restored to the opener.
- Content column: maximum 76 rem, page padding 24 px; cards use a 18 rem minimum grid track so text does not collapse into narrow columns.
- Below 900 px is not a supported working layout; the native window enforces the minimum size.

## Themes

- Preferences are `system`, `light`, and `dark`, persisted per install.
- An explicit choice sets `data-theme` on the root element; `system` removes the attribute so the platform preference applies.
- The theme is applied before the first React render to avoid a flash.
- Every semantic token must resolve in both themes. Text, borders, focus rings, and status colors are validated in both.
- System background materials and CSS `backdrop-filter` remain prohibited in every theme.

## Iconography

- Interface icons come from Lucide (ISC) through the single registry in `src/components/ui/Icon.tsx`. See [ICONS.md](ICONS.md).
- Icons inherit `currentColor`, default to 18 px at 1.75 stroke width, and are decorative unless given a label.
- The application icon is the canonical Mnelys brand artwork and is not part of the interface icon set.

## Semantic tokens

```text
Color.Window.Background
Color.Surface.Default
Color.Surface.Elevated
Color.Border.Subtle
Color.Text.Primary
Color.Text.Secondary
Color.Accent.Primary
Color.Status.Success / Warning / Error

Radius.Small / Medium / Large
Space.1 / 2 / 3 / 4 / 6 / 8
Motion.Fast / Normal / Emphasis
```

Pages and controls must not introduce isolated brand colors, radii, spacing values, or animation durations when a semantic token applies.

## Accessibility baseline

- Every action is keyboard reachable and has a visible focus indicator.
- Status is not communicated by color alone.
- Controls provide automation names and help text where their visible label is insufficient.
- The shell must be usable at 200% DPI and honor reduced-motion preferences.
- Simplified Chinese and English resources remain separate from the first shell implementation.
