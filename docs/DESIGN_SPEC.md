# Design specification baseline

Status: Accepted for M1
Last updated: 2026-10-26

Mnelys uses a stable, opaque, content-first desktop surface. Windows Mica, Acrylic, desktop sampling, transparent root windows, and undocumented DWM blur APIs are prohibited.

## Brand identity

- International product name: `Mnelys`
- Simplified Chinese product name: `忆涟`
- Traditional Chinese product name: `憶漣`
- The UI shows exactly one localized name. Japanese and every other catalog use `Mnelys`; no bilingual pair, separator, or second name is permitted.
- Default accent: brand pink `#C2185B`.
- Canonical source artwork: [`assets/branding/mnelys-icon-source.jpg`](../assets/branding/mnelys-icon-source.jpg)
- The source artwork must remain unchanged. Platform icon generation uses derived assets and must not overwrite the canonical source.
- Derived icons must preserve the recognizable face, hair silhouette, eye mark, flower, and pink/blue palette. Cropping must not remove the face or primary silhouette.
- Platform-specific padding, corner masks, color profiles, and output formats are handled during the packaging PoC.

## Window baseline

- Initial size: 1280×800
- Minimum size: 900×600
- Root background: opaque
- Title bar: 2 rem drawn bar. Windows and Linux draw minimize, maximize/restore, and close controls; macOS keeps the native traffic lights and the bar reserves their space without drawing controls. The bar is a drag region and double-click maximizes on Windows and Linux.
- The window title follows the active catalog and always carries the same single product name as the UI.
- Compact layout: 900–1199 px
- Standard layout: 1200–1599 px
- Expanded developer layout: 1600 px and above

## Shell layout

- Navigation rail: 220 px standard, 64 px icon-only compact rail below 1200 px. Each rail link keeps an accessible name and a tooltip in compact mode.
- Primary pages: Start, Instances, Downloads, Content (the resource hub), Multiplayer, Settings. Accounts and the task center are reached from the top bar; the developer section appears only in developer mode.
- Top bar: one visual page title, theme control, task-center control, account entry.
- Status bar: persistent core/runtime health with a non-color indicator.
- Task center: modal drawer from the right, with a focus trap, Escape to close, and focus restored to the opener.
- Content column: maximum 76 rem, page padding 24 px; cards use a 18 rem minimum grid track so text does not collapse into narrow columns.
- Below 900 px is not a supported working layout; the native window enforces the minimum size.

## Themes and accent

- Preferences are `system`, `light`, and `dark`, persisted per install.
- An explicit choice sets `data-theme` on the root element; `system` removes the attribute so the platform preference applies.
- The theme is applied before the first React render to avoid a flash.
- The accent is a semantic token. Presets cover the brand pink plus eight additional hues; a validated custom hex is also accepted.
- A custom accent expands into accent, hover, soft surface, focus ring, and on-accent text. If WCAG AA contrast cannot be met, only lightness is adjusted, as little as possible; hue and saturation stay fixed, and the settings screen states that the colour was adjusted.
- The brand preset removes per-install overrides so the CSS palettes decide.
- Every semantic token must resolve in both themes. Text, borders, focus rings, and status colors are validated in both.
- System background materials and CSS `backdrop-filter` remain prohibited in every theme.

## Iconography

- Interface icons come from Lucide (ISC) through the single registry in `src/components/ui/Icon.tsx`. See [ICONS.md](ICONS.md).
- Icons inherit `currentColor`, default to 18 px at 1.75 stroke width, and are decorative unless given a label.
- The application icon is the canonical Mnelys brand artwork and is not part of the interface icon set. The rail's brand mark renders that same PNG (`public/app-icon.png`, the 128 px derivative also used as the document favicon); it is never a letter taken from the product name.

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
- Catalogs are bundled for `en`, `zh-Hans`, `zh-Hant`, `ja`, `ko`, `fr`, `de`, `es`, `pt-BR`, and `ru`; every catalog must declare the same keys as `en`.
- The picker lists core languages first and labels each entry with its endonym. Right-to-left base languages are recognized by the shell so a future catalog flips document direction without layout changes.
- Text must not be baked into fixed-width controls where translation would clip; compact navigation keeps accessible names when labels are visually hidden.
