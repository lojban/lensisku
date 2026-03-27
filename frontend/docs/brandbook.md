# Lensisku Brandbook & UI Guidelines

## Overview

This document outlines the design system, Tailwind CSS custom classes, and UI components used across the Lensisku frontend. It provides guidance on which classes/components to use when, and proposes a streamlined architecture to resolve existing discrepancies.

---

## 1. Tailwind UI Classes & The Button Ecosystem

The Lensisku UI relies on two distinct "button ecosystems": **Glossy/Aqua** (`.aqua-base`) and **Flat/Standard** (`.btn-base`).

### Why do we need two different button forms?

By having two forms, the design system creates a visual hierarchy.

- **Form Language of Aqua Buttons**: 3D, tactile, glossy, heavily shadowed. This draws immediate attention and feels like a physical "core" button. It communicates: _"I am the primary action on this page. Clicking me changes the global state or navigates to a new area."_
- **Form Language of Flat Buttons**: 2D, bordered, minimal styling, subdued. This communicates: _"I am a secondary or inline action. Clicking me modifies a specific adjacent content piece, opens a small menu, or performs a standard list interaction without leaving this context."_

### A. Glossy / Aqua Buttons (`.btn-aqua-*`)

**Purpose:** Use for main, prominent Call-to-Action (CTA) buttons or core application navigation.

| Class                                                                                   | Appearance / Color            | Target Function & User Expectation                                                                                                                                         |
| :-------------------------------------------------------------------------------------- | :---------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`btn-aqua-teal`** / **`btn-aqua-emerald`**                                            | 🟩 Green / Emerald            | **Create / Save (Primary)**: The user expects to successfully complete a major flow (e.g., Save User Profile, Create Database Entry).                                      |
| **`btn-aqua-blue`** / **`btn-aqua-sky`**                                                | 🟦 Blue / Sky                 | **Navigate / Core Action**: The user expects to move forward, submit a search, or trigger the primary neutral action of the view.                                          |
| **`btn-aqua-red`** / **`btn-aqua-rose`**                                                | 🟥 Red / Rose                 | **Destructive (Major)**: The user expects an irreversible action that affects the global state (e.g., Delete Account).                                                     |
| **`btn-aqua-orange`** / **`btn-aqua-yellow`**                                           | 🟧 Orange / Yellow            | **Warning / Attention**: The user expects to perform an action that requires caution but isn't strictly destructive (e.g., Suspend Resource, Change Permissions).          |
| **`btn-aqua-white`** / **`btn-aqua-gray`** / **`btn-aqua-slate`** / **`btn-aqua-zinc`** | ⬜ Neutral (White/Gray/Slate) | **Secondary Core Action / Toggles**: The user expects to toggle a major view mode (e.g., filters, tabs) or perform a secondary action that doesn't demand immediate focus. |

_(QA Note: If you see a blue/green Aqua button but it only edits a single row in a table, it is a mismatch. It should be a flat button.)_

### B. Flat / Standard Buttons (`.btn-*`)

**Purpose:** Use for inline actions, context-specific form submissions, and data lists.

| Class                                                   | Appearance / Color     | Target Function & User Expectation                                                                                                                    |
| :------------------------------------------------------ | :--------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`btn-create`** / **`btn-success`** / **`btn-insert`** | 🟩 Green/Blue Tones    | **Add Item / Inline Success**: The user expects to add a new item to a list, attach a file, or confirm an inline modal.                               |
| **`btn-update`** / **`btn-action`**                     | 🟦 Teal/Pink Tones     | **Edit / Modify Context**: The user expects to open an edit dialog for a specific row or trigger a rapid inline change.                               |
| **`btn-delete`** / **`btn-error`**                      | 🟥 Red Tones           | **Remove Item**: The user expects to delete a single adjacent item or row without blowing up the entire page flow.                                    |
| **`btn-cancel`** / **`btn-empty`**                      | ⬜ Gray/Neutral Tones  | **Dismiss / Clear**: The user expects to close a modal, clear a text input, or undo their current typing state.                                       |
| **`btn-revert`** / **`btn-warning`**                    | 🟨 Yellow Tones        | **Undo / Soft Reset**: The user expects to revert a local change back to its original state.                                                          |
| **`btn-reaction`** / **`btn-reply`**                    | 🔵 Gray to Blue Toggle | **Social Interaction**: The user expects to interact with community content (e.g., Like, Reply to comment). Changes from neutral to colored on click. |
| **`btn-get`** / **`btn-link`**                          | 🌐 Blue Tones (Subtle) | **Load Data / Go to Link**: The user expects to load more items inline or open a link styled as a tiny button.                                        |
| **`btn-previous`** / **`btn-next`** / **`btn-history`** | ⚙️ Context-specific    | **Pagination / Time**: Navigating timeline lists or paginating through a dataset.                                                                     |

_(QA Note: Flat buttons should never be the absolute primary action on an empty page. If a page has only one action "Create First Post", it should probably be an Aqua button.)_

#### C. Forms & Readability

- `input-field`, `textarea-field`: Use for all standard text inputs to maintain the `shadow-inner shadow-slate-200` flat styling.
- `checkbox-toggle`, `checkmark-aqua`: Aqua styling for checkboxes.
- `read-box`: For read-only text containers.

#### D. Cards

- `card-base`, `card-elevated`, `card-compact`: Standard container rules.

#### E. Wave discussion actions (comments / “waves”)

These patterns keep “start a new wave” and “go to wave root” visually consistent across entry discussions, definition-link discussions, the comments home view, and the floating action button.

**New wave (root)** — start a new top-level wave in the current discussion context (not a reply).

| Element | Specification |
| :------ | :-------------- |
| **Button classes** | `inline-flex items-center gap-2 btn-aqua-white` |
| **Icon** | Lucide `AudioWaveform`, typically `h-4 w-4 shrink-0 text-purple-600` |
| **Markup** | `type="button"`; optional `:aria-label` matching the visible label; icon, then `<span>` with the translated string |
| **Sizing** | Default follows the toolbar (compact). For empty states, FAB, or other emphasis, add utilities such as `h-12 text-base !px-5` and scale the icon (e.g. `h-6 w-6`) — same token set, stronger presence. |

Reference implementation: `CommentList.vue` (toolbar “New wave” when the list already has comments). Empty-state and FAB variants use the same classes with larger height/icon as above.

**Wave root** — when viewing a deep-linked reply (`comment_id` &gt; 0), jump back to the root of that wave (`comment_id` cleared / zero).

| Element | Specification |
| :------ | :-------------- |
| **Button classes** | `inline-flex items-center btn-aqua-slate` |
| **Icon** | Lucide `Home`, `h-5 w-5` |
| **Markup** | `type="button"`; `:aria-label` with `commentList.waveRoot`; icon and label text use `t('commentList.waveRoot')` |

Reference implementation: `CommentList.vue` (“Wave root”). Any future wave-root control should reuse this pattern (not flat blue or ad-hoc `bg-blue-600` buttons).

_(QA: “New wave” must look like the glossy white aqua + purple waveform icon; “Wave root” must be slate aqua + home icon. No `Plus` icon for these two actions.)_

---

## 2. Proposed System Improvements (Tailwind & CSS)

To prevent cognitive overhead for developers and unify the UI, the current class ecosystem should be refactored:

1.  **Semantic Over Color (Aqua)**: Instead of `btn-aqua-teal`, define `btn-aqua-success`. Have a fixed palette:
    - `Primary` (Blue/Sky)
    - `Success` (Emerald/Green)
    - `Danger` (Red/Rose)
    - `Warning` (Yellow/Orange)
    - `Default/Neutral` (White/Gray/Slate)
2.  **Unify Action Verbs (Flat)**: Deprecate highly specific classes like `btn-market` or `btn-get`. Use standard UI vocabulary:
    - `btn-primary`, `btn-secondary`, `btn-outline`, `btn-ghost`, `btn-danger`.
    - Specific contextual colors can be added via utility classes (e.g., `btn-outline text-rose-500 border-rose-500`).
3.  **Group Alignment**: Unify `.btn-group-item` and `.btn-aqua-group-item` logic using CSS variables so we don't need distinct group logic for glossy vs. flat.

---

## 3. `packages/ui` and `packages/components` Architecture

The `ui` package contains base functional wrappers (`Button.vue`, `IconButton.vue`, `Card.vue`), while `src/components` contains business-logic-heavy components.

### Current Discrepancies & Issues

1.  **`Button.vue` Hardcoded Variants:**
    The `Button.vue` component contains a `VARIANT_CLASSES` dictionary mapping strings like `aqua-white` to `btn-aqua-white`. However, it misses many classes defined in `tailwind.config.js` and behaves as a leaky abstraction.
2.  **`IconButton.vue` default classes:**
    Currently hardcodes `btn-aqua-teal` as the default button class, enforcing a glossy-success look on every basic icon button unless manually overridden.
3.  **Ambiguous Component Splitting:**
    The separation between `packages/ui` and `packages/components` (and `src/components`) is ill-defined. Wait, `packages/components` doesn't even exist (or is virtually empty/untracked).

### Recommendations for Streamlining

#### 1. Refactor `Button.vue`

Remove the hardcoded variant map. Instead, rely on a prop `theme` ("aqua" | "flat") and `color` ("primary", "danger", "success").

```vue
<!-- Proposed Button.vue API -->
<Button theme="aqua" color="success">Submit</Button>
<Button theme="flat" color="danger" variant="outline">Delete</Button>
```

_Implementation internally computes the classes `btn-aqua-teal` dynamically rather than hardcoding a list._

#### 2. Refactor `IconButton.vue`

Make `IconButton.vue` a simple wrapper around the refactored `Button.vue`, passing down properties and rendering slots. Do not hardcode `btn-aqua-teal`.

```vue
<template>
  <Button :theme="theme" :color="color" :icon-only="!label" v-bind="$attrs"> ... </Button>
</template>
```

#### 3. Standardize `packages/ui`

Treat `packages/ui` as a **dumb, stateless library**.

- It should **never** import from `src/`.
- It should accept standard HTML attributes and emit standard events.
- Deprecate any `packages/components` directory and move everything to either `src/components` (domain specific) or `packages/ui` (domain agnostic).

#### 4. Cards

`Card.vue` is currently well-architected with semantic styling and slots (`media`, `header`, `footer`). Ensure all new list items or data grids use this component rather than reinventing DOM structures in `src/components`.

---

## Summary for QA and Marketing Teams

When reviewing UI implementation, look out for:

- **Consistency in Glossy vs Flat buttons**: Are major page actions using the Aqua system, while secondary/inline forms use the Flat system?
- **Color Meanings**: Green/Emerald should strictly be for Success/Create actions. Red/Rose for Delete/Destructive actions.
- **Responsiveness**: The `.card-base` and grouping buttons (`btn-group-forced`, etc) are built to flow dynamically. Verify that list elements wrap cleanly and group items share inner borders correctly.
