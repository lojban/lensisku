import { buildFlatButtonLayer } from './tailwind.flat-buttons.mjs'
import {
  aquaToggleOffShadow,
  buildAquaBaseLayer,
  buildAquaButtonGroupLayer,
  buildAquaButtonPrimitives,
  buildAquaUiBtnGroupItemGeometry,
} from './tailwind.aqua-buttons.mjs'

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{vue,js,ts,jsx,tsx}',
    './packages/**/*.{vue,js,ts,jsx,tsx}',
  ],
  /**
   * `packages/ui/Button.vue` resolves `variant` to `ui-btn--${suffix}` at runtime. Those strings are not
   * static in source, so JIT would otherwise skip `buttonUiThemeLayer` rules and the `btn-aqua-*` /
   * `btn-*` primitives they `@apply` (e.g. study CTA palette buttons on collection cards).
   */
  safelist: [{ pattern: /^ui-btn--[a-z0-9-]+$/ }],
  theme: {
    extend: {
      /** Global UI stack; glossy `.aqua-base` inherits via `@apply font-sans` (see `docs/brandbook.md`). */
      fontFamily: {
        sans: ['"Open Sans"', 'system-ui', 'Tahoma', 'sans-serif'],
      },
      colors: {
        /** Primary nav / legacy link blue (navbar-item, NavLink, mobile logout). */
        'nav-link': '#007bff',
        /** FAB / accent — classic cornflower blue and steps for hover/active. */
        cornflower: {
          400: '#6495ED',
          500: '#5789E8',
          600: '#4B7DDB',
          700: '#3D6BC4',
        },
      },
    },
  },
  plugins: [
    function ({ addComponents, addBase, theme }) {
      /** Base layer: `btn-base` + Aqua theme bases (`tailwind.aqua-buttons.mjs`). */
      addBase({
        /** Icon + label: use this flex `gap-*` only—do not put `mr-*` / `ml-*` on icons or labels (breaks when labels are hidden). */
        /**
         * Shared pill geometry for **Flat** (with `.btn-flat-surface` / semantic `btn-*`) and other controls.
         * Tinted flat fills inherit interaction from `.btn-flat-surface` (CSS variables); do not add a generic hover fill here.
         *
         * **Core** omits hover `shadow-none` so embossed `.btn-empty` can own hover box-shadow without a duplicate
         * rule fighting `btn-base` (links + buttons both use the same primitives via `ui-btn--empty` → `@apply`).
         */
        '.btn-base-core': {
          '@apply gap-2 px-3 md:px-4 py-1.5 text-xs font-medium flex items-center justify-center h-6 border rounded-full transition-colors duration-200 shadow-sm shadow-slate-200 disabled:opacity-40 select-none disabled:cursor-not-allowed whitespace-nowrap focus:outline-none cursor-pointer':
            {},
          '&:not(:disabled)': {
            '--tw-ring-color': 'var(--btn-color, currentColor)',
            '@apply active:scale-x-[1.02]': {},
          },
        },
        '.btn-base': {
          '@apply btn-base-core': {},
          '&:not(:disabled):hover': {
            '@apply ring-0 shadow-none': {},
          },
        },
        ...buildAquaBaseLayer(),
      })

      /** Global document defaults. */
      addBase({
        html: {
          '@apply font-sans antialiased': {},
        },
      })

      /**
       * Component registry (purpose / theme order for readability):
       * prose → app shell → nav → banners & toasts → icon buttons → surfaces & lists → assistant →
       * auth & empty states → dropdowns & filters → study & lingo → avatars & attachments →
       * forms & toolbar → aqua buttons → flat buttons → groups & toggles → cards & badges → tabs.
       */
      addComponents({
        // --- Typography & prose ---
        blockquote: {
          '@apply pl-4 border-l-4 border-gray-300 my-4 text-sm italic': {},
          '& p': {
            '@apply text-gray-600': {},
          },
        },
        // --- Navigation (desktop + mobile) ---
        '.navbar-item': {
          '@apply flex h-9 min-w-12 items-center justify-center gap-2 rounded-full px-2 py-1.5 text-base font-medium text-gray-600 transition-colors select-none whitespace-nowrap md:px-4':
            {},
          '&:not(.primary):not(.nav-link-active)': {
            '@apply hover:bg-gray-200 text-nav-link': {},
          },
        },
        '.btn-login': {
          '@apply navbar-item border border-gray-300 px-3 sm:px-4': {},
          '&.nav-link-active': {
            '@apply !text-gray-400 border-gray-200': {},
          },
        },
        '.btn-signup': {
          '@apply navbar-item border border-gray-300 px-3 sm:px-4 !text-teal-600': {},
          '&.nav-link-active': {
            '@apply !text-gray-400 border-gray-200': {},
          },
        },
        // --- App shell & global chrome ---
        /** Full-viewport loading veil: blur content behind spinner (used by LoadingSpinner page variant). */
        '.page-loading-overlay': {
          '@apply fixed inset-0 z-50 flex min-h-0 items-center justify-center bg-white/50 backdrop-blur-sm':
            {},
        },
        /** Sticky app chrome (logo, nav, auth). */
        '.app-header-bar': {
          '@apply sticky top-0 z-40 border-b border-gray-200 bg-white': {},
        },
        /**
         * FAB outer shell: `aqua-base` sets `overflow:hidden` (gloss), which clips the button’s own
         * outer `box-shadow` in browsers — elevation lives on this wrapper instead (ui-ux-pro-max).
         */
        // --- FAB & toggles (chrome-adjacent) ---
        '.fab-elevation-shell': {
          '@apply inline-flex cursor-pointer rounded-full ring-1 ring-slate-900/10 transition-[box-shadow] duration-200':
            {},
          boxShadow:
            '0 10px 26px -8px rgba(15, 23, 42, 0.18), 0 6px 14px -6px rgba(225, 29, 72, 0.22)',
          '&:hover': {
            boxShadow:
              '0 18px 36px -10px rgba(15, 23, 42, 0.22), 0 10px 22px -8px rgba(225, 29, 72, 0.32)',
          },
        },
        /** Footer modal: iOS-style toggle track + thumb (pair thumb with translate-x-0 / translate-x-5). */
        '.toggle-switch': {
          '@apply relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2':
            {},
        },
        '.toggle-switch-thumb': {
          '@apply pointer-events-none inline-block h-5 w-5 rounded-full bg-white shadow transition duration-200 ease-in-out will-change-transform':
            {},
        },
        /**
         * Checkbox sibling track (`CommentList` / definition link discussion): put `peer` + `sr-only` on the `<input>`;
         * this class goes on the following `<div>` (do not add `peer` here).
         */
        '.toggle-switch-peer-track': {
          "@apply relative h-5 w-9 shrink-0 rounded-full bg-gray-200 after:absolute after:left-[2px] after:top-[2px] after:h-4 after:w-4 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-focus:outline-none peer-checked:after:translate-x-full peer-checked:after:border-white peer-checked:bg-nav-link":
            {},
        },
        // --- Banners & toasts ---
        /** Anonymous progress banner: touch-sized auth links (`RouterLink` + `ui-btn--*`). */
        '.anon-banner-cta': {
          '@apply min-h-[44px] min-w-[44px] items-center justify-center rounded-md px-3 py-2 text-sm inline-flex':
            {},
        },
        /** Dismiss control (icon-only, 44px min). */
        '.anon-banner-dismiss': {
          '@apply icon-btn-ghost flex min-h-[44px] min-w-[44px] items-center justify-center rounded-md p-2.5 text-gray-400 hover:bg-gray-100 hover:text-gray-600':
            {},
        },
        /** Fixed strip under app header (`PageBanner`, test-data warnings). */
        '.page-banner-fixed': {
          '@apply fixed left-0 right-0 top-14 z-10 mx-auto border text-center select-none md:top-12':
            {},
        },
        '.page-banner--warning': {
          '@apply w-full px-4 py-2 text-sm opacity-90 bg-yellow-100 border-yellow-300': {},
        },
        '.page-banner--error': {
          '@apply w-fit px-2 py-0 text-xs opacity-80 bg-red-100 border-red-200': {},
        },
        /** Centered toast / lightweight alert surface (`ToastFloat.vue`); matches card elevation + success/error borders. */
        '.toast-float-shell': {
          '@apply pointer-events-none fixed inset-0 z-[65] flex items-center justify-center p-4':
            {},
        },
        '.toast-float-panel': {
          '@apply pointer-events-auto w-full max-w-[min(90vw,28rem)] overflow-hidden rounded-2xl border border-gray-200 bg-white text-gray-800':
            {},
          boxShadow: '0 0.75px 3px rgba(0, 0, 0, 0.04), 0 6px 16px rgba(0, 0, 0, 0.06)',
        },
        '.toast-float-panel--success': {
          '@apply border-green-300': {},
        },
        '.toast-float-panel--error': {
          '@apply border-red-300': {},
        },
        '.toast-float-body': {
          '@apply flex items-center gap-3 p-4 sm:p-5': {},
        },
        '.toast-float-icon': {
          '@apply shrink-0 self-center': {},
        },
        '.toast-float-icon--success': {
          '@apply text-green-600': {},
        },
        '.toast-float-icon--error': {
          '@apply text-red-600': {},
        },
        '.toast-float-message': {
          '@apply min-w-0 flex-1 text-sm font-medium leading-snug text-gray-800 sm:text-base': {},
        },
        /** Dismiss control: same affordance family as modal chrome (neutral hover, visible focus). */
        '.toast-float-close': {
          '@apply -m-1 flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-800 focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-400 focus-visible:ring-offset-2 focus-visible:ring-offset-white':
            {},
        },
        /** `ModalComponent` default slot: scroll area. Padding inset keeps button box-shadows and focus rings inside the scrollport (they paint outside the border box and would otherwise clip). */
        '.modal-scroll-body': {
          '@apply min-h-0 flex-1 overflow-y-auto overscroll-contain px-2 pt-2 pb-6 sm:px-3 sm:pb-7':
            {},
        },
        '.toast-float-extra': {
          '@apply border-t border-gray-100 pt-3': {},
        },
        // --- Icon buttons ---
        /** Circular icon-only control (field clears, trailing actions); matches toast-close affordance at smaller hit target. */
        '.icon-btn-ghost': {
          '@apply inline-flex items-center justify-center shrink-0 rounded-full p-1 text-gray-400 transition-colors duration-200 hover:text-gray-600 hover:bg-gray-100 focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-1':
            {},
          '&:hover > svg': {
            '@apply text-gray-600': {},
          },
        },
        '.icon-btn-ghost--compact': {
          '@apply p-0.5': {},
        },
        /** Default flex layout for packages/ui IconButton (aqua / flat ui-btn). */
        '.icon-btn-ui-layout': {
          '@apply inline-flex items-center gap-2 w-auto md:flex-none': {},
        },
        /** Destructive icon control (e.g. delete in assistant chat list). */
        '.icon-btn-ghost-danger': {
          '@apply shrink-0 rounded p-1 text-gray-400 transition-colors hover:text-red-600 hover:bg-red-50 focus:outline-none focus:ring-2 focus:ring-red-300 md:opacity-100':
            {},
        },
        '.icon-btn-ghost-danger--reveal-md': {
          '@apply md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100': {},
        },
        // --- Surfaces, chips & list rows ---
        /** Clickable list / grid row: bordered card with hover + focus ring (user list, pickers). */
        '.surface-list-row': {
          '@apply min-w-0 max-w-full bg-white p-4 sm:p-5 rounded-xl border border-gray-200 transition-all duration-200 cursor-pointer focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 hover:border-blue-400/60 hover:shadow-md':
            {},
        },
        /** Selma\'o / definition tag chip (purple). */
        '.badge-definition-tag': {
          '@apply inline-flex items-center px-2 py-1 text-xs font-medium bg-purple-100 text-purple-700 rounded-md justify-center sm:justify-start hover:bg-purple-200 hover:text-purple-800 transition-colors min-w-0 max-w-full truncate':
            {},
        },
        '.badge-definition-tag--pill': {
          '@apply rounded-full': {},
        },
        /** Search similarity pill on definition cards (centered on top edge, straddles border). */
        '.badge-definition-similarity': {
          '@apply pointer-events-none absolute left-1/2 top-0 z-10 -translate-x-1/2 -translate-y-1/2 rounded-full border border-gray-200 bg-gray-100 px-2.5 py-1 text-xs font-medium italic text-gray-600 shadow-sm':
            {},
        },
        /** Monospace snippet (canonical form, inline code blocks). */
        '.code-snippet-surface': {
          '@apply text-sm text-gray-700 font-mono bg-blue-50/30 p-2 rounded border border-blue-100/30 whitespace-pre-wrap leading-relaxed overflow-x-auto':
            {},
        },
        /** Compact monospace chip (study UI). */
        '.code-snippet-inline': {
          '@apply text-[10px] text-gray-700 font-mono bg-blue-50/30 px-1.5 py-0.5 rounded border border-blue-100/30 inline-block mx-auto sm:text-xs':
            {},
        },
        /** App mobile drawer row: layout + hover only — link color comes from `NavLink` (`text-nav-link` / `nav-link-active`). */
        '.mobile-nav-row': {
          '@apply flex cursor-pointer items-center gap-2 rounded-md px-4 py-2 text-base transition-colors duration-200 hover:bg-gray-100 hover:text-blue-800 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-400 focus-visible:ring-offset-2 focus-visible:ring-offset-white':
            {},
        },
        /** Logout / primary action row in mobile nav. */
        '.mobile-nav-row--emphasis': {
          '@apply flex w-full cursor-pointer items-center gap-2 rounded-md px-4 py-2 text-left text-base text-nav-link transition-colors duration-200 hover:bg-gray-100 hover:text-blue-800 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-400 focus-visible:ring-offset-2 focus-visible:ring-offset-white':
            {},
        },
        /** Desktop “More” dropdown panel under navbar. */
        '.nav-dropdown-panel': {
          '@apply absolute flex-col mt-0 bg-white border border-gray-200 rounded-md shadow-lg z-50 p-1 w-auto max-w-96':
            {},
        },
        // --- Assistant (chat UI) ---
        /** Assistant chat: scrollable message column shell. */
        '.assistant-messages-pane': {
          '@apply relative min-h-0 flex-1 overflow-x-hidden rounded-lg border border-gray-200 bg-white [overscroll-behavior-y:contain]':
            {},
        },
        /** Assistant chat: sidebar chat search field. */
        '.assistant-input-search': {
          '@apply w-full rounded-lg border border-gray-200 bg-white py-2 pl-9 pr-3 text-sm text-gray-900 placeholder:text-gray-400 focus:border-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400/30':
            {},
        },
        /** Assistant chat: “New chat” dashed control in sidebar. */
        '.assistant-new-chat-trigger': {
          '@apply flex min-w-0 flex-1 items-center justify-center gap-2 rounded-lg border border-dashed border-gray-300 bg-white/80 py-2.5 text-sm font-medium text-gray-700 hover:border-blue-400 hover:bg-blue-50/60 hover:text-blue-800 focus:outline-none focus:ring-2 focus:ring-blue-400/40 transition-colors':
            {},
        },
        /** Assistant chat: session list item button (base). Pair with Tailwind `group` on the element for child reveal. */
        '.assistant-session-row': {
          '@apply w-full text-left rounded-lg px-2.5 py-2.5 transition-colors focus:outline-none focus:ring-2 focus:ring-inset focus:ring-blue-400/50':
            {},
        },
        '.assistant-session-row--active': {
          '@apply bg-blue-100/90 border border-blue-200/80 shadow-sm': {},
        },
        '.assistant-session-row--idle': {
          '@apply border border-transparent hover:bg-gray-100/90': {},
        },
        /** Assistant: compact header icon (close sidebar, open drawer). */
        '.assistant-icon-btn-soft': {
          '@apply shrink-0 rounded-lg p-1.5 text-gray-600 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-blue-400/50':
            {},
        },
        /** Assistant: compact header icon (new chat, etc.) — matches panel control look without responsive hide rules. Inset ring so focus/active visuals stay inside the box under `overflow-hidden` ancestors (assistant main column). */
        '.assistant-icon-btn-header': {
          '@apply shrink-0 inline-flex items-center justify-center p-2 rounded-lg border border-gray-200 bg-white text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-blue-400/50':
            {},
        },
        '.assistant-icon-btn-panel': {
          '@apply assistant-icon-btn-header md:hidden': {},
        },
        /** Assistant bubble: user message. */
        '.assistant-bubble-user': {
          '@apply min-w-0 max-w-[calc(100%-2.5rem)] rounded-lg px-3 py-2 text-sm break-words bg-blue-600 text-white whitespace-pre-wrap':
            {},
        },
        /** Assistant bubble: assistant markdown body. */
        '.assistant-bubble-assistant': {
          '@apply max-w-[80%] min-w-0 rounded-lg px-3 py-2 text-sm break-words bg-gray-100 text-gray-900':
            {},
        },
        /** Icon control inside assistant bubbles (copy, etc.). */
        '.assistant-bubble-action': {
          '@apply shrink-0 rounded-md p-1.5 text-gray-500 hover:bg-gray-100 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-400/40':
            {},
        },
        '.assistant-bubble-thinking-shell': {
          '@apply max-w-[80%] rounded-lg px-3 py-2.5 bg-gray-100 text-gray-600 text-sm': {},
        },
        /** Send / stop control in assistant composer textarea. */
        '.assistant-composer-send': {
          '@apply !rounded-full absolute bottom-3 right-3 z-10 flex h-8 w-8 shrink-0 items-center justify-center !p-0 border border-gray-300 bg-white text-black shadow-sm transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-white focus:ring-gray-400/60 enabled:hover:bg-gray-50 enabled:hover:border-gray-400 disabled:cursor-not-allowed disabled:opacity-50':
            {},
        },
        /** Assistant tool-step: `<details>` blocks (batched semantic search UI). */
        '.assistant-fold-details': {
          '@apply rounded border border-gray-200 bg-gray-50/80 px-2 py-1': {},
        },
        '.assistant-fold-details-nested': {
          '@apply rounded border border-gray-100 bg-white px-2 py-1 text-xs': {},
        },
        '.assistant-fold-summary': {
          '@apply cursor-pointer list-none select-none text-xs font-medium text-gray-700 hover:underline [&::-webkit-details-marker]:hidden':
            {},
        },
        '.assistant-fold-summary-row': {
          '@apply flex cursor-pointer list-none flex-wrap items-baseline gap-x-1 gap-y-0 select-none text-gray-800 hover:underline [&::-webkit-details-marker]:hidden':
            {},
        },
        '.assistant-fold-summary-warning': {
          '@apply cursor-pointer list-none select-none font-medium text-amber-900 hover:underline [&::-webkit-details-marker]:hidden':
            {},
        },
        '.assistant-fold-details-warning': {
          '@apply rounded border border-amber-100 bg-amber-50/60 px-2 py-1 text-xs': {},
        },
        /** Message / thread list row (discussions, mail). Pair with `--clickable` when the row navigates on click. */
        '.message-thread-card': {
          '@apply bg-white border border-blue-200 rounded-lg transition-colors shadow-sm': {},
          wordBreak: 'break-word',
          '&:hover': {
            '@apply border-blue-300': {},
            boxShadow: '0 2px 8px rgba(0, 0, 0, 0.05)',
          },
        },
        '.message-thread-card--clickable': {
          '@apply cursor-pointer': {},
        },
        /** Primary blue title inside message-thread-card / lists. */
        '.link-message-title': {
          '@apply text-lg font-semibold text-blue-700 hover:text-blue-800 hover:underline': {},
        },
        /** Inline nav / definition heading (truncated). */
        '.link-heading-primary': {
          '@apply text-base font-semibold flex-shrink-0 min-w-0 max-w-full truncate text-blue-700 hover:text-blue-800 hover:underline':
            {},
        },
        // --- Auth & empty states ---
        /** Blue glass panel: password reset, change password, similar auth flows. */
        '.auth-glass-card': {
          '@apply w-full max-w-md p-8 mx-auto rounded-2xl border border-blue-200 flex-shrink-0 backdrop-blur-xl bg-blue-50/90 shadow-lg transition-all duration-300 hover:shadow-xl flex flex-col items-center space-y-6':
            {},
        },
        /** Frosted white auth card: login, signup. */
        '.auth-form-card': {
          '@apply card-elevated flex w-full max-w-md flex-shrink-0 flex-col items-center rounded-2xl border border-gray-200/90 bg-white/95 p-8 ring-1 ring-gray-900/5 backdrop-blur-sm':
            {},
        },
        /** Centered column for login / signup (frosted card). */
        '.auth-page-shell': {
          '@apply flex min-h-full w-full flex-col items-center justify-center px-4 py-10 sm:px-6 sm:py-12':
            {},
        },
        /** Centered column for glass auth flows (reset, change password). */
        '.auth-glass-page-shell': {
          '@apply relative flex w-full min-h-[calc(100vh-12rem)] items-center justify-center': {},
        },
        /** Frosted auth card (`AuthFormCard`) main heading. */
        '.auth-form-title': {
          '@apply mb-6 text-center text-2xl font-bold text-gray-900 sm:text-3xl': {},
        },
        /** Glass auth card headings (`AuthGlassCard`). */
        '.auth-glass-title': {
          '@apply text-center text-2xl font-bold text-blue-900 sm:text-3xl': {},
        },
        '.auth-glass-title--spaced': {
          '@apply mb-6 w-full': {},
        },
        /** In-page H1-style title (tools, admin). */
        '.page-section-title': {
          '@apply text-2xl font-bold text-gray-800': {},
        },
        /** Full-width primary submit under auth forms; add `ui-btn--*` separately. */
        '.auth-form-wide-submit': {
          '@apply w-full flex justify-center items-center gap-2 py-3 rounded-full text-lg font-semibold transition-all disabled:opacity-50 disabled:cursor-not-allowed':
            {},
        },
        /** Centered empty / zero-state block (lists, collections). */
        '.empty-state-panel': {
          '@apply flex flex-col items-center justify-center text-center py-12 bg-gray-50 rounded-lg border border-blue-100':
            {},
        },
        // --- Dropdowns & filters ---
        /** Teleported dropdown menu panel (FAB, ellipsis menus). */
        '.dropdown-menu-panel': {
          '@apply fixed z-50 w-fit min-w-0 max-w-[calc(100vw-1rem)] overflow-y-auto bg-white border border-gray-200 rounded-lg shadow-lg py-1':
            {},
        },
        /** Anchored multiselect / popover list under a trigger. */
        '.dropdown-floating-panel': {
          '@apply absolute left-0 right-0 z-50 mt-1 flex min-h-0 flex-col rounded-lg border border-gray-200 bg-white py-2 shadow-lg':
            {},
        },
        /** Default trigger for `Dropdown.vue` (ellipsis). */
        '.dropdown-ellipsis-trigger': {
          '@apply w-full sm:w-auto h-9 px-3 hover:bg-gray-100 rounded-full inline-flex items-center justify-between sm:justify-center gap-2 shrink-0':
            {},
        },
        /** Top strip on CombinedFilters (language row). */
        '.filters-bar-row': {
          '@apply flex flex-col sm:flex-row items-center sm:justify-between gap-4 md:p-4 md:bg-white md:rounded-lg md:shadow-sm':
            {},
        },
        /** Compact definition preview card (list rows). */
        '.surface-definition-compact': {
          '@apply w-full bg-white border rounded-lg hover:border-blue-300 transition-colors shadow hover:shadow-none p-4':
            {},
        },
        /** Activity “thread” row with blue hover border. */
        '.surface-activity-row': {
          '@apply space-y-2 bg-white p-4 rounded-lg border border-gray-200 hover:border-blue-200 transition-colors cursor-pointer':
            {},
        },
        /** Indented quote / snippet in activity lists. */
        '.activity-quote-snippet': {
          '@apply text-sm text-gray-600 border-l-2 border-gray-300 pl-2 [&_img]:max-h-48 [&_img]:object-contain':
            {},
        },
        /** Small “streak / correct” pill (flashcard / study UI). */
        '.badge-streak-success': {
          '@apply flex items-center justify-center gap-1 text-xs text-green-600 bg-green-50 border border-green-200 rounded px-3 py-1.5':
            {},
        },
        '.badge-streak-success--compact': {
          '@apply flex items-center justify-center gap-1 text-[10px] text-green-600 bg-green-50 border border-green-200 rounded px-1.5 py-0.5':
            {},
        },
        // --- Study & lingo ---
        /** Flashcard study: quiz multiple-choice tiles (text + image). */
        '.study-quiz-option-flashcard-text': {
          '@apply cursor-pointer rounded-xl border-2 border-gray-200 bg-white p-4 text-left text-base font-medium text-gray-800 transition-colors duration-200 hover:border-blue-400 hover:bg-blue-50/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60':
            {},
        },
        '.study-quiz-option-flashcard-image': {
          '@apply flex min-h-[100px] cursor-pointer flex-col items-center justify-center rounded-xl border-2 border-gray-200 bg-white p-3 transition-colors duration-200 hover:border-blue-400 hover:bg-blue-50/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60':
            {},
        },
        /** Lingo compact study: same interaction, green accent + smaller type. */
        '.study-quiz-option-lingo-text': {
          '@apply cursor-pointer rounded-xl border-2 border-slate-200 bg-white p-2.5 text-left text-xs font-medium text-gray-800 transition-colors duration-200 hover:border-green-400 hover:bg-green-50/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60 sm:text-sm':
            {},
        },
        '.study-quiz-option-lingo-image': {
          '@apply flex min-h-[80px] cursor-pointer flex-col items-center justify-center rounded-xl border-2 border-slate-200 bg-white p-2 transition-colors duration-200 hover:border-green-400 hover:bg-green-50/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60':
            {},
        },
        /** Lingo study: main lesson card (Duolingo-style bottom border). */
        '.lingo-study-card-shell': {
          '@apply flex min-h-0 flex-1 shrink-0 flex-col overflow-hidden rounded-xl border-2 border-b-4 border-neutral-200 bg-white p-2 pb-2 shadow-sm transition-colors duration-200 hover:bg-black/5 sm:p-2.5 sm:pb-2.5':
            {},
        },
        /** Lingo study header row (tap highlight off for mobile). */
        '.lingo-study-header-bar': {
          '-webkit-tap-highlight-color': 'transparent',
          '@apply sticky top-0 z-40 mx-auto flex w-full max-w-[1140px] items-center gap-x-2 border-b border-slate-200 bg-white px-3 py-2 sm:px-4 sm:py-2.5 lg:px-6 lg:py-3':
            {},
        },
        /** Lingo header back control. */
        '.icon-btn-header-back': {
          '@apply flex h-10 w-10 shrink-0 cursor-pointer items-center justify-center rounded-lg text-slate-500 transition-colors duration-200 hover:bg-slate-100 focus:outline-none focus:ring-2 focus:ring-green-500':
            {},
        },
        '.icon-btn-header-back--compact': {
          '@apply h-9 w-9': {},
        },
        /** Comment composer shell. */
        '.surface-comment-form': {
          '@apply mt-3 mb-6 bg-white border rounded-lg p-3 hover:border-blue-300 transition-colors relative':
            {},
        },
        /** Flashcard collection summary row. */
        '.surface-flashcard-summary': {
          '@apply bg-white p-4 rounded-lg border hover:border-blue-300 shadow hover:shadow-none transition-all duration-200 max-w-full overflow-hidden':
            {},
        },
        /** Mail thread attachment row chip. */
        '.attachment-chip': {
          '@apply px-3 py-1.5 bg-gray-50 rounded-lg border border-gray-200 hover:border-blue-200 transition-colors flex items-center gap-2':
            {},
        },
        /** Avatar placeholder (list size ~3rem). */
        '.avatar-placeholder-sm': {
          '@apply w-12 h-12 rounded-full bg-gray-50 flex items-center justify-center text-gray-400 border border-gray-100 shadow-sm':
            {},
        },
        /** Avatar upload target (profile, ~8rem). */
        '.avatar-placeholder-lg': {
          '@apply w-32 h-32 rounded-full bg-gray-200 flex items-center justify-center text-gray-400 border-4 border-white shadow-lg':
            {},
        },
        /** Collection cover on list cards: fixed square; image uses `.collection-cover-thumb` (contain, no crop). */
        '.collection-card-logo': {
          '@apply flex h-12 w-12 sm:h-16 sm:w-16 shrink-0 items-center justify-center rounded-lg sm:rounded-xl border border-gray-100 bg-white shadow-sm overflow-hidden':
            {},
        },
        '.collection-card-logo-placeholder': {
          '@apply flex w-12 sm:w-16 min-h-0 rounded-lg sm:rounded-xl bg-gray-50 items-center justify-center text-gray-400 border border-gray-100 shadow-sm shrink-0':
            {},
        },
        /** Collection detail header: fixed width, height follows hint+title row (`items-stretch`). */
        '.collection-header-logo': {
          '@apply box-border flex h-full min-h-0 w-max max-w-24 shrink-0 items-center justify-center rounded-lg sm:rounded-xl border border-gray-100 bg-white shadow-md overflow-hidden':
            {},
        },
        '.collection-header-logo-placeholder': {
          '@apply box-border flex h-full min-h-0 w-16 shrink-0 items-center justify-center rounded-lg sm:rounded-xl bg-gray-50 text-gray-400 border border-gray-100 shadow-md md:w-24 lg:w-28':
            {},
        },
        /** Image inside collection thumbs: fill frame without cropping (letterboxing as needed). */
        '.collection-cover-thumb': {
          '@apply h-full w-full max-h-full max-w-full object-contain': {},
        },
        /** Collection header cover: full row height, intrinsic width (contain, no crop). */
        '.collection-header-cover-thumb': {
          '@apply block h-full w-auto max-h-full max-w-full object-contain': {},
        },
        /**
         * Default `PageHeader` card shell: mobile-only full-bleed (cancels `#main-child` `pt-3` + inner `px-3`
         * from `App.vue`), square corners; from `sm` restores inset + `rounded-lg`.
         */
        '.page-header-shell': {
          '@apply -mx-3 -mt-3 rounded-none border border-gray-200 bg-white p-4 sm:p-6 sm:mx-0 sm:mt-0 sm:rounded-lg':
            {},
        },
        /**
         * Flashcard study session strip under the app bar (`border-t-0`, bottom radius only from `sm`).
         * Same mobile horizontal / top bleed as `.page-header-shell`.
         */
        '.page-header-study-strip': {
          '@apply -mx-3 -mt-3 rounded-none border border-gray-200 border-t-0 bg-white p-3 mb-4 sm:mx-0 sm:mt-0 sm:rounded-b-lg':
            {},
        },
        /** Collection cover in edit modal (matches profile avatar target size, square). */
        '.collection-edit-logo': {
          '@apply flex h-28 w-28 items-center justify-center rounded-xl border-4 border-white bg-white shadow-lg overflow-hidden':
            {},
        },
        '.collection-edit-logo-placeholder': {
          '@apply w-28 h-28 rounded-xl bg-gray-200 flex items-center justify-center text-gray-400 border-4 border-white shadow-lg':
            {},
        },
        /** Logo SVG wrapper in app header. */
        '.logo-svg-container': {
          '@apply flex h-full w-full items-center justify-center [&>svg]:block [&>svg]:h-full [&>svg]:w-full [&>svg]:max-h-full [&>svg]:max-w-full':
            {},
        },
        // --- Forms, toolbar & media controls ---
        '.input-field': {
          '@apply px-4 py-1.5 text-sm h-8 text-gray-700 bg-white border border-gray-300 placeholder-blue-400 rounded-full transition-all focus:ring-1 focus:ring-blue-500 focus:border-blue-500 focus:outline-none focus:z-50 whitespace-nowrap shadow-inner shadow-slate-200':
            {},
          '&:hover:not(:disabled)': {
            '@apply border-blue-400': {},
          },
          '&::placeholder': {
            '@apply text-gray-400': {},
          },
          '&:disabled': {
            '@apply bg-gray-100 cursor-not-allowed opacity-75': {},
          },
        },
        /** Native `<select>`: vertical padding stacks with fixed `h-8` and clips option text — keep full height for the control chrome. */
        'select.input-field': {
          '@apply py-0': {},
        },
        /** Decorative icon inside `relative` input wrapper (user, mail, key). */
        '.input-field-trailing-icon': {
          '@apply pointer-events-none absolute right-3 top-1/2 h-5 w-5 -translate-y-1/2 shrink-0 text-gray-400':
            {},
        },
        /** Password visibility toggle in auth forms; pair with `input-field` + `pr-10`. */
        '.input-field-password-toggle': {
          '@apply absolute right-2 top-1/2 flex h-8 w-8 -translate-y-1/2 cursor-pointer items-center justify-center rounded-md text-gray-400 transition-colors duration-200 hover:bg-gray-100 hover:text-gray-600 focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-500/50 disabled:cursor-not-allowed disabled:opacity-60':
            {},
        },
        /** CSV / file picker control label (focus ring on inner input). */
        '.file-input-label': {
          '@apply relative cursor-pointer rounded-md bg-white font-medium text-blue-600 transition-colors duration-200 hover:text-blue-500 focus-within:outline-none focus-within:ring-2 focus-within:ring-blue-500 focus-within:ring-offset-2':
            {},
        },
        '.btn-panel-primary': {
          '@apply flex-1 cursor-pointer rounded-lg bg-blue-600 px-3 py-2.5 font-medium text-white transition-colors duration-200 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2':
            {},
        },
        '.btn-panel-outline': {
          '@apply flex-1 cursor-pointer rounded-lg border border-gray-300 bg-white px-3 py-2.5 font-medium text-gray-700 transition-colors duration-200 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2':
            {},
        },
        /** Closed trigger for toolbar dropdowns (search mode, language multiselect): matches input-field visual language at h-10. */
        '.dropdown-trigger': {
          '@apply w-full h-10 min-h-[2.5rem] shrink-0 flex items-center justify-between gap-2 px-3 text-left text-sm font-normal text-gray-700 bg-white border border-gray-300 rounded-full transition-all focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 focus:z-50 shadow-inner shadow-slate-200':
            {},
          '&:hover:not(:disabled)': {
            '@apply border-blue-400': {},
          },
        },
        /** Compact toolbar panel used by list filters and controls. */
        '.toolbar-panel': {
          '@apply bg-white border border-gray-200 rounded-xl shadow-sm p-3 sm:p-4': {},
        },
        /** Inline labels beside compact toolbar controls. */
        '.toolbar-control-label': {
          '@apply text-sm font-medium text-gray-700 whitespace-nowrap': {},
        },
        /** Main flex row inside toolbar-panel (search + filters). */
        '.toolbar-row': {
          '@apply flex flex-wrap items-center gap-3 sm:gap-4': {},
        },
        /** Search column: full width on small screens, bounded width on sm+. */
        '.toolbar-search-slot': {
          '@apply w-full sm:w-auto sm:min-w-[220px] sm:max-w-[280px] flex-1 sm:flex-initial': {},
        },
        /** Label + control cluster (role filter, sort group). */
        '.toolbar-field-row': {
          '@apply flex w-auto max-w-full min-w-0 shrink-0 items-center gap-2': {},
          '& > .toolbar-control-label': {
            '@apply w-auto shrink-0': {},
          },
        },
        /** Positioning wrapper for Dropdown root. */
        '.toolbar-dropdown-anchor': {
          '@apply relative shrink-0': {},
        },
        /** Separate toolbar controls (e.g. sort field + order) with normal spacing between. */
        '.toolbar-inline-actions': {
          '@apply flex w-auto max-w-full min-w-0 shrink-0 flex-wrap items-center gap-2': {},
        },
        /** Leading segment of the home search bar (flush join with the query input). */
        '.dropdown-trigger--search-bar-leading': {
          '@apply w-auto max-w-none rounded-l-full rounded-r-none': {},
        },
        /** Trailing query column: below mode selector (z-10) at rest; stacks above on hover/focus/active of the field. */
        '.search-form-query-col': {
          '@apply relative z-0 flex-1 -ml-px min-w-0': {},
          '&:hover, &:focus-within, &:has(:active)': {
            '@apply z-20': {},
          },
        },
        /** Label above fields in CombinedFilters advanced grid (aligned spacing vs. inputs and dropdowns). */
        '.filters-field-label': {
          '@apply block text-sm font-medium text-gray-700 mb-2 leading-snug': {},
        },
        '.input-group': {
          '@apply flex items-stretch w-full': {},
          '& .input-field': {
            '@apply flex-1 min-w-0': {},
            '&:first-child:not(:last-child)': {
              '@apply rounded-r-none border-r-0': {},
            },
            '&:last-child:not(:first-child)': {
              '@apply rounded-l-none border-l-0': {},
            },
            '&:not(:first-child):not(:last-child)': {
              '@apply rounded-none border-r-0': {},
            },
          },
        },
        '.textarea-field': {
          '@apply w-full px-3 py-2 bg-white border border-gray-300 rounded-md text-sm placeholder-gray-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all focus:outline-none shadow-inner shadow-slate-200':
            {},
          '&:disabled': {
            '@apply bg-gray-100 cursor-not-allowed opacity-75': {},
          },
        },
        // Aqua theme primitives (`tailwind.aqua-buttons.mjs`)
        ...buildAquaButtonPrimitives(),
        ...buildAquaButtonGroupLayer(),
        // Flat theme primitives (gradient layers sit behind text — see `tailwind.flat-buttons.mjs`)
        ...buildFlatButtonLayer(theme),
        '.checkbox-toggle': {
          '@apply w-6 h-6 text-blue-600 border-gray-300 rounded focus:ring-blue-500 cursor-pointer transition-colors':
            {},
          '&:hover': {
            '@apply border-blue-400': {},
          },
        },
        '.checkmark-aqua': {
          '@apply p-0 justify-center cursor-pointer': {},
          // appearance: 'none',
          background: "url('/assets/icons/check.svg') no-repeat center",
          backgroundSize: '0%',
          border: '0.75px solid #333',
          borderRadius: '0.25rem',
          width: '16px',
          height: '16px',
          transition: 'background-size 0.1s ease-in-out',
          '&:checked': {
            backgroundSize: '16px',
            border: 0,
            backgroundColor: '#fff',
          },
          whiteSpace: 'nowrap',
          textOverflow: 'ellipsis',
          gridRow: '1',
          position: 'relative',
          overflow: 'hidden',
          cursor: 'default',
          outline: 'none',
        },
        // --- Cards, badges & streak UI ---
        '.card-base': {
          '@apply bg-white rounded-2xl border border-gray-200 overflow-hidden transition-all duration-200 h-full flex flex-col min-h-0':
            {},
        },
        '.card-elevated': {
          boxShadow: '0 0.75px 3px rgba(0, 0, 0, 0.04), 0 6px 16px rgba(0, 0, 0, 0.06)',
          '&:hover': {
            boxShadow: '0 4px 12px rgba(0, 0, 0, 0.06), 0 12px 28px rgba(0, 0, 0, 0.08)',
            '@apply border-gray-300': {},
          },
        },
        '.card-body': {
          '@apply flex-1 min-h-0 grid p-5 sm:p-6': {},
          gridTemplateRows: 'auto 1fr auto',
        },
        '.card-header-wrap': {
          '@apply min-h-0 overflow-auto': {},
        },
        '.card-footer-spacer': {
          '@apply min-h-0': {},
        },
        '.card-footer': {
          '@apply flex flex-col min-h-0 flex-shrink-0': {},
        },
        '.card-footer-block': {
          '@apply flex flex-col flex-shrink-0': {},
        },
        '.card-actions': {
          '@apply flex flex-wrap items-center justify-center gap-2 pt-4 flex-shrink-0': {},
        },
        '.card-compact .card-actions': {
          '@apply pt-3': {},
        },
        /** Title next to a thumbnail (e.g. collection card); multi-line clamp, no single-line truncate. */
        '.card-title--multiline': {
          '@apply text-lg font-semibold text-gray-800 min-w-0 leading-snug line-clamp-4 transition-colors':
            {},
          '&:hover': { '@apply text-blue-600': {} },
          '&:focus': { '@apply text-blue-600 outline-none': {} },
        },
        '.card-description': {
          '@apply text-gray-600 text-sm line-clamp-3 leading-snug': {},
        },
        '.card-footer-inner': {
          '@apply flex flex-wrap items-center justify-between gap-2': {},
        },
        '.badge': {
          '@apply inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium': {},
        },
        '.badge-muted': {
          '@apply bg-gray-50 text-gray-500': {},
        },
        '.card-meta': {
          '@apply flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-gray-500 shrink-0': {},
        },
        '.card-meta-by': {
          '@apply flex items-center gap-1': {},
        },
        '.card-meta-link': {
          '@apply font-medium text-blue-600 hover:text-blue-800 hover:underline': {},
        },
        '.card-meta-date': {
          '@apply text-gray-400': {},
        },
        /** Study streak strip (collections): seven columns that shrink in flex layouts; larger type than generic card-meta-date. */
        '.card-streak': {
          '@apply w-full min-w-0 max-w-full': {},
        },
        '.card-streak-header': {
          '@apply mb-2 flex min-w-0 flex-wrap items-center justify-between gap-x-2 gap-y-1': {},
        },
        '.card-streak-title': {
          '@apply min-w-0 flex-1 text-base font-semibold text-gray-800 sm:flex-none': {},
        },
        '.card-streak-meta': {
          '@apply card-meta min-w-0 sm:shrink-0': {},
        },
        '.card-streak-week-grid': {
          '@apply grid w-full min-w-0 gap-1 sm:gap-2': {},
          gridTemplateColumns: 'repeat(7, minmax(0, 1fr))',
        },
        '.card-streak-day': {
          '@apply flex h-[6rem] min-h-0 min-w-0 flex-col items-center justify-center gap-1 overflow-hidden text-center sm:h-[7.25rem]':
            {},
        },
        '.card-streak-day-label': {
          '@apply line-clamp-2 w-full px-0.5 text-xs font-medium leading-tight text-gray-500 break-words sm:text-sm':
            {},
        },
        '.card-streak-day-points': {
          '@apply line-clamp-2 w-full px-0.5 text-xs font-medium leading-tight text-gray-400 break-words sm:text-sm':
            {},
        },
        '.card-streak-day-count': {
          '@apply flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-xs font-medium sm:h-8 sm:w-8 sm:text-sm':
            {},
        },
        /** Pulse bars: heights match `.card-streak-day-label` / `.card-streak-day-points` line boxes (no CLS vs loaded streak). */
        '.card-streak-skeleton-line': {
          '@apply mx-auto h-[0.9375rem] w-full max-w-[3.5rem] shrink-0 rounded bg-gray-100 sm:h-[1.09375rem]':
            {},
        },
        '.card-streak-skeleton-line--points': {
          '@apply max-w-[2.25rem]': {},
        },
        '.card-study-area': {
          '@apply flex items-center justify-center min-h-[5.5rem] py-6 sm:py-8 flex-shrink-0': {},
        },
        // --- Tab strips (aqua vs flat styled in `buttonUiThemeLayer`) ---
        /**
         * Page tab row (`TabbedPageHeader` / `NotebookTab`). Aqua: Cheetah GTK notebook strip + tabs;
         * flat: underline tabs (legacy).
         */
        '.ui-tab-strip': {
          '@apply flex flex-wrap gap-x-2 gap-y-0': {},
        },
        '.ui-tab': {
          '@apply inline-flex items-center gap-2 font-medium transition-[color,background,box-shadow,border-color,filter] duration-200 ease-out':
            {},
          '&:focus-visible': {
            '@apply outline outline-2 outline-offset-2 outline-[#3B82D6]': {},
          },
        },
      })

      const buttonUiThemeLayer = () => {
        const rules = {}
        const themeSelectors = {
          aqua: 'html[data-button-theme="aqua"] .CLASS',
          flat: 'html[data-button-theme="flat"] .CLASS, html:not([data-button-theme]) .CLASS',
        }
        const selectorFor = (theme, cls) => themeSelectors[theme].replace(/CLASS/g, cls)
        const addThemeMap = (themeMap) => {
          for (const [cls, classesByTheme] of Object.entries(themeMap)) {
            for (const [theme, primitives] of Object.entries(classesByTheme)) {
              rules[selectorFor(theme, cls)] = { [`@apply ${primitives}`]: {} }
            }
          }
        }
        /**
         * Semantic button **roles** (flat + aqua primitives via `@apply`).
         * Canonical names: `read`, `edit`, `create`, `remove`, `dismiss`, `back`, `forward`, `toolbar`, …
         * Legacy synonyms stay in the map so older strings keep working (`get`≈`read`, `update`≈`edit`, …).
         * Authoritative human table: `frontend/docs/brandbook.md` §6.3.
         */
        const buttonThemeClassMap = {
          'ui-btn--accent-purple': { aqua: 'btn-aqua-purple', flat: 'btn-link' },
          'ui-btn--action': { aqua: 'btn-aqua-pink', flat: 'btn-action' },
          'ui-btn--amber': { aqua: 'btn-aqua-amber', flat: 'btn-warning' },
          /** Flat: neutral embossed `btn-empty` (not blue `btn-get`) — filter toggles, toolbars. */
          'ui-btn--aqua-default': { aqua: 'btn-aqua-simple', flat: 'btn-empty' },
          /** Role: default toolbar / filter chrome — synonym of `aqua-default`. */
          'ui-btn--toolbar': { aqua: 'btn-aqua-simple', flat: 'btn-empty' },
          'ui-btn--auth-login': { aqua: 'btn-aqua-orange', flat: 'btn-warning' },
          'ui-btn--auth-signup': { aqua: 'btn-aqua-teal', flat: 'btn-create' },
          'ui-btn--cancel': { aqua: 'btn-aqua-zinc', flat: 'btn-cancel' },
          /** Role: dismiss modal / abandon branch — synonym of `cancel`. */
          'ui-btn--dismiss': { aqua: 'btn-aqua-zinc', flat: 'btn-cancel' },
          'ui-btn--continue': { aqua: 'btn-aqua', flat: 'btn-get' },
          'ui-btn--create': { aqua: 'btn-aqua-emerald', flat: 'btn-create' },
          'ui-btn--danger-rose': { aqua: 'btn-aqua-rose', flat: 'btn-delete' },
          'ui-btn--delete': { aqua: 'btn-aqua-red', flat: 'btn-delete' },
          /** Role: destructive remove — synonym of `delete`. */
          'ui-btn--remove': { aqua: 'btn-aqua-red', flat: 'btn-delete' },
          'ui-btn--empty': { aqua: 'btn-aqua-white', flat: 'btn-empty' },
          'ui-btn--error': { aqua: 'btn-aqua-red', flat: 'btn-error' },
          /** Role: read / open / navigate / neutral forward — canonical; `get` is legacy synonym. */
          'ui-btn--read': { aqua: 'btn-aqua-get', flat: 'btn-get' },
          'ui-btn--get': { aqua: 'btn-aqua-get', flat: 'btn-get' },
          'ui-btn--group-item': { aqua: 'btn-aqua-group-item', flat: 'btn-group-item' },
          'ui-btn--history': { aqua: 'btn-aqua-purple', flat: 'btn-history' },
          'ui-btn--insert': { aqua: 'btn-aqua-blue', flat: 'btn-insert' },
          'ui-btn--link': { aqua: 'btn-aqua-sky', flat: 'btn-link' },
          'ui-btn--market': { aqua: 'btn-aqua-rose', flat: 'btn-market' },
          'ui-btn--neutral': { aqua: 'btn-aqua-white', flat: 'btn-get' },
          'ui-btn--neutral-muted': { aqua: 'btn-aqua-zinc', flat: 'btn-cancel' },
          'ui-btn--neutral-slate': { aqua: 'btn-aqua-slate', flat: 'btn-get' },
          'ui-btn--next': { aqua: 'btn-aqua-zinc', flat: 'btn-next' },
          /** Role: pagination / sequence forward — synonym of `next`. */
          'ui-btn--forward': { aqua: 'btn-aqua-zinc', flat: 'btn-next' },
          'ui-btn--palette-amber': { aqua: 'btn-aqua-amber', flat: 'btn-palette-amber' },
          'ui-btn--palette-blue': { aqua: 'btn-aqua-blue', flat: 'btn-palette-blue' },
          'ui-btn--palette-cyan': { aqua: 'btn-aqua-cyan', flat: 'btn-palette-cyan' },
          'ui-btn--palette-emerald': { aqua: 'btn-aqua-emerald', flat: 'btn-palette-emerald' },
          'ui-btn--palette-fuchsia': { aqua: 'btn-aqua-fuchsia', flat: 'btn-palette-fuchsia' },
          'ui-btn--palette-indigo': { aqua: 'btn-aqua-indigo', flat: 'btn-palette-indigo' },
          'ui-btn--palette-lime': { aqua: 'btn-aqua-lime', flat: 'btn-palette-lime' },
          'ui-btn--palette-orange': { aqua: 'btn-aqua-orange', flat: 'btn-palette-orange' },
          'ui-btn--palette-pink': { aqua: 'btn-aqua-pink', flat: 'btn-palette-pink' },
          'ui-btn--palette-purple': { aqua: 'btn-aqua-purple', flat: 'btn-palette-purple' },
          'ui-btn--palette-red': { aqua: 'btn-aqua-red', flat: 'btn-palette-red' },
          'ui-btn--palette-rose': { aqua: 'btn-aqua-rose', flat: 'btn-palette-rose' },
          'ui-btn--palette-sky': { aqua: 'btn-aqua-sky', flat: 'btn-palette-sky' },
          'ui-btn--palette-slate': { aqua: 'btn-aqua-slate', flat: 'btn-palette-slate' },
          'ui-btn--palette-teal': { aqua: 'btn-aqua-teal', flat: 'btn-palette-teal' },
          'ui-btn--palette-violet': { aqua: 'btn-aqua-violet', flat: 'btn-palette-violet' },
          'ui-btn--palette-white': { aqua: 'btn-aqua-white', flat: 'btn-palette-white' },
          'ui-btn--palette-yellow': { aqua: 'btn-aqua-yellow', flat: 'btn-palette-yellow' },
          'ui-btn--palette-zinc': { aqua: 'btn-aqua-zinc', flat: 'btn-palette-zinc' },
          'ui-btn--previous': { aqua: 'btn-aqua-zinc', flat: 'btn-previous' },
          /** Role: pagination / go back — synonym of `previous`. */
          'ui-btn--back': { aqua: 'btn-aqua-zinc', flat: 'btn-previous' },
          'ui-btn--primary': { aqua: 'btn-aqua-emerald', flat: 'btn-create' },
          'ui-btn--reaction': { aqua: 'btn-aqua-cyan', flat: 'btn-reaction' },
          'ui-btn--reaction-active': { aqua: 'btn-aqua-blue', flat: 'btn-reaction-active' },
          'ui-btn--reply': { aqua: 'btn-aqua-sky', flat: 'btn-reply' },
          'ui-btn--revert': { aqua: 'btn-aqua-yellow', flat: 'btn-revert' },
          'ui-btn--sort-amber': { aqua: 'btn-aqua-amber', flat: 'btn-warning' },
          'ui-btn--sort-blue': { aqua: 'btn-aqua-blue', flat: 'btn-insert' },
          'ui-btn--sort-emerald': { aqua: 'btn-aqua-emerald', flat: 'btn-create' },
          'ui-btn--sort-sky': { aqua: 'btn-aqua-sky', flat: 'btn-success' },
          'ui-btn--study-correct': { aqua: 'btn-aqua-teal', flat: 'btn-success' },
          'ui-btn--study-wrong': { aqua: 'btn-aqua-rose', flat: 'btn-error' },
          'ui-btn--success': { aqua: 'btn-aqua-emerald', flat: 'btn-success' },
          'ui-btn--update': { aqua: 'btn-aqua-teal', flat: 'btn-update' },
          /** Role: save edits / apply changes — synonym of `update`. */
          'ui-btn--edit': { aqua: 'btn-aqua-teal', flat: 'btn-update' },
          'ui-btn--warning': { aqua: 'btn-aqua-orange', flat: 'btn-warning' },
          'ui-btn--warning-orange': { aqua: 'btn-aqua-orange', flat: 'btn-warning' },
          /** Flat: `btn-revert` (yellow) vs `btn-warning` (amber) so orange/yellow variants stay distinct. */
          'ui-btn--warning-yellow': { aqua: 'btn-aqua-yellow', flat: 'btn-revert' },
        }
        addThemeMap(buttonThemeClassMap)
        /** Selected/saved state for neutral buttons (e.g. bookmark): push from slate to light blue. */
        const activeNeutralMap = {
          aqua: 'btn-aqua-sky',
          flat: 'btn-get',
        }
        for (const [theme, primitives] of Object.entries(activeNeutralMap)) {
          rules[selectorFor(theme, 'ui-btn--empty.active')] = { [`@apply ${primitives}`]: {} }
        }
        /** Overflow menu row: outer drop shadow (aqua glossy inner shadow stays on the button primitive). */
        const aquaRules = {}
        const flatRules = {}
        /** Same rose/market primitives as `ui-btn--market`; larger touch target, pill, bigger type. */
        /** Inline `aqua-base` + fill (not `@apply btn-aqua-cornflower`): that primitive lives in an earlier
         * `addComponents` object; a second `addComponents(buttonUiThemeLayer)` pass cannot `@apply` it. */
        aquaRules[selectorFor('aqua', 'ui-btn--fab')] = {
          '@apply aqua-base bg-cornflower-400 !h-12 !w-12 cursor-pointer rounded-full !text-lg !text-white':
            {},
          '--aqua-hue': '219',
        }
        flatRules[selectorFor('flat', 'ui-btn--fab')] = {
          '@apply btn-fab !h-12 !w-12 cursor-pointer rounded-full !text-lg': {},
        }
        /** Flat FAB: no outer ring; soft cornflower-tinted float shadow (ui-btn--fab is borderless). */
        flatRules[selectorFor('flat', 'fab-elevation-shell')] = {
          '@apply ring-0': {},
          boxShadow:
            '0 12px 28px -10px rgba(61, 107, 196, 0.38), 0 8px 18px -10px rgba(100, 149, 237, 0.32)',
          '&:hover': {
            boxShadow:
              '0 16px 36px -12px rgba(61, 107, 196, 0.42), 0 10px 22px -10px rgba(100, 149, 237, 0.38)',
          },
        }
        /** Aqua FAB: elevation matches cornflower hue (replaces rose-tinted shell). */
        aquaRules[selectorFor('aqua', 'fab-elevation-shell')] = {
          boxShadow:
            '0 10px 26px -8px rgba(15, 23, 42, 0.16), 0 6px 14px -6px rgba(100, 149, 237, 0.32)',
          '&:hover': {
            boxShadow:
              '0 18px 36px -10px rgba(15, 23, 42, 0.2), 0 10px 22px -8px rgba(87, 137, 232, 0.38)',
          },
        }
        aquaRules[selectorFor('aqua', 'ui-btn--toggle.active')] = {
          '@apply aqua-base bg-red-400': {},
          '--aqua-hue': '0',
        }
        aquaRules[selectorFor('aqua', 'ui-btn--toggle:not(.active)')] = {
          '@apply btn-aqua-white': {},
          boxShadow: aquaToggleOffShadow,
        }
        flatRules[selectorFor('flat', 'ui-btn--toggle.active')] = {
          '@apply btn-error': {},
        }
        flatRules[selectorFor('flat', 'ui-btn--toggle:not(.active)')] = {
          '@apply btn-empty': {},
        }
        /** Beats semantic ui-btn--* primitives so inner segments stay joined in forced groups (all breakpoints). */
        aquaRules[selectorFor('aqua', 'btn-group-forced .ui-btn--group-item')] =
          buildAquaUiBtnGroupItemGeometry()
        flatRules[selectorFor('flat', 'btn-group-forced .ui-btn--group-item')] = {
          /** Full borders on every segment; `-ml-px` overlaps adjacent 1px borders; hover/focus z-index reveals full outline. */
          '@apply relative z-0 rounded-none border-r first:ml-0 -ml-px first:rounded-l-full last:rounded-r-full':
            {},
          '&:hover': {
            '@apply z-10': {},
          },
          '&:focus-visible': {
            '@apply z-10': {},
          },
        }
        /** Cheetah aqua: `notebook > header` + tab chrome (#F5F5F5 strip, #9ac7e6 band, GTK tab gradients). */
        aquaRules[selectorFor('aqua', 'ui-tab-strip')] = {
          backgroundColor: 'transparent',
          borderBottom: '4px solid #9ac7e6',
          boxShadow: '0 1px #848484, inset 0 -1px #3a5c8c',
          padding: '6px 6px 0',
        }
        flatRules[selectorFor('flat', 'ui-tab-strip')] = {
          '@apply bg-transparent border-0 shadow-none p-0': {},
        }
        aquaRules[selectorFor('aqua', 'ui-tab.ui-tab--inactive')] = {
          color: '#000000',
          backgroundImage: 'linear-gradient(to bottom, #d0d0d0 40%, #FFFFFF)',
          border: '1px solid #9a9a9a',
          borderBottom: 'none',
          borderRadius: '6px 6px 0 0',
          boxShadow: 'inset 0 6px 3px -2px #ffffff',
          margin: '1px',
          padding: '6px 12px',
          '&:hover': {
            filter: 'brightness(1.04)',
            borderColor: '#848484',
          },
        }
        aquaRules[selectorFor('aqua', 'ui-tab.ui-tab--active')] = {
          color: '#000000',
          backgroundColor: '#FFFFFF',
          backgroundImage: 'linear-gradient(to bottom, #63a1df 35%, #b9edff)',
          border: '1px solid #07067b',
          borderLeftColor: '#5397e7',
          borderRightColor: '#5397e7',
          borderTopColor: '#5397e7',
          borderBottom: 'none',
          borderRadius: '6px 6px 0 0',
          boxShadow: 'inset 0 4px 3px -2px #ffffff',
          margin: '1px',
          padding: '6px 12px',
          zIndex: '1',
        }
        flatRules[selectorFor('flat', 'ui-tab')] = {
          '@apply px-3 py-2 border-b-2': {},
        }
        flatRules[selectorFor('flat', 'ui-tab--active')] = {
          '@apply border-blue-500 text-blue-600': {},
        }
        flatRules[selectorFor('flat', 'ui-tab--inactive')] = {
          '@apply border-transparent text-gray-500': {},
          '&:hover': {
            '@apply text-gray-700 border-gray-300': {},
          },
        }
        Object.assign(rules, aquaRules, flatRules)
        return rules
      }
      addComponents(buttonUiThemeLayer())
    },
  ],
}
