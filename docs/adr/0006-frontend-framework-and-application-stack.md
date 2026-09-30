# ADR 0006: Frontend framework and application stack — React and TypeScript

- Status: Accepted
- Date: 2026-09-30
- Decision owners: Mnelys maintainers
- Depends on: [ADR 0005](0005-runtime-and-ui-baseline-tauri.md)
- Completes: the decision ADR 0005 deliberately left open

## Context

[ADR 0005](0005-runtime-and-ui-baseline-tauri.md) fixed Tauri 2.x as the desktop shell and Rust as the core language, but explicitly deferred the frontend framework, language, state management, routing, and design-token pipeline. It forbade committing production UI code until this decision existed.

Mnelys needs a large amount of ordinary application UI — instance and version lists, mod and resource-pack management, a download queue, a task center, log and crash-report viewers, account and Java management, a settings surface, and a developer workspace with Gradle output and debugger controls. Almost none of it is exotic; the dominant requirements are long-term maintainability by a very small team, fast iteration on list-and-form screens, a real component and tooling ecosystem, and a well-understood accessibility and internationalization story.

Two constraints from ADR 0005 shape the choice:

- The frontend runs in three different webviews (WebView2, WKWebView, WebKitGTK). The stack must degrade predictably and must not depend on a browser feature that one platform webview lacks.
- The frontend never holds credentials and only reaches native capability through declared Tauri commands. The stack should make that boundary easy to enforce mechanically rather than by discipline alone.

The product ships in Chinese (Simplified), English, and Japanese — the same three languages the repository READMEs already use.

## Decision

- **Framework**: React 19 with **TypeScript** in `strict` mode. `any` is a lint error in application code; use `unknown` plus narrowing at boundaries.
- **Build tool**: **Vite**. It is the tool Tauri's own frontend integration targets, it keeps the dev server fast with a large dependency graph, and it produces a static bundle that Tauri can embed.
- **Package manager**: **pnpm**, pinned in the `packageManager` field and driven through Corepack, with the lockfile committed. A single lockfile is the only supported install path.
- **Routing**: **React Router** in declarative mode, with every route declared in one typed route table. Deep links must survive a restart, so the route table — not component state — is the source of truth for "where the user is".
- **Rust-backed data**: **TanStack Query** owns every value that originates in the Rust core or a network call. Query keys are declared in one module. Components never cache core state themselves; the Rust core is the single source of truth, and the frontend cache is explicitly disposable.
- **Ephemeral UI state**: **Zustand**, limited to state that has no core-side owner — panel layout, current selection, wizard step, transient filters. No global store may duplicate core-owned data.
- **Styling**: **CSS Modules** plus a **design-token layer of CSS custom properties**. No runtime CSS-in-JS, so there is no style-recomputation cost on large lists and no extra runtime shipped into an already-sized webview. Themes are implemented by swapping a token set on the root element; components read tokens only, never literal colors.
- **Window composition**: the token layer and all component styles must resolve to opaque surfaces. `backdrop-filter`, transparent window backgrounds, and any effect that samples the desktop are prohibited here exactly as ADR 0005 prohibits them at the window level.
- **Internationalization**: **i18next** with `react-i18next`, catalogs for `zh-Hans`, `en`, and `ja`. Message keys are typed from the `en` catalog so a missing or misspelled key fails the type check rather than rendering a raw key at runtime.
- **Testing**: **Vitest** with **React Testing Library** for units and components, and **WebdriverIO** with `tauri-driver` for end-to-end runs against a real packaged build on Windows and Linux. Component tests run against a mocked Tauri transport and never invoke real commands.
- **Lint and format**: **ESLint** (flat config) with the React and TypeScript rules, plus **Prettier**. Formatting is checked in CI, not debated in review.

## The Rust/frontend boundary

Two rules make ADR 0005's isolation claim verifiable rather than aspirational:

1. **Exactly one Tauri client module.** All `invoke` calls live in a single typed client. Components and hooks import named functions from it and never call `invoke` directly. An ESLint restriction forbids importing Tauri's `invoke` outside that module.
2. **The command surface is a typed contract.** Command names, request payloads, and response types are declared once and mirrored on both sides, with a contract test that fails when the Rust command set and the TypeScript client diverge. Every payload carries a schema version, consistent with the persistence rule in [ARCHITECTURE](../ARCHITECTURE.md).

Errors crossing the boundary are typed results, not thrown strings: a failed command returns a structured error with a stable code that the UI maps to a localized message.

## Alternatives considered

- **Vue 3 + TypeScript** — the strongest alternative. Excellent documentation in Chinese and a mature ecosystem, and it would have been a defensible choice. Rejected because the deciding factor was the depth of third-party component and virtualized-list libraries for dense data screens, where React's ecosystem is still materially ahead.
- **Svelte / SvelteKit** — smallest runtime and the best fit for Tauri's "thin shell" philosophy. Rejected because the component ecosystem is markedly smaller, which matters more than bundle size for a product made mostly of lists, tables, and forms, and because long-term maintenance by a small team favors the most widely known option.
- **SolidJS** — the best raw DOM performance of the four. Rejected on ecosystem size and on the smallest hiring and community-support surface of any candidate.
- **No framework / server-driven UI** — rejected. The product needs rich local interaction, offline behavior, and a task center that updates from core-side events; a document-oriented model fights all three.
- **CSS-in-JS at runtime (styled-components, Emotion)** — rejected for the runtime cost on large lists and for the extra bundle weight in a webview we do not control.
- **Redux Toolkit** — rejected. The split between Rust-owned data (TanStack Query) and genuinely ephemeral UI state (Zustand) covers the requirements without a global store, and a global store would tempt contributors to duplicate core-owned state in the frontend.

## Consequences

- The frontend and the Rust core are two toolchains in one repository. CI must install and cache both, and a contributor needs a Rust toolchain plus a Node.js toolchain to run the app end to end.
- Webview differences remain a real risk. React and the chosen libraries mask most of it, but the design-token layer, focus behavior, font rendering, and IME behavior for Chinese and Japanese input must be verified in all three webviews before a release is promoted.
- Typed message keys and a strict `any` ban add friction at the edges and remove a whole class of runtime defects; the boundary contract test must be kept passing rather than treated as advisory.
- Accessibility becomes a first-class, testable requirement: semantic markup, keyboard navigation for every list and dialog, and reduced-motion support in the token layer.
- Bundle size is now a tracked number. Because the app is shipped inside the Tauri binary, the frontend build output is part of the artifact and must be watched, not left to grow unnoticed.
- Choosing React over the alternatives was close between React and Vue. If the component-ecosystem advantage does not materialize in practice, the reversal cost is a frontend rewrite, not a core rewrite — the Rust core and the typed command boundary are framework-independent by design, which is the main reason that boundary was defined before the framework was chosen.
