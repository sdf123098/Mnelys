# Interface iconography

Status: Accepted for M1
Last updated: 2026-10-26

## Policy

Every interface pictogram in the React frontend comes from an open-source icon set. Mnelys does not draw proprietary UI glyphs, bundle icon fonts, or load icons from a runtime CDN.

| Item                       | Value                                                                         |
| -------------------------- | ----------------------------------------------------------------------------- |
| Icon set                   | [Lucide](https://lucide.dev/)                                                 |
| Distribution               | `lucide-react`, version pinned by `pnpm-lock.yaml`                            |
| License                    | ISC                                                                           |
| Delivery                   | Inline React SVG components bundled by Vite                                   |
| Single source of truth     | `src/components/ui/Icon.tsx`                                                  |
| Application icon exception | The Mnelys brand artwork described in `assets/branding/README.md`, not Lucide |

The application icon is brand identity, not interface iconography. It is never replaced by an icon-set glyph, and the navigation rail renders the icon image itself (`public/app-icon.png`) rather than a letter from the product name.

## Usage rules

1. Pages and components import `Icon` and pass a semantic name. Importing `lucide-react` anywhere else is a review failure.
2. The registry maps stable product names (`start`, `instances`, `warning`, …) to Lucide components. Changing an underlying glyph must not change the name pages use.
3. Decorative icons render with `aria-hidden="true"`. An icon that carries meaning on its own must receive a localized `label`, which produces `role="img"` and an accessible name.
4. Icon-only buttons use `IconButton`, whose required `label` becomes both the accessible name and the tooltip. A bare `Icon` is never the only content of a button.
5. Icons inherit `currentColor`. Color comes from semantic tokens; components must not hardcode accent or status hex values.
6. Default rendering is 18 px at 1.75 stroke width. Compact chrome (status bar, inline badges) may reduce size; do not scale icons below 14 px.
7. New icons are added to the registry with a real product use case. Unused glyphs stay out so the bundle and the vocabulary remain small.

## License and release obligations

Lucide is ISC-licensed. The license permits use and redistribution provided the copyright notice and permission notice are retained. The dependency license ships with the package and is recorded by the package manager.

Before the first public release, `THIRD_PARTY_NOTICES` must include Lucide's ISC notice alongside the Rust crates and other distributed dependencies. This file records the policy; it is not a substitute for the release notice.

## Audit

- Registry: `src/components/ui/Icon.tsx`
- Allowed import site: `src/components/ui/Icon.tsx` only
- Brand asset rules: `assets/branding/README.md`
- Release notices and SBOM: `docs/SECURITY.md`
