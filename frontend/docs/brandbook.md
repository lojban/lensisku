# Lensisku brand book

This document is the **human-readable contract** for product, design, QA, and engineering. The **machine-readable source of truth** for tokens, component classes, and theme switching lives in **`frontend/tailwind.config.js`** (Tailwind `theme.extend`, `addBase`, `addComponents`, and the `buttonUiThemeLayer`). When code and this file disagree, **fix the code**—do not fork one-off colors or button styles in individual `.vue` files.

---

## 1. Goals

| Goal                          | What it means in practice                                                                                                                                                                        |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Recognizable product**      | Glossy **Aqua** and flat **toolbar** controls are both “Lensisku”; users can switch themes without breaking layouts. **Lingo** is a separate green chunky shell (Duolingo-style), not Aqua/Flat. |
| **Predictable actions**       | Primary flows (save, search, destructive) map to **semantic** `ui-btn--*` names, not random Tailwind color utilities on `<button>`.                                                              |
| **Accessible by default**     | Visible focus states (`focus-visible` / rings), icon-only controls require **accessible names** (`aria-label` or visible `label`).                                                               |
| **Maintainable UI**           | New screens use **named patterns** from the Tailwind layer (`input-field`, `toolbar-panel`, `card-elevated`, …) and shared **`packages/ui`** primitives.                                         |
| **No layout hacks for icons** | Icon + text controls use **`gap-*` on the parent**—never `mr-*` / `ml-*` on icons (breaks when labels are hidden or translated).                                                                 |

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
| **`tailwind.lingo.mjs`**                | **Lingo** shell: Duolingo-style chunky `.lingo-btn*`, sidebar rail, mobile header, course cards, lesson nodes (§10.3).                                                                                                      |
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

- **Navigation / links**: `text-nav-link` (`#007bff`) and **active** state via `.nav-link-active` (see `NavLink`, `navbar-item`). The mobile drawer uses `.nav-menu-grid` (two columns) and `.mobile-nav-row` (hover/focus only—link color comes from NavLink). The desktop **More** menu (`.nav-dropdown-panel`) hangs from its trigger and sizes to its contents (`w-max`), in one column (`.nav-menu-stack`). When logged in, **Logout** sits on the same row as **My activity and profile**. Language and button-theme controls use `.nav-select` / `.nav-select-field` and fill that menu.
- **Semantic blues** in surfaces: borders and hovers (`border-blue-*`, `hover:border-blue-*`) for **interactive lists** (`.surface-list-row`, `.message-thread-card`, …).
- **Status**: green success, red danger, amber warning—prefer existing **badge** and **btn** semantics over new hex values. **Yellow** is reserved for **edit / apply** (`ui-btn--edit`).
- **Content-type icons**: three glyphs per concept (§5c) — one, several, and one-plus. Import from `@packages/ui`. Hues, when required, apply to all three variants of that concept: **definitions** emerald (`.fab-menu-icon--definition`, `.definition-chip`); **favorites / collections** amber (`.fab-menu-icon--collection`); **waves** purple (`.wave-icon`, `.fab-menu-icon--discussion`); **wiki** black (`.wiki-icon` / `.fab-menu-icon--wiki`, `.wiki-chip`). **Export**, including **Export results**, stays cyan (`.fab-menu-icon--export`) on the down arrow from §5b. Import and upload stay the up arrow. Do not paint those hue classes onto the icon when it sits in a tab, filter, nav link, or colored button — those inherit `currentColor`. In a plain menu, color every icon.

### 5c. Content-type icons

Every concept below has **three** glyphs. They share one hue. **One** is the single-item picture. **Several** is a different picture for a list or a set. **Create** adds a plus in the bottom-right corner, with the same strokes as Lucide `StarPlus` (`M15 18h6`, `M18 15v6`), and the plus does not join the glyph.

| Concept                      | One                                   | Several (a list, a collection, results, exporting that set)                       | Create (new instance)                                                                                                   | Hue when required                                                                                                                     |
| ---------------------------- | ------------------------------------- | --------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| **Thread**                   | `MessageSquare` (one comment)         | `MessagesSquare` (discuss, discussions tab, discussions as a source)              | `WaveCreateIcon` (plus above two waves: new wave, new discussion, new thread). A new comment stays `MessageSquarePlus`. | purple: `.wave-icon`, `.fab-menu-icon--discussion`                                                                                    |
| **Definition**               | `DefinitionIcon` (Lucide `TextQuote`) | `DefinitionsIcon` (Lucide `List`)                                                 | `DefinitionCreateIcon` (quoted gloss + plus)                                                                            | emerald: `.definition-chip`, `.fab-menu-icon--definition`                                                                             |
| **Wiki article**             | `WikiIcon` (the letter W)             | `WikisIcon` (stacked books with a W: wiki pages, the wiki book, wiki as a source) | `WikiCreateIcon` (book, plus, and W: new wiki page, into wiki)                                                          | black: `.wiki-icon`, `.fab-menu-icon--wiki`, `.wiki-chip`                                                                             |
| **Favorite / collection**    | `StarIcon` (Lucide `Star`)            | `StarsIcon` (star with the left half of a star beside it)                         | `StarCreateIcon` (Lucide `StarPlus`)                                                                                    | amber: `.fab-menu-icon--collection`                                                                                                   |
| **Entry** (valsi / headword) | `EntryIcon` (`BookA`)                 | —                                                                                 | —                                                                                                                       | blue chip: `.entry-chip`. Page title inherits text color. A new dictionary sense is **create definition**, not a separate entry-plus. |

**Which variant**

1. **One** — a single thing: “In definition”, “In entry”, one wiki chip, one comment, one favorite.
2. **Several** — a set of that thing: the definitions tab (`List`), the wiki tab, the discussions tab (`MessagesSquare`), “My collections”, an empty list of that type. The activity **All** tab is Lucide `Layers`. The wave-source filter’s **All** option is Lucide `LayoutGrid`. The list glyph stays with definitions.
3. **Create** — making a new one: new definition, new wave (`WaveCreateIcon`), new wiki article, add to a collection. A new comment stays `MessageSquarePlus`.

**Rules**

1. Do not invent a fourth drawing for the same concept. “Create wiki” is `WikiCreateIcon`, not a bare `WikiIcon` and not Lucide `FilePlus`.
2. Do not substitute Lucide `BookMarked`, `BookOpen`, `Globe`, or `Globe2` for wiki, and do not use `BookA` or `TextQuote` for wiki. Discuss links, the discussions tab, and discussions as a source use Lucide `MessagesSquare`. Do not use `AudioWaveform` or `WavesHorizontal` for those.
3. A collection or course with no cover uses `PlayingCardsFanIcon` (Lucide `playing-cards-fan`). That is the cover fallback, not a dictionary entry. Amber on a star is the collection hue, not the edit-yellow button.
4. **Threads and comments:** discuss links, the discussions tab, and discussions as a source use Lucide `MessagesSquare`. A new wave, a new discussion, and a new thread use `WaveCreateIcon` (a plus above two wave lines). A new comment uses `MessageSquarePlus`. A single comment, and comment filters, use `MessageSquare`. Freeforums as a source uses Lucide `MessageCircle` (the circle bubble).
5. Source chips are always the **one** glyph: `.entry-chip`, `.definition-chip`, `.wiki-chip`, `.mail-chip`, `.jbotcan-chip`, `.freeforums-chip`.
6. In a **plain menu**, every icon takes its concept hue. Tabs, filters, nav links, and colored buttons inherit `currentColor`.
7. Exporting a file, a generated download, or a **result set** (search **Export results**) uses the down arrow in §5b (`ExportIcon` or `FileExportIcon`), with `.fab-menu-icon--export` (cyan) in a plain menu. It does not use a content-type glyph.

- **Selma'o / tags**: `.badge-definition-tag` (purple chip)—use for grammatical tags, not general emphasis.

### 5b. Transfer-direction icons (import / upload · export / download)

| Action                                 | Direction      | Approved Lucide glyphs | Import from                                                                                       |
| -------------------------------------- | -------------- | ---------------------- | ------------------------------------------------------------------------------------------------- |
| **Import** (pick file / bring data in) | Arrow **up**   | `Upload`, `FileUp`     | `ImportIcon`, `FileImportIcon` from `@packages/ui`                                                |
| **Upload** (send file to server)       | Arrow **up**   | `Upload`               | `UploadIcon` from `@packages/ui`                                                                  |
| **Export** (generate / send data out)  | Arrow **down** | `Download`, `FileDown` | `ExportIcon`, `FileExportIcon` from `@packages/ui`. Plain menus: `.fab-menu-icon--export` (cyan). |
| **Download** (save locally)            | Arrow **down** | `Download`             | `DownloadIcon` from `@packages/ui`                                                                |

**Rules**

1. **Import and upload** always use arrow-**up** icons; **export and download** always use arrow-**down** icons—do not swap them even if other products do.
2. File-oriented menus: **`FileExportIcon`** / **`FileImportIcon`**; generic upload: **`UploadIcon`**; generic export, including search **Export results**: **`ExportIcon`** (down arrow); cached downloads / attachments: **`DownloadIcon`**; file import buttons: **`ImportIcon`**. In a plain menu, the down arrow uses **`.fab-menu-icon--export`** (cyan). Nav links keep the link color. Content-type glyphs (§5c) are not substitutes for these arrows.
3. Voting, reorder, and unrelated arrows (`ArrowUp`, `ArrowDown`, …) are **out of scope**.
4. ESLint blocks direct `@lucide/vue` use of `Upload`, `Download`, `FileUp`, `FileDown`, and `Import`; **`packages/ui/transferIcons.ts`** is the only re-export site. Do not use Lucide’s `Import` glyph (it is a down arrow).

---

## 6. Button system (three families)

Lensisku dictionary chrome uses **Aqua** or **Flat** (user toggle). **Lingo** (`/lingo`, collection lingo study) uses a separate **chunky** family (§10.3) that does **not** follow `data-button-theme`.

### 6.1 Theme switch: Aqua vs Flat

- Stored as `lensisku.buttonTheme` in `localStorage`; mirrored on **`html[data-button-theme="aqua"]`** or **`flat`** (default **flat** if unset).
- **`index.html`** runs a small script before Vue boots so the first paint uses the correct primitive classes.
- **Does not apply** to Lingo shell controls (`.lingo-btn*`, `.lingo-icon-btn`, lesson nodes, course cards).

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

| Action role                   | Canonical class                                      | Typical use                                                                                           |
| ----------------------------- | ---------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| **Read / open / navigate**    | `ui-btn--read`                                       | Links, “open”, export/download, neutral forward. _Legacy:_ `ui-btn--get`.                             |
| **Edit / apply**              | `ui-btn--edit`                                       | Save / apply / bulk-edit — **yellow** (`btn-update` / `btn-aqua-yellow`). _Legacy:_ `ui-btn--update`. |
| **Create / add**              | `ui-btn--create`                                     | Submit new entity, add row. Often paired with **`ui-btn--primary`** (same mapping).                   |
| **Delete / remove**           | `ui-btn--delete`                                     | Destructive removal. _Synonym:_ `ui-btn--remove` (same mapping).                                      |
| **Dismiss / cancel**          | `ui-btn--cancel`                                     | Close modal, secondary “no”. _Synonym:_ `ui-btn--dismiss`.                                            |
| **Toolbar / filter chrome**   | `ui-btn--toolbar`                                    | Compact toggles, reset filters (`Button variant="toolbar"`). _Legacy:_ `ui-btn--aqua-default`.        |
| **Neutral / low emphasis**    | `ui-btn--empty`                                      | Deselect, clipboard, embossed default.                                                                |
| **Neutral (tinted)**          | `ui-btn--neutral`, `neutral-muted`, `neutral-slate`  | Secondary pills, back links, muted actions.                                                           |
| **Pagination back / forward** | `ui-btn--back`, `ui-btn--forward`                    | Prev/next page. _Legacy:_ `previous`, `next`.                                                         |
| **Continue**                  | `ui-btn--continue`                                   | Resume flow (maps like read/get).                                                                     |
| **Market / commerce**         | `ui-btn--market`                                     | Rose / market styling.                                                                                |
| **History / versions**        | `ui-btn--history`                                    | Version list, purple history.                                                                         |
| **Insert / inline add**       | `ui-btn--insert`                                     | Add to selection, blue insert.                                                                        |
| **Link / outbound**           | `ui-btn--link`                                       | Blue link-style.                                                                                      |
| **Accent (activity)**         | `ui-btn--accent-purple`                              | “View activity” and similar.                                                                          |
| **Generic action (pink)**     | `ui-btn--action`                                     | Promotional / emphasis CTA.                                                                           |
| **Reply**                     | `ui-btn--reply`                                      | Thread reply.                                                                                         |
| **Reaction**                  | `ui-btn--reaction`, `reaction-active`                | Toggle reactions.                                                                                     |
| **Auth**                      | `ui-btn--auth-login`, `auth-signup`                  | Login vs signup emphasis.                                                                             |
| **Status**                    | `ui-btn--success`, `error`, `warning`, `danger-rose` | Result / alert / destructive rose.                                                                    |
| **Revert**                    | `ui-btn--revert`                                     | Undo to old revision — **amber** (yellow reserved for edit).                                          |
| **Study grading**             | `study-correct`, `study-wrong`                       | Flashcard / Lingo result.                                                                             |
| **Warning shades**            | `warning`, `warning-orange`, `warning-yellow`        | Caution; orange/amber vs decorative **yellow** (`warning-yellow` / `palette-yellow`).                 |
| **Sort / palette**            | `sort-*`, `palette-*`                                | Rotating colors in lists (see map).                                                                   |
| **Segmented group**           | `ui-btn--group-item`                                 | Inner segment (with `.btn-group` / `.btn-group-forced`).                                              |
| **Special**                   | `ui-btn--fab`, `toggle`                              | FAB and toggle have extra rules in the same Tailwind layer.                                           |

**Rules**

1. Prefer **`<Button variant="…">`** or **`<IconButton button-classes="…">`** over raw `class="btn-aqua-emerald"` in new code—keeps theme switching coherent.
2. **Account / profile toolbars**: pair **read** (`ui-btn--read`, e.g. change password) with **edit** (`ui-btn--edit`). Do not use **`ui-btn--warning-*`** for routine account actions—they read as caution chrome.
3. **Yellow = edit (at most one per cluster)**: `ui-btn--edit` / `ui-btn--update` use yellow. Decorative yellow (`ui-btn--warning-yellow`, `ui-btn--palette-yellow`) shares that hue. In any **toolbar**, **`.btn-group`**, or **`.btn-group-forced`**, show **at most one** yellow control—never two edits, or edit beside `warning-yellow` / `palette-yellow`. Prefer amber/orange for secondary caution actions in the same row.
4. **Flat theme**: major empty-state CTAs may still use **`ui-btn--*`**; do not mix random `bg-blue-600` full-width buttons unless using an existing pattern (e.g. `.btn-panel-primary` for specific panels).
5. **Icon-only**: must have **`aria-label`** (or visible text). `IconButton` derives `aria-label` from `ariaLabel` or `label`.

### 6.4 When Aqua vs Flat “wins”

- Users choose **Aqua** for nostalgia / gloss; **Flat** for calmer, flatter UI.
- QA should verify **both** themes on any change that touches `ui-btn--*` or `btn-aqua-*` / `btn-*` primitives.

---

## 7. Forms & inputs

- Text fields: **`.input-field`**, **`.textarea-field`** (inner shadow, blue focus ring).
- Tall toolbar triggers (filters): **`.dropdown-trigger`** (height aligned with `h-10` inputs).
- Action / overflow menus (ellipsis): **`.dropdown-action-trigger`** — Aqua uses glossy button chrome; Flat uses the select pill **without** inner shadow.
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

| Class                                                       | Use                                                                                                 |
| ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `.app-header-bar`                                           | Sticky top chrome (logo, nav, auth).                                                                |
| `.page-loading-overlay`                                     | Full-screen loading veil (auth bootstrap).                                                          |
| `.content-loading-overlay`                                  | In-region veil over a `relative` parent (filter/search refresh).                                    |
| `.route-view-freeze` + `.route-freeze-spinner`              | Path-change overlay in App (not query/tabs).                                                        |
| `.auth-page-shell` / `.auth-form-card` / `.auth-glass-card` | Login, signup, password flows.                                                                      |
| `.toolbar-panel` + `.toolbar-row`                           | Search + filter toolbars.                                                                           |
| `.page-section-title`                                       | In-page H1-style tool titles.                                                                       |
| `.empty-state-panel`                                        | Zero-state blocks.                                                                                  |
| `.feed-page` / `__header` / `__body` / `__footer`           | Dictionary full-height feeds: pad owner is `__body` (`px-3 pt-3`).                                  |
| `.lingo-main-inner`                                         | Lingo **width** shell only (`max-w-[1056px]`). **No** padding.                                      |
| Lingo + `feed-page`                                         | Pad owner: `.lingo-main-inner .feed-page__body` (`px-4 pt-4 pb-4 lg:px-6`) in `tailwind.lingo.mjs`. |
| `.lingo-feed-layout`                                        | Pad owner for levels/leaderboard/etc. (`px-4 py-4 lg:px-6`) when not using `feed-page`.             |

**Padding rule:** one inset owner per Lingo page — never `.lingo-main-inner` **and** `__body` / `.lingo-feed-layout`.

---

## 9. Cards & lists

- **`.card-base`**, **`.card-elevated`**, **`.card-compact`**: list/detail cards; prefer **`Card.vue`** from `packages/ui` for structure.
- **`.surface-list-row`**, **`.surface-definition-compact`**: clickable rows.
- **`.message-thread-card`**, **`.link-message-title`**: mail / thread lists.

---

## 10. Domain-specific patterns

### 10.1 “Waves” (comment threads)

- **New wave / new discussion / new thread**: `WaveCreateIcon` from `@packages/ui` (a plus above two wave lines). In a plain menu add `.fab-menu-icon--discussion` (purple). **New comment**: Lucide `MessageSquarePlus`. **Discuss** (entry header, linked definition, definition toolbar) and **the discussions tab**: Lucide `MessagesSquare`. Colored buttons inherit `currentColor`.
- **Wave root** (jump to root): `inline-flex items-center btn-aqua-slate` + **`Home`** icon.
- Do **not** use Lucide `Waves`, ad-hoc `Plus`, or raw `bg-blue-600` for these actions.

### 10.2 Assistant chat

- Use **`.assistant-*`** classes for the sidebar, bubbles, and composer (see Tailwind config) so the assistant stays visually isolated from the dictionary chrome.
- **Tool-step foldouts** (semantic search results in the transcript): **`.assistant-fold-details`**, **`.assistant-fold-details-nested`**, **`.assistant-fold-summary`**, **`.assistant-fold-summary-row`**, and **`.assistant-fold-details-warning`** / **`.assistant-fold-summary-warning`** for error blocks—see `AssistantThoughtStep.vue`.

### 10.3 Lingo (study shell) — third control family

`/lingo/*` and `/collections/:id/lingo/*` use a **Duolingo-style** shell copied from `archive/duolingo-clone`, not Aqua or Flat pills. Machine source: **`frontend/tailwind.lingo.mjs`** (merged into `tailwind.config.js`).

| Piece              | Class / meta                                                                                                                                                                                                                                                   | Notes                                                                                                                                                                                                           |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Route shell**    | `meta.lingoShell` (+ `hideTopBar`, `hideFab`, `hideFooter`, `fullHeight`)                                                                                                                                                                                      | Hides Lensisku top bar / FAB / footer; `#main-child` goes full-bleed (`max-w-none`).                                                                                                                            |
| **Layout**         | `LingoLayout` → chrome + `.lingo-main` → **`.lingo-main-inner`** (width only)                                                                                                                                                                                  | `<lg`: App-matching top bar + bottom tabs. Desktop: **256px** rail. **Inset:** `feed-page__body` _or_ `.lingo-feed-layout` — never both, never on `.lingo-main-inner`.                                          |
| **Chunky buttons** | `.lingo-btn` + `.lingo-btn--{default\|primary\|secondary\|danger\|locked\|ghost\|sidebar\|sidebar-active}`                                                                                                                                                     | Thick **bottom border** (`border-b-4` / press `border-b-0` or `border-b-2`), `rounded-xl`, uppercase tracking. Sizes: `--sm`, `--lg`, `--icon`, `--rounded`.                                                    |
| **Icon chrome**    | `.lingo-icon-btn`, `--on-green`, `--muted`                                                                                                                                                                                                                     | Menu / close / back — **never** `Button` / `ui-btn--*` (Aqua gloss “mutilates” these). Drawer close sits in `.lingo-drawer-toolbar` (own row), not absolutely over nav.                                         |
| **Course tiles**   | `.lingo-course-card` + `__media` / `__title` / `__desc` / `__active-badge`; `--active`                                                                                                                                                                         | Top-aligned fixed slots (cover → 2-line title → 2-line desc). **Current course:** sky border + in-card green badge with **Dumbbell** (same as Learn), not clone Check. `aria-current`. Not `variant="neutral"`. |
| **Lesson ladder**  | `.lingo-lesson-node` + `--open` / `--locked`                                                                                                                                                                                                                   | 70×70 circle, `border-b-8`.                                                                                                                                                                                     |
| **Unit banner**    | `.lingo-unit-banner`                                                                                                                                                                                                                                           | Green strip; Continue uses `.lingo-btn--secondary`.                                                                                                                                                             |
| **Study**          | Full-bleed chrome + `.lingo-study-chrome-inner`; title `.lingo-study-lesson-title` (`lg:text-3xl`); stimulus `.lingo-study-question-bubble` + `.lingo-study-stimulus` (`text-sm`/`lg:text-base`); card `.lingo-study-card-shell`; rate `.lingo-study-rate-row` | Prompt is large; stimulus is smaller in a speech bubble (clone ASSIST). Check → footer grades. Quiz: `.study-quiz-option-lingo-*`.                                                                              |

**Rules**

1. Inside Lingo chrome (sidebar, course grid, ladder, study check/next), use **`.lingo-*` only** — do not wrap those controls in `packages/ui` `Button` / `IconButton` with Aqua/Flat themes. Exception: `.lingo-mobile-header` matches the App.vue top bar (logo + `Button variant="topbar"` logout).
2. Lensisku **modals** opened from Lingo (create level, add cards, delete confirm) may still use **`ui-btn--*`** — they are dictionary tooling, not lesson chrome.
3. Green (`green-500` / `green-600`) is the Lingo accent; sky is for **active sidebar** and **primary** CTAs (create), matching the clone. Do not introduce purple gradients or Aqua gloss here.
4. When adding a new Lingo control, extend **`tailwind.lingo.mjs`** and document the class in this section — do not scatter one-off `border-b-4` utilities in Vue files.
5. **Do not** add `px-*` / `pt-*` / `pb-*` on page roots inside `LingoLayout` to “fix” spacing — change the pad owner (`.feed-page__body` under Lingo, or `.lingo-feed-layout`) in `tailwind.lingo.mjs`.

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
- On **Lingo** pages: wrapping menu / close / course cards / study Check in **`Button` `variant="neutral"`** (Aqua gloss “mutilates” Duolingo chunky controls)—use **`.lingo-btn*` / `.lingo-icon-btn`** instead (§10.3).
- Showing the Lensisku **top bar or FAB** on `/lingo` routes—set **`meta.lingoShell`** (and hide flags) on the route.
- Stacking **two content insets** (padding on `.lingo-main-inner` plus `__body` / `.lingo-feed-layout`) — pick one pad owner in `tailwind.lingo.mjs`.

---

## 13. QA checklist (quick)

- [ ] New buttons use **`Button` / `IconButton`** or documented **`ui-btn--*`** + primitives.
- [ ] Both **`data-button-theme="flat"`** and **`aqua`** look acceptable for the touched screen (skip for pure Lingo chrome — use §10.3 checklist instead).
- [ ] Lingo routes: no Lensisku top bar / FAB; menu and close are flat `.lingo-icon-btn`; desktop rail is 256px; **one** content pad owner (`feed-page__body` or `.lingo-feed-layout`), not `.lingo-main-inner`.
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

**Lingo** (`/lingo/…`, collection ladder + study) is a separate product shell: green mobile bar, fixed left rail on desktop, **chunky bottom-border buttons** (no Aqua gloss). QA: confirm the Lensisku top bar and pink FAB are **hidden**, sidebar fits on mobile drawer and desktop rail, and Check/Continue study buttons stay green/rose chunky pills—not metallic Aqua circles.
