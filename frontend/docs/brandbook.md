# Lensisku brand book

This document is the **human-readable contract** for product, design, QA, and engineering. The **machine-readable source of truth** for tokens, component classes, and theme switching lives in **`frontend/tailwind.config.js`** (Tailwind `theme.extend`, `addBase`, `addComponents`, and the `buttonUiThemeLayer`). When code and this file disagree, **fix the code**—do not fork one-off colors or button styles in individual `.vue` files.

---

## 1. Goals

| Goal                          | What it means in practice                                                                                                                                |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Recognizable product**      | Glossy **Aqua** controls and flat **toolbar** controls are both “Lensisku”; users can switch themes without breaking layouts.                            |
| **Predictable actions**       | Primary flows (save, search, destructive) map to **semantic** `ui-btn--*` names, not random Tailwind color utilities on `<button>`.                      |
| **Accessible by default**     | Visible focus states (`focus-visible` / rings), icon-only controls require **accessible names** (`aria-label` or visible `label`).                       |
| **Maintainable UI**           | New screens use **named patterns** from the Tailwind layer (`input-field`, `toolbar-panel`, `card-elevated`, …) and shared **`packages/ui`** primitives. |
| **No layout hacks for icons** | Icon + text controls use **`gap-*` on the parent**—never `mr-*` / `ml-*` on icons (breaks when labels are hidden or translated).                         |

---

## 2. Audience & tone

- **Primary users**: Learners and contributors around **Lojban**—dictionary search, definitions, collections, study tools, and community threads.
- **Visual tone**: **Clear, calm, tool-like**—white surfaces, blue as the primary interactive hue (`text-nav-link`, link headings), optional **Aqua** theme for a tactile, “desktop app” feel (glossy pills, GTK-inspired tabs).
- **Not**: Generic “AI dashboard” purple gradients, interchangeable system-gray cards, or unrelated display fonts in body copy.

---

## 3. Source of truth & file roles

| Asset                                   | Role                                                                                                                                                                                                                        |
| --------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`tailwind.config.js`**                | Brand tokens (`colors.nav-link`, `fontFamily.sans`), shared **component classes** (`.btn-base`, `.aqua-base`, `.input-field`, …), **semantic button mapping** (`ui-btn--*` → aqua/flat primitives via `data-button-theme`). |
| **`tailwind.flat-buttons.mjs`**         | **Flat** theme: `.btn-flat-surface` + semantic `btn-*` palettes (imported into `tailwind.config.js`).                                                                                                                       |
| **`tailwind.aqua-buttons.mjs`**         | **Aqua** theme: `.aqua-base` / `.aqua-base-secondary`, semantic `btn-aqua-*`, segment geometry helper, toggle-off shadow.                                                                                                   |
| **`packages/ui/*.vue`**                 | **Reusable, mostly stateless** building blocks (`Button`, `IconButton`, `Card`, `Dropdown`, …). They must **not** import from `src/`.                                                                                       |
| **`src/components/**`\*\*               | Domain-aware UI (search, definitions, filters, Lingo study, …). Prefer composing `packages/ui` + Tailwind named classes.                                                                                                    |
| **`src/composables/useButtonTheme.ts`** | Persists `aqua` \| `flat` in `localStorage`, sets `document.documentElement.dataset.buttonTheme`.                                                                                                                           |
| **`index.html`**                        | Inline script sets `data-button-theme` before paint (avoids flash); global font loading.                                                                                                                                    |

---

## 4. Typography

- **UI font stack**: **`Open Sans`** (loaded globally), with `system-ui`, `Tahoma`, and `sans-serif` fallbacks (`theme.extend.fontFamily.sans`).
- **Base**: `html` uses `font-sans` and `antialiased` (via Tailwind base layer).
- **Glossy Aqua controls** (`.aqua-base`, `.aqua-base-secondary`) inherit the same stack via `font-sans`—do not duplicate ad-hoc `font-family` in components.
- **Monospace** (messages, code paths): **JetBrains Mono** where already used in specific pages—keep monospace scoped; do not set global body to monospace.

---

## 5. Color system (high level)

- **Navigation / links**: `text-nav-link` (`#007bff`) and **active** state via `.nav-link-active` (see `NavLink`, `navbar-item`). Mobile rows use `.mobile-nav-row` (hover/focus only—link color comes from NavLink).
- **Semantic blues** in surfaces: borders and hovers (`border-blue-*`, `hover:border-blue-*`) for **interactive lists** (`.surface-list-row`, `.message-thread-card`, …).
- **Status**: green success, red danger, amber/yellow warning—prefer existing **badge** and **btn** semantics over new hex values.
- **Selma'o / tags**: `.badge-definition-tag` (purple chip)—use for grammatical tags, not general emphasis.

---

## 6. Button system (two families)

### 6.1 Theme switch: Aqua vs Flat

- Stored as `lensisku.buttonTheme` in `localStorage`; mirrored on **`html[data-button-theme="aqua"]`** or **`flat`** (default **flat** if unset).
- **`index.html`** runs a small script before Vue boots so the first paint uses the correct primitive classes.

### 6.2 Primitives

| Family            | Base class                                                                           | Character                                                                                               |
| ----------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------- |
| **Glossy / Aqua** | `.aqua-base` (+ `btn-aqua-*` fills) — `tailwind.aqua-buttons.mjs`                    | 3D, high-gloss, strong shadow; **primary chrome** for “hero” actions when Aqua theme is on.             |
| **Flat**          | `.btn-base` → `.btn-flat-surface` (+ semantic `btn-*`) — `tailwind.flat-buttons.mjs` | 2D, bordered, subdued; default for **inline**, **toolbar**, and **list** actions when Flat theme is on. |

#### 6.2a Flat theme: layered model (machine source: `tailwind.config.js`)

Flat controls share one interaction recipe; **only palette + label color** change per role.

| Layer                | Class                                    | Responsibility                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| -------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Geometry**         | `.btn-base`                              | Pill shape, padding, border radius, disabled opacity, default outer shadow, `cursor-pointer`, press nudge (`active:scale-x`).                                                                                                                                                                                                                                                                                                                                                |
| **Surface + motion** | `.btn-flat-surface`                      | `@apply btn-base`. **Default** gradient on `::before` and **hover/active** gradient on `::after` (opacity crossfade ~220ms). Both layers use **`z-index: -1`** so they paint **behind** all in-flow label text (including unwrapped text nodes—no `z-index` on icons only). **Active** swaps `::after` to the darker `--bf-*` pair. Border on the root tracks hover/active. **Focus**: `ring-2` + `--bf-ring`. **`prefers-reduced-motion`**: shorten the opacity transition. |
| **Embossed neutral** | `.btn-empty`                             | **Not** `.btn-flat-surface`: **inset** emboss on the root; same **`::before` / `::after`** stacking (`z-index: -1`) for hover/active fills.                                                                                                                                                                                                                                                                                                                                  |
| **Role / hue**       | `btn-get`, `btn-update`, `btn-delete`, … | Set **text color** + `--bf-*` tokens. Most hues use a **shared generator** (`100/50` → `200/100` → `300/200` stops, borders `400`→`600`→`700`). **CTA** (`btn-insert`, `btn-reaction-active`) and **soft** rows (`btn-market`, `btn-action`, `btn-reaction`) set custom `--bf-*` explicitly.                                                                                                                                                                                 |

**Do not** use Tailwind’s `enabled:hover:*` on flat primitives for fills that must apply to **`<a>` / `RouterLink`**: `:enabled` does not match anchors. The flat layer uses **`:hover:not(:disabled)`** etc. on `.btn-flat-surface` so buttons and links behave the same.

### 6.3 Semantic API for components: `ui-btn--*`

Application code and **`Button.vue`** / **`IconButton.vue`** should use **semantic** classes whose names start with **`ui-btn--`** (or pass `variant="neutral"` etc., which resolves to `ui-btn--neutral`). Each **role** maps to **one flat primitive** (`btn-*`) and **one aqua primitive** (`btn-aqua-*`) in `buttonThemeClassMap` inside **`tailwind.config.js`** (authoritative list).

**Prefer canonical role names** in new code; **legacy synonyms** stay in the map so older templates keep working.

| Action role                   | Canonical class                                      | Typical use                                                                                    |
| ----------------------------- | ---------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| **Read / open / navigate**    | `ui-btn--read`                                       | Links, “open”, export/download, neutral forward. _Legacy:_ `ui-btn--get`.                      |
| **Edit / apply**              | `ui-btn--edit`                                       | Save profile, merge, apply recommendation. _Legacy:_ `ui-btn--update`.                         |
| **Create / add**              | `ui-btn--create`                                     | Submit new entity, add row. Often paired with **`ui-btn--primary`** (same mapping).            |
| **Delete / remove**           | `ui-btn--delete`                                     | Destructive removal. _Synonym:_ `ui-btn--remove` (same mapping).                               |
| **Dismiss / cancel**          | `ui-btn--cancel`                                     | Close modal, secondary “no”. _Synonym:_ `ui-btn--dismiss`.                                     |
| **Toolbar / filter chrome**   | `ui-btn--toolbar`                                    | Compact toggles, reset filters (`Button variant="toolbar"`). _Legacy:_ `ui-btn--aqua-default`. |
| **Neutral / low emphasis**    | `ui-btn--empty`                                      | Deselect, clipboard, embossed default.                                                         |
| **Neutral (tinted)**          | `ui-btn--neutral`, `neutral-muted`, `neutral-slate`  | Secondary pills, back links, muted actions.                                                    |
| **Pagination back / forward** | `ui-btn--back`, `ui-btn--forward`                    | Prev/next page. _Legacy:_ `previous`, `next`.                                                  |
| **Continue**                  | `ui-btn--continue`                                   | Resume flow (maps like read/get).                                                              |
| **Market / commerce**         | `ui-btn--market`                                     | Rose / market styling.                                                                         |
| **History / versions**        | `ui-btn--history`                                    | Version list, purple history.                                                                  |
| **Insert / inline add**       | `ui-btn--insert`                                     | Add to selection, blue insert.                                                                 |
| **Link / outbound**           | `ui-btn--link`                                       | Blue link-style.                                                                               |
| **Accent (activity)**         | `ui-btn--accent-purple`                              | “View activity” and similar.                                                                   |
| **Generic action (pink)**     | `ui-btn--action`                                     | Promotional / emphasis CTA.                                                                    |
| **Reply**                     | `ui-btn--reply`                                      | Thread reply.                                                                                  |
| **Reaction**                  | `ui-btn--reaction`, `reaction-active`                | Toggle reactions.                                                                              |
| **Auth**                      | `ui-btn--auth-login`, `auth-signup`                  | Login vs signup emphasis.                                                                      |
| **Status**                    | `ui-btn--success`, `error`, `warning`, `danger-rose` | Result / alert / destructive rose.                                                             |
| **Revert**                    | `ui-btn--revert`                                     | Undo to old revision (yellow).                                                                 |
| **Study grading**             | `study-correct`, `study-wrong`                       | Flashcard / Lingo result.                                                                      |
| **Warning shades**            | `warning`, `warning-orange`, `warning-yellow`        | Caution; yellow uses flat **`btn-revert`** vs amber **`btn-warning`**.                         |
| **Sort / palette**            | `sort-*`, `palette-*`                                | Rotating colors in lists (see map).                                                            |
| **Segmented group**           | `ui-btn--group-item`                                 | Inner segment (with `.btn-group` / `.btn-group-forced`).                                       |
| **Special**                   | `ui-btn--fab`, `toggle`                              | FAB and toggle have extra rules in the same Tailwind layer.                                    |

**Rules**

1. Prefer **`<Button variant="…">`** or **`<IconButton button-classes="…">`** over raw `class="btn-aqua-emerald"` in new code—keeps theme switching coherent.
2. **Account / profile toolbars**: pair **read** (`ui-btn--read`, e.g. change password) with **edit** (`ui-btn--edit`). Do not use **`ui-btn--warning-*`** for routine account actions—they read as caution chrome; **`warning-yellow`** in flat theme maps to **`btn-revert`** (yellow) to stay distinct from amber **`btn-warning`**.
3. **Flat theme**: major empty-state CTAs may still use **`ui-btn--*`**; do not mix random `bg-blue-600` full-width buttons unless using an existing pattern (e.g. `.btn-panel-primary` for specific panels).
4. **Icon-only**: must have **`aria-label`** (or visible text). `IconButton` derives `aria-label` from `ariaLabel` or `label`.

### 6.4 When Aqua vs Flat “wins”

- Users choose **Aqua** for nostalgia / gloss; **Flat** for calmer, flatter UI.
- QA should verify **both** themes on any change that touches `ui-btn--*` or `btn-aqua-*` / `btn-*` primitives.

---

## 7. Forms & inputs

- Text fields: **`.input-field`**, **`.textarea-field`** (inner shadow, blue focus ring).
- Tall toolbar triggers (filters): **`.dropdown-trigger`** (height aligned with `h-10` inputs).
- Password fields: **`.input-field-password-toggle`** for the eye toggle; pair with **`pr-10`** on the input.
- File picking: **`.file-input-label`** pattern for accessible focus rings.
- **Do not** recreate `rounded-full border border-gray-300 shadow-inner` ad hoc—use the classes above.

---

## 7b. Toggles & banners

- **Threaded / flat discussion toggle** (checkbox + track): **`toggle-switch-peer-track`** on the visual track; put **`peer sr-only`** on the `<input>` (see `DefinitionLinkDiscussion.vue`). Uses **`bg-nav-link`** when checked (not raw `bg-blue-600`).
- **Footer modal** iOS toggle: **`.toggle-switch`** + **`.toggle-switch-thumb`** (see `FooterComponent.vue`).
- **Anonymous progress banner** CTAs: **`.anon-banner-cta`** plus **`ui-btn--*`**; dismiss: **`.anon-banner-dismiss`**.

---

## 8. Layout & shells

| Class                                                       | Use                                  |
| ----------------------------------------------------------- | ------------------------------------ |
| `.app-header-bar`                                           | Sticky top chrome (logo, nav, auth). |
| `.page-loading-overlay`                                     | Full-screen loading veil.            |
| `.auth-page-shell` / `.auth-form-card` / `.auth-glass-card` | Login, signup, password flows.       |
| `.toolbar-panel` + `.toolbar-row`                           | Search + filter toolbars.            |
| `.page-section-title`                                       | In-page H1-style tool titles.        |
| `.empty-state-panel`                                        | Zero-state blocks.                   |

---

## 9. Cards & lists

- **`.card-base`**, **`.card-elevated`**, **`.card-compact`**: list/detail cards; prefer **`Card.vue`** from `packages/ui` for structure.
- **`.surface-list-row`**, **`.surface-definition-compact`**: clickable rows.
- **`.message-thread-card`**, **`.link-message-title`**: mail / thread lists.

---

## 10. Domain-specific patterns

### 10.1 “Waves” (comment threads)

- **New wave** (root): `inline-flex items-center gap-2 btn-aqua-white` + Lucide **`AudioWaveform`**, icon `text-purple-600` (see `CommentList.vue`).
- **Wave root** (jump to root): `inline-flex items-center btn-aqua-slate` + **`Home`** icon.
- Do **not** use ad-hoc `Plus` or raw `bg-blue-600` for these two actions.

### 10.2 Assistant chat

- Use **`.assistant-*`** classes for the sidebar, bubbles, and composer (see Tailwind config) so the assistant stays visually isolated from the dictionary chrome.
- **Tool-step foldouts** (semantic search results in the transcript): **`.assistant-fold-details`**, **`.assistant-fold-details-nested`**, **`.assistant-fold-summary`**, **`.assistant-fold-summary-row`**, and **`.assistant-fold-details-warning`** / **`.assistant-fold-summary-warning`** for error blocks—see `AssistantThoughtStep.vue`.

---

## 11. Motion & seasons

- Respect **`prefers-reduced-motion`** where transitions are decorative (see `style.css` for toggles). **Flat** `.btn-flat-surface` and **`.btn-empty`** shorten the **gradient overlay** opacity transition when reduced motion is requested.
- Seasonal effects (e.g. winter snowflakes) must use **`aria-hidden="true"`** on decorative containers.

---

## 12. Anti-patterns (do not ship)

- One-off **`bg-*` / `text-*`** on buttons that should use **`ui-btn--*`** or **`Button`**.
- **Margin on icons** inside flex button rows—use **`gap-*`**.
- New **hex colors** in Vue for things that already have a **named class** in `tailwind.config.js`.
- **`packages/ui`** importing **`src/`**—forbidden.

---

## 13. QA checklist (quick)

- [ ] New buttons use **`Button` / `IconButton`** or documented **`ui-btn--*`** + primitives.
- [ ] Both **`data-button-theme="flat"`** and **`aqua`** look acceptable for the touched screen.
- [ ] Icon-only controls have **`aria-label`** (or visible label).
- [ ] Focus visible on keyboard navigation (tab through interactive elements).
- [ ] No new raw duplicate of **`input-field`** styling for standard text inputs.

---

## 14. Evolution

- **Adding a new semantic button**: extend **`buttonThemeClassMap`** with both **aqua** and **flat** primitive targets, then document the intent here in §6.3 (role row + legacy synonym if any).
- **Renaming for clarity**: keep **legacy** keys in the map when renaming (e.g. `get`→`read`) so third-party forks and old strings do not break.
- **Adding a new surface**: add a **single** class in `tailwind.config.js` and reuse—avoid scattering the same 6 Tailwind utilities across files.

---

## Summary for marketing & QA

Lensisku’s UI is **white, blue-linked, tool-oriented**, with an optional **Aqua** theme for glossy controls. **Buttons are not random colors**: they follow **`ui-btn--*`** semantics mapped in Tailwind. **Typography** is **Open Sans** globally. Tests should always include **both** button themes when verifying anything that looks like a button, tab, or toolbar.
