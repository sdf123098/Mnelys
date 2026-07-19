# Design specification baseline

Status: Accepted for M0
Last updated: 2026-07-19

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
