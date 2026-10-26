# ADR 0007: Accent colour and localization policy

- Status: Accepted
- Date: 2026-10-26
- Decision owners: Mnelys maintainers
- Supersedes: the three-language localization scope in [ADR 0006](0006-frontend-framework-and-application-stack.md); the stack decisions in ADR 0006 remain in force

## Context

[ADR 0006](0006-frontend-framework-and-application-stack.md) fixed i18next and three catalogs (`zh-Hans`, `en`, `ja`), and the M1 shell inherited a single fixed accent colour from the token set. Two product constraints made both decisions too narrow:

- The product owner requires **pink as the default accent** and a **user-selectable accent colour**, not a fixed brand blue/pink pair that contributors must edit in CSS.
- The launcher must support **as many languages as practical**. A three-catalog set turns every new language into an application change, which guarantees the set stays small.

At the same time, the project has one non-negotiable naming rule: the UI shows a **single localized product name**, never a bilingual pair. The earlier shell displayed "忆涟 · Mnelys", which mixed languages in the title bar and window title.

## Decision

### Accent colour

- The default accent is the brand pink `#C2185B`. It is a semantic token, not a literal in components.
- Settings offers a fixed preset list plus a validated custom hex colour. The choice is stored per install with the other preferences and applied before the first React render.
- A custom colour is expanded into five tokens — accent, hover, soft surface, focus ring, and on-accent text — by one derivation routine. If the colour cannot reach WCAG AA contrast against the accent surface, its **lightness alone** is adjusted as little as necessary; hue and saturation are preserved, and the UI states that the colour was adjusted.
- The brand preset removes the overrides so the CSS palettes decide. No component reads a literal colour.

### Localization

- Catalogs are **discovered from `src/i18n/locales/*.json`** at build time. Adding a language is adding one JSON file; no registry, route table, or type union needs editing.
- The English catalog is the **canonical key set**. Message keys are typed from it, and a parity test asserts that every bundled catalog declares exactly the same keys. A language can therefore be missing no message and can invent no message.
- Starting set: `en`, `zh-Hans`, `zh-Hant`, `ja`, `ko`, `fr`, `de`, `es`, `pt-BR`, `ru`. The picker shows the four core languages first and then every additional catalog by code; each entry is labelled with its endonym.
- Unknown browser or stored tags fall back in this order: exact tag, Chinese script/region handling, same base language, English.
- Right-to-left languages are recognised by a base-language set so a future Arabic, Hebrew, Persian, or Urdu catalog flips the document direction without code changes. None of those catalogs ship yet.
- The product name is **single-language**: `忆涟` in Simplified Chinese, `憶漣` in Traditional Chinese, and `Mnelys` everywhere else, including Japanese. The window title follows the active catalog. No catalog may append a second name separated by a slash or dot.

## Consequences

- The localization test suite grows with each catalog, by design. A new language ships only after it has the full key set; a partially translated file is a build failure rather than a source of raw keys at runtime.
- Untranslated future keys must be added to every catalog in the same change. Keeping unused-but-legacy keys is permitted so parity stays exact, but no key may exist in only one catalog.
- The accent derivation is pure TypeScript with unit tests, so it runs identically in all three webviews and cannot drift into per-component colour values.
- Language coverage is still bounded by translation quality, not by architecture. The endonym table already includes more languages than catalogs ship; adding one is a translation task, not an engineering task.
- The Japanese UI showing "Mnelys" is an explicit product decision, not a missing translation.
