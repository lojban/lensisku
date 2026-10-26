import { buildFlatButtonLayer, buildFlatToggleSwitchThemeRules } from './tailwind.flat-buttons.mjs'
import {
  aquaToggleOffShadow,
  buildAquaBaseLayer,
  buildAquaButtonGroupLayer,
  buildAquaButtonPrimitives,
  buildAquaToggleSwitchThemeRules,
  buildAquaUiBtnGroupItemGeometry,
} from './tailwind.aqua-buttons.mjs'
import { buildLingoThemeLayer } from './tailwind.lingo.mjs'

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
      keyframes: {
        'route-progress': {
          '0%': { transform: 'translateX(-100%)' },
          '100%': { transform: 'translateX(350%)' },
        },
      },
      animation: {
        'route-progress': 'route-progress 1.05s ease-in-out infinite',
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
        /**
         * Full-viewport loading veil (LoadingSpinner `page` variant).
         * Flat wash — no backdrop blur. The ring is the indicator.
         */
        '.page-loading-overlay': {
          '@apply fixed inset-0 z-50 flex min-h-0 flex-col items-center justify-center bg-white/80':
            {},
        },
        /**
         * In-region veil (LoadingSpinner `content` variant) — parent must be `relative`.
         * For query/filter refreshes; path changes use App’s route progress bar instead.
         */
        /** Skeleton primitives (PageSkeleton and page-specific skeletons). */
        '.skeleton-block': {
          '@apply block animate-pulse rounded bg-gray-200': {},
        },
        '.page-skeleton__card': {
          '@apply rounded-lg border border-gray-200 bg-white p-3 sm:p-4': {},
        },
        /** LoadingSpinner root: layout via `data-variant` (no JS class map). */
        '.loading-spinner[data-variant="page"]': {
          '@apply fixed inset-0 z-50 flex min-h-0 flex-col items-center justify-center bg-white/80':
            {},
        },
        '.loading-spinner[data-variant="content"]': {
          '@apply absolute inset-0 z-10 flex min-h-[8rem] flex-col items-center justify-center bg-white/60':
            {},
        },
        '.loading-spinner[data-variant="inline"]': {
          '@apply flex flex-col items-center justify-center': {},
        },
        /** Track plus a leading edge. Generic “Loading…” stays off screen (sr-only). */
        '.loading-spinner__mark': {
          '@apply h-10 w-10 shrink-0 animate-spin rounded-full border-2 border-green-500 border-t-transparent':
            {},
          /* Own compositor layer: keeps turning while the main thread is busy mounting a page. */
          'will-change': 'transform',
          /* Always full speed, even with prefers-reduced-motion (owner's call: a slowed/static
             indicator reads as "stuck"). No reduced-motion override on purpose. */
        },
        /* Lingo ring at three sizes (default md = 40px). */
        '.loading-spinner[data-size="sm"] .loading-spinner__mark': {
          '@apply h-5 w-5': {},
        },
        '.loading-spinner[data-size="lg"] .loading-spinner__mark': {
          '@apply h-14 w-14': {},
        },
        '.loading-spinner__label': {
          '@apply mt-3 text-sm font-medium text-slate-600': {},
        },
        /** Indeterminate bar on the viewport top edge, above the sticky header. */
        '.route-progress': {
          '@apply pointer-events-none fixed inset-x-0 top-0 z-[60] h-[3px] overflow-hidden bg-green-500/15':
            {},
        },
        '.route-progress__bar': {
          '@apply block h-full w-2/5 bg-green-500 animate-route-progress': {},
          'will-change': 'transform',
        },
        /** Sticky app chrome (logo, nav, auth). */
        '.app-header-bar': {
          '@apply sticky top-0 z-40 shrink-0 border-b border-gray-200 bg-white': {},
        },
        /**
         * Header + in-flow banners + main. Full viewport — no reserved strip for the
         * removed fixed live-chat footer (was `calc(100svh - 24px)` / `h-6`).
         */
        '.app-layout-column': {
          '@apply flex min-h-0 flex-col': {},
          height: '100vh',
          '@supports (height: 100svh)': {
            height: '100svh',
          },
        },
        /** Canonical vertical rhythm for normal page routes (applied directly in App.vue). */
        '.page-sections': {
          '@apply flex flex-col gap-4 sm:gap-5': {},
        },
        /**
         * Bottom inset on scrolling page columns so the fixed FAB (h-12 + bottom-4)
         * does not cover the last content. Skip on full-height / auth-fullbleed layouts.
         */
        '.page-fab-clearance': {
          '@apply pb-24': {},
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
        /**
         * FAB overflow menu icons — hues mirror semantic btn roles (nav / accent / reply / create).
         * Use with Lucide size props; keep stroke-width on the SVG.
         */
        '.fab-menu-icon': {
          '@apply h-4 w-4 shrink-0': {},
        },
        '.fab-menu-icon--assistant': {
          '@apply text-nav-link': {},
        },
        '.fab-menu-icon--graph': {
          '@apply text-cornflower-600': {},
        },
        /**
         * Wave / discuss glyphs (`WaveIcon`) when a hue is required. Purple — same as
         * “new wave” on comment threads. Do not put this on the icon when it sits inside
         * a colored button (inherit `currentColor`).
         */
        '.wave-icon': {
          '@apply text-purple-600': {},
        },
        '.fab-menu-icon--discussion': {
          '@apply text-purple-600': {},
        },
        /** Definition type and “create definition” (`DefinitionIcon`, `DefinitionCreateIcon`). */
        '.fab-menu-icon--definition': {
          '@apply text-emerald-600': {},
        },
        /** Add to a collection (`StarPlus`) in a plain menu. Amber — not edit yellow. */
        '.fab-menu-icon--collection': {
          '@apply text-amber-600': {},
        },
        /**
         * Export (`ExportIcon`, `FileExportIcon`) in a plain menu. Cyan — same family as
         * the read/export button (`btn-get`). Nav links inherit `currentColor` instead.
         */
        '.fab-menu-icon--export': {
          '@apply text-cyan-700': {},
        },
        /**
         * Entry (valsi) chips: book-with-A on blue. Distinct from definition emerald.
         * Page-title entry icons inherit `currentColor` — do not force this fill there.
         */
        '.entry-chip': {
          '@apply bg-blue-50 text-blue-700': {},
        },
        /** Definition chips: quoted gloss on emerald (same hue as create-definition). */
        '.definition-chip': {
          '@apply bg-emerald-50 text-emerald-700': {},
        },
        '.mail-chip': {
          '@apply bg-violet-50 text-violet-700': {},
        },
        '.jbotcan-chip': {
          '@apply bg-amber-50 text-amber-700': {},
        },
        '.freeforums-chip': {
          '@apply bg-orange-50 text-orange-700': {},
        },
        /**
         * Wiki glyphs where a hue is required (create menu, chips). Black — not the emerald
         * used for definitions. Do not put this on `WikiIcon` itself: tabs and other
         * chrome inherit `currentColor` (inactive gray, active tab color).
         */
        '.wiki-icon': {
          '@apply text-black': {},
        },
        '.fab-menu-icon--wiki': {
          '@apply text-black': {},
        },
        '.wiki-chip': {
          '@apply bg-gray-100 text-black': {},
        },
        /**
         * Footer modal: iOS-style toggle track + thumb (pair thumb with translate-x-0 / translate-x-5).
         * Use `Button variant="plain"` so aqua/flat `ui-btn--*` padding/chrome do not fight the track.
         * Theme gloss / emboss lives in `buttonUiThemeLayer` (aqua vs flat).
         */
        '.toggle-switch': {
          '@apply relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent bg-gray-200 p-0 transition-[background-color,box-shadow] duration-200 ease-in-out focus:outline-none focus-visible:ring-2 focus-visible:ring-nav-link focus-visible:ring-offset-2':
            {},
          '&[aria-pressed="true"]': {
            '@apply bg-nav-link': {},
          },
          /** `Button.vue` wraps the slot; keep the thumb free to translate inside the track. */
          '& > span': {
            '@apply !m-0 flex !h-full !w-full !items-center !justify-start !gap-0 !p-0 !whitespace-normal':
              {},
          },
        },
        '.toggle-switch-thumb': {
          '@apply pointer-events-none relative z-[1] inline-block h-5 w-5 shrink-0 rounded-full bg-white shadow transition duration-200 ease-in-out will-change-transform':
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
        /** In-flow strip under app header (`PageBanner`, email / test-data warnings). */
        '.page-banner-row': {
          '@apply relative z-10 mx-auto w-full shrink-0 border text-center select-none': {},
        },
        '.page-banner--warning': {
          '@apply w-full px-4 py-2 text-sm opacity-90 bg-yellow-100 border-yellow-300': {},
        },
        '.page-banner--error': {
          '@apply w-fit px-2 py-0 text-xs opacity-80 bg-red-100 border-red-200': {},
        },
        /**
         * Room for an aqua pill's outer drop shadow inside an overflow clip.
         * The shadow reaches about 0.5em sideways and 0.9em below the border box.
         */
        '.shadow-bounds': {
          '@apply overflow-visible px-3 pt-1 pb-3': {},
        },
        /** In-form note. Inset from the header rule; text stays off the box border. */
        '.form-callout': {
          '@apply mb-4 rounded-lg border border-blue-200 bg-blue-50 px-4 py-3 text-sm leading-relaxed text-blue-800':
            {},
        },
        '.form-callout--warning': {
          '@apply border-amber-200 bg-amber-50 text-amber-900': {},
        },
        '.form-callout--error': {
          '@apply mb-0 mt-3 border-red-200 bg-red-50 text-red-800': {},
        },
        /** `ModalComponent` default slot: scroll area. Padding inset keeps button box-shadows and focus rings inside the scrollport (they paint outside the border box and would otherwise clip). */
        '.modal-scroll-body': {
          '@apply min-h-0 flex-1 overflow-y-auto overscroll-contain px-2 pt-2 pb-6 sm:px-3 sm:pb-7':
            {},
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
        /**
         * Icon above label. Aqua pills are `h-6` + `overflow:hidden`, which otherwise clips stacked
         * content and lets the grid shrink the control below the label’s min-content width.
         */
        '.icon-btn-ui-layout--stacked': {
          '@apply !h-auto min-h-max min-w-min !rounded-md !py-1.5 !leading-snug !whitespace-normal [&>span]:flex-col [&>span]:items-center [&>span]:justify-center [&>span]:text-center [&>span]:leading-snug [&>span]:gap-0.5 [&>span]:whitespace-normal':
            {},
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
        /**
         * Two equal columns for the mobile drawer.
         * One 0.5rem rhythm for the column gap, the row gap, and the inset from the edges.
         */
        '.nav-menu-grid': {
          display: 'grid',
          alignItems: 'stretch',
          gridTemplateColumns: 'minmax(0, 1fr) minmax(0, 1fr)',
          gap: '0.5rem',
          paddingInline: '0.5rem',
        },
        /** Desktop More menu: one column, as wide as its contents. */
        '.nav-menu-stack': {
          display: 'grid',
          alignItems: 'stretch',
          gridTemplateColumns: 'minmax(0, 1fr)',
          gap: '0.5rem',
          paddingInline: '0.5rem',
        },
        '.nav-menu-grid__full': {
          '@apply col-span-2': {},
        },
        /** App mobile drawer row: layout + hover only — link color comes from `NavLink` (`text-nav-link` / `nav-link-active`). */
        '.mobile-nav-row': {
          '@apply flex min-w-0 cursor-pointer items-center gap-1.5 rounded-md py-2 text-base leading-snug transition-colors duration-200 hover:bg-gray-100 hover:text-blue-800 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-400 focus-visible:ring-offset-2 focus-visible:ring-offset-white [&>svg]:shrink-0':
            {},
        },
        /** Logout / primary action row in mobile nav. */
        '.mobile-nav-row--emphasis': {
          '@apply flex w-full cursor-pointer items-center gap-2 rounded-md px-4 py-2 text-left text-base text-nav-link transition-colors duration-200 hover:bg-gray-100 hover:text-blue-800 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-400 focus-visible:ring-offset-2 focus-visible:ring-offset-white':
            {},
        },
        /** Desktop More menu. Hangs from the trigger and sizes to its contents. */
        '.nav-dropdown-panel': {
          '@apply absolute left-0 z-50 mt-0 w-max max-w-96 flex-col': {},
        },
        '.nav-dropdown-sheet': {
          '@apply rounded-md border border-gray-200 bg-white py-2 shadow-lg': {},
        },
        /** Centered header column (logo, nav, auth). */
        '.app-header-inner': {
          '@apply mx-auto max-w-4xl px-1 sm:px-2': {},
        },
        /**
         * Language / theme select that fills its nav cell
         * (desktop More menu and the mobile drawer grid).
         */
        '.nav-select': {
          '@apply flex w-full min-w-0 items-center': {},
        },
        '.nav-select-field': {
          '@apply !h-6 !w-full !min-w-0 appearance-none !py-0 !pr-3 !text-xs': {},
        },
        /**
         * Language / theme field sized to its label (profile toolbar).
         * The menu variant (`.nav-select`) stays full width.
         */
        '.nav-select-inline': {
          '@apply w-max max-w-full shrink-0': {},
        },
        // --- Assistant (chat UI) ---
        /** Assistant chat: scrollable message column shell. */
        '.assistant-messages-pane': {
          '@apply relative min-h-0 flex-1 overflow-x-hidden rounded-lg border border-gray-200 bg-white [overscroll-behavior-y:contain]':
            {},
        },
        /** Assistant chat: sidebar chat search field. */
        '.assistant-input-search': {
          '@apply w-full rounded-lg border border-gray-200 bg-white py-2 pr-3 text-sm text-gray-900 placeholder:text-gray-400 focus:border-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400/30':
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
        /** Generic foldable tip panel (e.g. lujvo component definitions on add-entry). */
        '.fold-panel': {
          '@apply rounded-lg border border-blue-200 bg-blue-50 px-3 py-2 text-blue-800': {},
        },
        '.fold-panel-summary': {
          '@apply cursor-pointer list-none select-none text-sm font-medium text-blue-800 hover:underline [&::-webkit-details-marker]:hidden':
            {},
        },
        '.fold-panel-body': {
          '@apply mt-3 space-y-4 text-gray-800': {},
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
        /** Social login buttons under email/password auth forms. */
        '.auth-social-stack': {
          '@apply mt-6 flex w-full flex-col gap-3': {},
        },
        '.auth-social-divider': {
          '@apply flex w-full items-center gap-3 text-sm text-gray-500': {},
        },
        '.auth-social-divider-line': {
          '@apply h-px flex-1 bg-gray-200': {},
        },
        /** Centered empty / zero-state block (lists, collections). */
        '.empty-state-panel': {
          '@apply flex flex-col items-center justify-center text-center py-12 bg-gray-50 rounded-lg border border-blue-100':
            {},
        },
        /**
         * CommentList list region: reserved footprint so loading → empty / comments
         * does not CLS. Loading uses SkeletonActivityItem; empty fills the box.
         */
        '.comment-list-feed': {
          '@apply flex w-full min-h-[24rem] flex-col sm:min-h-[26rem]': {},
        },
        /** Embedded (e.g. Lingo modal): fixed height + internal scroll — no modal chrome CLS. */
        '.comment-list-feed--embedded': {
          '@apply h-[min(50vh,28rem)] min-h-[20rem] overflow-y-auto overscroll-contain': {},
        },
        '.comment-list-status-panel': {
          '@apply flex min-h-0 w-full flex-1 flex-col items-center justify-center gap-4 rounded-lg border border-blue-100 bg-blue-50 p-4 text-center':
            {},
        },
        '.comment-list-status-panel__icon': {
          '@apply h-12 w-12 shrink-0 text-blue-400': {},
        },
        '.comment-list-status-panel__text': {
          '@apply max-w-prose text-sm text-gray-600 sm:text-base': {},
        },
        // --- Dropdowns & filters ---
        /** Teleported dropdown menu panel (FAB, ellipsis menus). */
        '.dropdown-menu-panel': {
          '@apply fixed z-50 w-fit min-w-0 max-w-[calc(100vw-1rem)] overflow-y-auto bg-white border border-gray-200 rounded-lg shadow-lg':
            {},
        },
        /** Horizontal rule between menu groups. */
        '.dropdown-menu-separator': {
          '@apply my-1 border-0 border-t border-gray-200': {},
        },
        /**
         * Two-column menu: paired choices (e.g. descending | ascending) share a row.
         * Pass `columns="2"` on `Dropdown` / `ToolbarSelectDropdown`.
         */
        '.dropdown-menu-columns': {
          display: 'grid',
          width: 'max-content',
          gridTemplateColumns: 'repeat(2, max-content)',
          alignItems: 'stretch',
          columnGap: '0.125rem',
        },
        /** Anchored multiselect / popover list under a trigger. */
        '.dropdown-floating-panel': {
          '@apply absolute left-0 right-0 z-50 mt-1 flex min-h-0 flex-col overflow-hidden rounded-lg border border-gray-200 bg-white py-2 shadow-lg':
            {},
        },
        /** Language / collection dropdown: full viewport width on mobile (teleported). */
        '.dropdown-floating-panel--viewport': {
          '@apply fixed left-0 right-0 z-[60] mt-0 w-auto': {},
        },
        '.dropdown-option-preface': {
          '@apply px-3 pb-2 text-sm text-gray-600': {},
        },
        '.dropdown-option-rich': {
          '@apply min-w-0 flex-1 truncate text-sm text-gray-700': {},
        },
        '.dropdown-option-rich b': {
          '@apply font-semibold text-gray-900': {},
        },
        '.dropdown-option-rich i': {
          '@apply ml-1 font-normal italic text-gray-500': {},
        },
        '.definition-card-collection-footer': {
          '@apply mt-0 block w-full rounded-b-lg border-t border-gray-200 px-4 py-2.5 text-sm text-gray-700 no-underline hover:bg-gray-50':
            {},
        },
        '.definition-card-collection-footer-name': {
          '@apply underline underline-offset-2': {},
        },
        /** Language + collection selectors sharing the CombinedFilters top row. */
        '.filters-primary-selects': {
          '@apply grid grid-cols-2 gap-2 w-full min-w-0 flex-1': {},
        },
        /**
         * Top strip on CombinedFilters (language row).
         * Children share one vertical center. Button groups elsewhere keep the
         * shadow gutter (`pt-1 pb-3`); here that gutter lifts the pills above the dropdowns.
         */
        '.filters-bar-row': {
          '@apply flex flex-col items-center gap-4 sm:flex-row sm:justify-between md:rounded-lg md:bg-white md:p-4 md:shadow-sm':
            {},
          /** Mode / reset / expand cluster. Nested so it ships with this row. */
          '& > .filters-bar-actions': {
            '@apply flex shrink-0 items-center gap-2': {},
          },
          '& > .filters-bar-actions > .btn-group': {
            '@apply flex items-center py-0': {},
          },
          '& > .filters-bar-actions > .btn-group-forced': {
            '@apply flex items-center py-0': {},
          },
        },
        /** Compact definition preview card (list rows). */
        '.surface-definition-compact': {
          '@apply w-full bg-white border rounded-lg hover:border-blue-300 transition-colors shadow hover:shadow-none p-4':
            {},
        },
        /**
         * Scrollport for definition preview (sidebar body or embedded stack).
         */
        '.discussion-definition-scroll': {
          '@apply min-h-0 w-full flex-1 overflow-y-auto overscroll-y-contain': {},
        },
        /**
         * Shared off-canvas / in-flow sidebar chrome (AssistantChat + CommentList).
         * Use with `AppSidebarHeader.vue`.
         */
        '.app-sidebar-header': {
          '@apply flex shrink-0 items-center gap-2 border-b border-gray-200/60 bg-white/50 px-2 py-2':
            {},
        },
        '.app-sidebar-header__body': {
          '@apply flex min-w-0 flex-1 items-center gap-2': {},
        },
        '.app-sidebar-header__title': {
          '@apply min-w-0 flex-1 truncate text-sm font-semibold leading-snug text-gray-800': {},
        },
        /**
         * Discussion page: sidebar (definition) + main (comments).
         * CSS-first responsive drawer (lg+ in-flow; below lg off-canvas).
         * Do not gate the open control with a different breakpoint than the aside.
         */
        '.discussion-page': {
          '@apply relative flex h-full min-h-0 w-full min-w-0 flex-1 overflow-hidden': {},
        },
        '.discussion-sidebar-backdrop': {
          '@apply fixed inset-0 z-40 bg-black/40 backdrop-blur-[1px] transition-opacity lg:hidden':
            {},
        },
        '.discussion-sidebar': {
          '@apply z-50 flex min-h-0 w-[min(20rem,88vw)] flex-shrink-0 flex-col border-r border-gray-200 bg-gradient-to-b from-slate-50 to-gray-50/90 fixed bottom-0 left-0 top-14 -translate-x-full pointer-events-none shadow-2xl transition-transform duration-200 ease-out sm:top-12 lg:relative lg:inset-auto lg:h-full lg:w-72 lg:max-w-none lg:translate-x-0 lg:pointer-events-auto lg:shadow-none':
            {},
        },
        '.discussion-sidebar--open': {
          '@apply translate-x-0 pointer-events-auto': {},
        },
        '.discussion-sidebar__body': {
          '@apply min-h-0 flex-1 overflow-y-auto overscroll-y-contain': {},
        },
        /** Desktop: top-level comment composer pinned under the definition. */
        '.discussion-sidebar__composer': {
          '@apply shrink-0 border-t border-gray-200/60 bg-white/70 p-2': {},
        },
        /** Mobile-only open control — same lg breakpoint as sidebar in-flow. */
        '.discussion-sidebar-toggle': {
          '@apply assistant-icon-btn-header lg:hidden': {},
        },
        '.discussion-main': {
          '@apply flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden': {},
        },
        '.discussion-main__header': {
          '@apply flex shrink-0 items-center gap-2 border-b border-gray-200/60 bg-white px-3 py-1.5':
            {},
        },
        '.discussion-main__header--tap-open': {
          '@apply cursor-pointer': {},
        },
        /** One line; ellipsis on the whole title (label + word), not per child. */
        '.discussion-main__title': {
          '@apply min-w-0 flex-1 select-none truncate text-sm font-semibold leading-snug text-gray-800':
            {},
        },
        '.discussion-main__title-label': {
          '@apply mr-1.5 italic font-normal text-gray-500': {},
        },
        '.discussion-main__title-word': {
          '@apply font-semibold text-gray-800': {},
        },
        '.discussion-main__title-word--link': {
          '@apply text-blue-700 hover:text-blue-800 hover:underline': {},
        },
        '.discussion-main__header-actions': {
          '@apply flex shrink-0 flex-nowrap items-center justify-end gap-2': {},
        },
        /** Compact threaded control — avoid full-width button primitives on a toolbar label. */
        '.discussion-threaded-toggle': {
          '@apply inline-flex h-6 shrink-0 cursor-pointer items-center gap-1.5 rounded-full border border-gray-200 bg-white px-2.5 text-sm text-gray-700 select-none hover:bg-gray-50':
            {},
        },
        '.discussion-threaded-toggle--on': {
          '@apply border-slate-300 bg-slate-100 text-slate-800': {},
        },
        '.discussion-threaded-toggle--disabled': {
          '@apply cursor-not-allowed opacity-40': {},
        },
        /** Tight stacked RecentChangeItem / CommentItem rows (home landing feeds). */
        '.home-feed-list': {
          '@apply flex flex-col gap-1': {},
          '& .comment-item': {
            '@apply my-0': {},
          },
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
        /** Duolingo-clone Lingo shell + chunky buttons (`tailwind.lingo.mjs`). */
        ...buildLingoThemeLayer(),
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
        /**
         * Icon-only play/pause: standard `h-6` pill (matches `.btn-base`).
         * Aqua gloss (`::before` z-2 + span overflow) would hide the triangle — keep the glyph above it.
         */
        '.audio-player-btn': {
          '@apply !h-6 !min-h-6 !w-6 !min-w-6 !px-0 cursor-pointer shrink-0': {},
          '& span:not(.sr-only)': {
            '@apply relative z-[3] overflow-visible max-w-none': {},
          },
          '& svg': {
            '@apply relative z-[3] h-4 w-4 shrink-0 overflow-visible': {},
          },
        },
        /**
         * Reaction chip: aqua pills clip labels (`overflow: hidden` on the button and its spans).
         * The emoji pop (rotate + scale) has to paint outside that box.
         */
        '.reaction-emoji-pop': {
          '@apply !overflow-visible': {},
          '& span:not(.sr-only)': {
            '@apply relative z-[3] !overflow-visible !max-w-none': {},
          },
        },
        '.study-quiz-option-lingo-text': {
          '@apply cursor-pointer rounded-xl border border-slate-200/90 bg-white p-2.5 text-left text-xs font-medium text-gray-800 shadow-sm shadow-slate-200/50 transition-colors duration-200 hover:border-green-400 hover:bg-green-50/50 hover:shadow-md disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60 sm:text-sm':
            {},
        },

        '.study-quiz-option-lingo-image': {
          '@apply flex min-h-[80px] cursor-pointer flex-col items-center justify-center rounded-xl border border-slate-200/90 bg-white p-2 shadow-sm shadow-slate-200/50 transition-colors duration-200 hover:border-green-400 hover:bg-green-50/50 hover:shadow-md disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60':
            {},
        },
        /** Lingo study: main lesson card — thin border + soft shadow. */
        '.lingo-study-card-shell': {
          '@apply flex min-h-0 flex-1 shrink-0 flex-col overflow-hidden rounded-xl border border-slate-200/90 bg-white p-2 pb-2 shadow-sm shadow-slate-200/60 transition-shadow duration-200 hover:shadow-md hover:shadow-slate-200/80 sm:p-2.5 sm:pb-2.5':
            {},
        },

        /**
         * Lingo study header / footer: full-bleed stripe; pair with
         * `.lingo-study-chrome-inner` so controls align to the lesson column.
         */
        '.lingo-study-header-bar': {
          '-webkit-tap-highlight-color': 'transparent',
          '@apply sticky top-0 z-40 w-full shrink-0 border-b-2 border-slate-200 bg-white': {},
        },
        '.lingo-study-chrome-inner': {
          '@apply mx-auto flex h-full w-full max-w-[600px] items-center gap-x-2 px-2 py-2 sm:px-3 sm:py-2.5':
            {},
        },
        /** Lingo header back / exit — alias of `.lingo-icon-btn--muted` for study pages. */
        '.icon-btn-header-back': {
          '@apply lingo-icon-btn lingo-icon-btn--muted': {},
        },
        '.icon-btn-header-back--compact': {
          '@apply h-9 w-9': {},
        },
        /** Comment composer shell. */
        '.surface-comment-form': {
          '@apply relative mt-3 mb-6 overflow-hidden rounded-lg border bg-white p-0 transition-colors hover:border-blue-300 has-[.top-bar-heading-dropdown]:overflow-visible':
            {},
          '& .milkdown-top-bar': {
            '@apply rounded-t-lg': {},
          },
        },
        '.surface-comment-form--borderless': {
          '@apply !mt-0 !mb-0 !p-0 border-0 shadow-none hover:border-transparent': {},
        },
        /** Comment row header: avatar | username+date | #id (middle truncates). */
        '.comment-header': {
          '@apply grid grid-cols-[auto_minmax(0,1fr)_auto] items-start gap-2 min-w-0 w-full': {},
        },
        '.comment-header-main': {
          '@apply min-w-0 w-full overflow-hidden': {},
        },
        '.comment-header-username': {
          '@apply block w-full max-w-full min-w-0 overflow-hidden text-ellipsis whitespace-nowrap text-sm font-medium text-gray-700 hover:text-blue-600 hover:underline':
            {},
        },
        '.comment-header-meta': {
          '@apply text-xs text-gray-500': {},
        },
        '.comment-header-id': {
          '@apply shrink-0 text-sm text-gray-500 hover:text-blue-600 hover:underline': {},
        },
        /** Flashcard collection summary row. */
        '.surface-flashcard-summary': {
          '@apply bg-white px-3 py-2.5 sm:px-4 sm:py-3 rounded-lg border hover:border-blue-300 shadow hover:shadow-none transition-all duration-200 max-w-full overflow-hidden':
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
        /** Collection cover on list cards: fixed square; thumb uses `.collection-cover-thumb` (bg contain). */
        '.collection-card-logo': {
          '@apply flex h-12 w-12 sm:h-16 sm:w-16 shrink-0 items-center justify-center rounded-lg sm:rounded-xl border border-gray-100 bg-white shadow-sm overflow-hidden':
            {},
        },
        /** Same fixed square as `.collection-card-logo` so cover ↔ icon swap does not CLS. */
        '.collection-card-logo-placeholder': {
          '@apply flex h-12 w-12 sm:h-16 sm:w-16 shrink-0 items-center justify-center rounded-lg sm:rounded-xl bg-gray-50 text-gray-400 border border-gray-100 shadow-sm':
            {},
        },
        /**
         * Collection detail / study header cover frame: fixed widths match placeholder
         * (background thumbs have no intrinsic size — brandbook §9.1).
         */
        '.collection-header-logo': {
          '@apply box-border flex h-full min-h-0 w-16 shrink-0 items-center justify-center rounded-lg sm:rounded-xl border border-gray-100 bg-white shadow-md overflow-hidden md:w-24 lg:w-28':
            {},
        },
        '.collection-header-logo-placeholder': {
          '@apply box-border flex h-full min-h-0 w-16 shrink-0 items-center justify-center rounded-lg sm:rounded-xl bg-gray-50 text-gray-400 border border-gray-100 shadow-md md:w-24 lg:w-28':
            {},
        },
        /**
         * Collection / course cover thumb: a `div` with `background-image`, not `<img>`.
         * Contain + center — letterbox inside the frame; never crop (brandbook §9.1).
         */
        '.collection-cover-thumb': {
          '@apply h-full w-full max-h-full max-w-full bg-contain bg-center bg-no-repeat': {},
        },
        /** Header / study-strip cover thumb — same bg pattern; fills `.collection-header-logo`. */
        '.collection-header-cover-thumb': {
          '@apply block h-full w-full max-h-full max-w-full bg-contain bg-center bg-no-repeat': {},
        },
        /**
         * `PageHeader` card shell (Courses / Library / collection detail).
         * Mobile full-bleed (cancels `#main-child` `pt-3` + inner `px-3`); from `sm` inset + `rounded-lg`.
         * Gaps: see `.page-header-stack` / `__body` / `__title-row` (brandbook §8).
         */
        '.page-header-shell': {
          '@apply -mx-3 -mt-3 rounded-none border border-gray-200 bg-white p-4 sm:p-6 sm:mx-0 sm:mt-0 sm:rounded-lg':
            {},
        },
        '.page-header-shell[data-margin="section"]': {
          '@apply mb-6': {},
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
          '@apply px-4 py-1.5 text-sm h-8 text-gray-700 bg-white border border-gray-300 placeholder-blue-400 rounded-full transition-all focus:ring-1 focus:ring-blue-500 focus:border-blue-500 focus:outline-none focus:z-50 whitespace-nowrap overflow-hidden text-ellipsis shadow-inner shadow-slate-200':
            {},
          '&:hover:not(:disabled)': {
            '@apply border-blue-400': {},
          },
          '&::placeholder': {
            '@apply text-gray-400 overflow-hidden text-ellipsis': {},
          },
          '&:disabled': {
            '@apply bg-gray-100 cursor-not-allowed opacity-75': {},
          },
        },
        /** Native `<select>`: vertical padding stacks with fixed `h-8` and clips option text — keep full height for the control chrome. */
        'select.input-field': {
          '@apply py-0': {},
        },
        /** Leading-icon slot inside `Input` (`searchIcon`); pair with `pl-10` on the field. */
        '.input-field-leading-icon': {
          '@apply pointer-events-none absolute inset-y-0 left-0 z-10 flex w-10 items-center justify-center text-gray-400':
            {},
        },
        /**
         * Soft right-edge fade over empty search fields so long placeholders ease into
         * padding instead of hard-clipping (pairs with `.input-field` ellipsis).
         */
        '.search-input-placeholder-fade': {
          '@apply pointer-events-none absolute inset-y-px right-px z-[5] w-8 rounded-r-full': {},
          background: 'linear-gradient(to left, rgb(255 255 255) 35%, rgb(255 255 255 / 0))',
          '&--trailing': {
            /* Sit just left of the h-4 spinner / clear control (`right-3` + icon). */
            '@apply right-9 w-10': {},
          },
        },
        /** Decorative icon inside `relative` input wrapper (user, mail, key). */
        '.input-field-trailing-icon': {
          '@apply pointer-events-none absolute right-3 top-1/2 z-10 h-5 w-5 -translate-y-1/2 shrink-0 text-gray-400':
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
        /** Closed trigger for toolbar dropdowns (search mode, language multiselect). */
        '.dropdown-trigger': {
          '@apply w-full h-6 min-h-[1.5rem] max-w-full min-w-0 shrink flex flex-nowrap items-center justify-between gap-2 overflow-hidden px-3 text-left text-sm font-normal text-gray-700 bg-white border border-gray-300 rounded-full transition-all focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 focus:z-50 shadow-inner shadow-slate-200':
            {},
          '&:hover:not(:disabled)': {
            '@apply border-blue-400': {},
          },
        },
        /**
         * Action-menu trigger (ellipsis overflow). Theme chrome is in `buttonUiThemeLayer`:
         * Aqua = glossy button; Flat = select pill without inset shadow.
         */
        '.dropdown-action-trigger': {
          '@apply w-auto max-w-full min-w-0 h-6 min-h-[1.5rem] shrink inline-flex flex-nowrap items-center gap-2 overflow-hidden px-3 text-left cursor-pointer select-none whitespace-nowrap transition-all disabled:opacity-40 disabled:cursor-not-allowed':
            {},
          '& > span': {
            '@apply relative z-[3] min-w-0': {},
          },
          '& svg': {
            '@apply relative z-[3] shrink-0': {},
          },
        },
        /**
         * Single-line trigger label. Preflight sets `svg { display: block }`, which stacks an icon
         * above the text inside a nowrap span; keep icons inline and ellipsize the text instead.
         */
        '.dropdown-trigger-label': {
          '@apply block min-w-0 max-w-full overflow-hidden text-ellipsis whitespace-nowrap text-left':
            {},
          '& svg': {
            '@apply inline-block shrink-0 align-middle': {},
          },
        },
        /** Default trigger for `Dropdown.vue` (ellipsis action menu). */
        '.dropdown-ellipsis-trigger': {
          '@apply dropdown-action-trigger': {},
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
        /**
         * Search column: dedicated full-width row on small screens (min-w-full beats
         * flex-1 shrinking beside filters), bounded width on sm+.
         */
        '.toolbar-search-slot': {
          '@apply w-full min-w-full sm:w-auto sm:min-w-[220px] sm:max-w-[280px] flex-1 sm:flex-initial':
            {},
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
        /** Trailing segment of the home search bar (flush join with the h-10 query input). */
        '.dropdown-trigger--search-bar-trailing': {
          '@apply w-auto max-w-none rounded-l-none rounded-r-full': {},
        },
        '.dropdown-trigger.dropdown-trigger--search-bar-leading, .dropdown-trigger.dropdown-trigger--search-bar-trailing':
          {
            height: '2.5rem',
            minHeight: '2.5rem',
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
        /** Unread count pip on nav icons (News / Activity). */
        '.nav-unread-badge': {
          '@apply absolute -top-1 -right-1 inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-red-500 px-1 text-xs font-medium text-white':
            {},
        },
        '.nav-unread-badge--wide': {
          '@apply h-6 min-w-6': {},
        },
        /**
         * Three-block feed shell (header | scroll body | footer).
         * Use with route meta.fullHeight so the page fills main and only the body scrolls.
         * The scrollport is the full width of main (scrollbar at the page edge).
         * Header, column, and footer stay on the shared max-w-4xl measure.
         */
        '.feed-page': {
          '@apply relative flex h-full min-h-0 w-full flex-1 flex-col': {},
        },
        '.feed-page__header': {
          '@apply z-20 mx-auto w-full max-w-4xl shrink-0 bg-transparent px-3': {},
        },
        '.feed-page__header--tabs': {
          '@apply pb-1': {},
        },
        /** Only this block scrolls. It is full width so the scrollbar sits on the main edge. */
        '.feed-page__body': {
          /* Keep block flow (not flex-col): flex shrink lets tall children overflow over later siblings. */
          '@apply min-h-0 w-full flex-1 overflow-x-hidden overflow-y-auto overscroll-y-contain pb-0':
            {},
          /* Reserve the scrollbar lane: tab/content swaps that drop the scrollbar must not shift (CLS). */
          scrollbarGutter: 'stable',
        },
        /** Content measure. Matches #main-child max-w-4xl on non-feed pages. */
        '.feed-page__column': {
          '@apply mx-auto w-full max-w-4xl px-3 pt-3': {},
        },
        '.feed-page__footer': {
          '@apply z-30 mx-auto w-full max-w-4xl shrink-0 border-t border-gray-200 bg-white md:bg-zinc-50':
            {},
        },
        '.feed-page__footer-inner': {
          '@apply flex w-full flex-col': {},
        },
        /** Upsert scroll body. Horizontal padding stays on the form so the notice spans this box. */
        '.upsert-scroll': {
          '@apply min-h-0 w-full flex-1 overflow-x-hidden overflow-y-auto overscroll-y-contain pb-0':
            {},
        },
        '.upsert-scroll__main': {
          '@apply w-full px-3 pt-4 pb-4 sm:px-4': {},
        },
        /**
         * Legal fine print at the end of page content.
         * Same striped wash as global `header` / `footer` in App.vue.
         */
        '.public-domain-notice': {
          '@apply mt-6 w-full shrink-0 border-t border-gray-200/80 px-3 py-2.5': {},
          background: 'repeating-linear-gradient(#f8f8f8, #f8f8f8 4px, #ffffff 4px, #ffffff 8px)',
        },
        '.public-domain-notice__text': {
          '@apply w-full text-center text-[10px] leading-snug text-gray-500': {},
        },
        /** Extra right inset so the FAB does not cover Previous/Next. */
        '.feed-page__pagination': {
          '@apply w-full shrink-0 pt-1 pb-1 pl-3 pr-20 sm:pt-2 sm:pb-2 sm:pl-3': {},
        },
        /**
         * One line when both clusters fit; wraps to centered rows on narrow viewports.
         * Children stay max-content so neither stretches to a full-width row.
         */
        /**
         * Full-width: summary left + controls right when both fit (sm+).
         * On wrap (narrow), each row is centered.
         */
        /** In-flow marker for docked pagination; hidden ancestor means this instance stays inline. */
        '.feed-pagination-sentinel': {
          '@apply pointer-events-none absolute h-px w-px overflow-hidden': {},
        },
        '.pagination-bar': {
          '@apply flex w-full flex-wrap items-center justify-center gap-x-4 gap-y-2 border-0 bg-transparent p-0 sm:justify-between':
            {},
        },
        '.pagination-bar--end': {
          '@apply !justify-end': {},
        },
        '.pagination-bar__summary': {
          '@apply max-w-full shrink-0 text-sm text-gray-700 whitespace-nowrap': {},
          width: 'max-content',
        },
        '.pagination-bar__controls': {
          '@apply flex max-w-full shrink-0 items-center justify-center gap-2': {},
          width: 'max-content',
        },
        '.navbar-item-badge-wrap': {
          '@apply relative inline-flex items-center': {},
        },
        '.badge': {
          '@apply inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium': {},
        },
        '.badge-muted': {
          '@apply bg-gray-50 text-gray-500': {},
        },
        /** Public / private collection visibility chip (`data-public="true|false"`). */
        '.badge-visibility': {
          '@apply inline-flex h-6 shrink-0 items-center select-none rounded-full px-2 py-1 text-xs':
            {},
        },
        '.badge-visibility[data-public="true"]': {
          '@apply bg-green-100 text-green-700': {},
        },
        '.badge-visibility[data-public="false"]': {
          '@apply bg-gray-100 text-gray-700': {},
        },
        /** Public catalog / library collection card grid. */
        '.collections-section': {
          '@apply relative w-full min-h-[20rem]': {},
        },
        '.collections-grid': {
          '@apply grid gap-4 sm:gap-5 items-stretch grid-cols-1 sm:grid-cols-2': {},
        },
        '.card-meta': {
          '@apply flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-gray-500 shrink-0': {},
        },
        '.card-meta-by': {
          '@apply flex items-center gap-1': {},
        },
        '.card-meta-link': {
          '@apply !font-normal text-blue-600 hover:text-blue-800 hover:underline': {},
        },
        '.card-meta-date': {
          '@apply text-gray-400': {},
        },
        /** Users who drilled at least one card in this public course during the last 30 days. */
        '.card-meta-drillers': {
          '@apply inline-flex basis-full items-center gap-1 text-xs text-gray-500': {},
        },
        /**
         * Study streak strip (collections / library / lingo aside).
         * Compact pad; min-heights + fixed day slots match skeleton → loaded (no CLS).
         * Chrome (`card-base` / `lingo-card`) is passed by the caller — no extra `p-*` on the Vue root.
         */
        '.card-streak': {
          '@apply w-full min-w-0 max-w-full min-h-[9.5rem] p-3 sm:min-h-[10.75rem]': {},
        },
        '.card-streak-header': {
          '@apply mb-1.5 flex min-h-7 min-w-0 flex-wrap items-center justify-between gap-x-2 gap-y-1':
            {},
        },
        '.card-streak-title': {
          '@apply min-w-0 flex-1 text-sm font-semibold text-gray-800 sm:flex-none sm:text-base': {},
        },
        '.card-streak-meta': {
          '@apply card-meta min-w-0 text-sm sm:shrink-0': {},
        },
        '.card-streak-week-grid': {
          '@apply grid w-full min-w-0 gap-1 sm:gap-1.5': {},
          gridTemplateColumns: 'repeat(7, minmax(0, 1fr))',
        },
        '.card-streak-day': {
          '@apply flex h-[5.5rem] min-h-0 min-w-0 flex-col items-center justify-center gap-0.5 overflow-hidden text-center sm:h-[6.5rem] sm:gap-1':
            {},
        },
        '.card-streak-day-label': {
          '@apply line-clamp-2 w-full text-[10px] font-medium leading-tight text-gray-500 break-words sm:text-xs':
            {},
        },
        '.card-streak-day-points': {
          '@apply w-full text-center text-sm leading-tight sm:text-base': {},
        },
        '.card-streak-day-count': {
          '@apply flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-[11px] font-medium sm:h-7 sm:w-7 sm:text-xs':
            {},
        },
        '.card-streak-day-count[data-active="true"]': {
          '@apply bg-blue-100 text-blue-700': {},
        },
        '.card-streak-day-count[data-active="false"]': {
          '@apply bg-gray-100 text-gray-400': {},
        },
        /** Pulse bars: heights match label / points line boxes (no CLS vs loaded streak). */
        '.card-streak-skeleton-line': {
          '@apply mx-auto h-[0.875rem] w-full max-w-[3.5rem] shrink-0 rounded bg-gray-100 sm:h-[1rem]':
            {},
        },
        '.card-streak-skeleton-line--points': {
          '@apply h-5 max-w-[2rem] sm:h-6': {},
        },
        /** Skeleton title / meta — same slots as loaded header (min-h-7). */
        '.card-streak-skeleton-title': {
          '@apply h-5 w-1/3 min-w-[5.5rem] max-w-[10rem] rounded bg-gray-200 sm:h-6': {},
        },
        '.card-streak-skeleton-meta': {
          '@apply h-4 w-24 rounded bg-gray-100 sm:w-28': {},
        },
        /**
         * Feed / card loading placeholders (Recent Changes tabs, etc.).
         * Prefer these over one-off gray bars so bone radius/color stay consistent.
         */
        '.skeleton-pulse': {
          '@apply animate-pulse': {},
          '@media (prefers-reduced-motion: reduce)': {
            animation: 'none',
          },
        },
        '.skeleton-bone': {
          '@apply rounded bg-gray-200': {},
        },
        '.skeleton-bone--soft': {
          '@apply rounded bg-gray-100': {},
        },
        /**
         * Legacy activity loading row shell. Prefer `SkeletonActivityItem.vue`
         * (`comment-item` chrome) so skeletons match CommentItem / RecentChangeItem.
         */
        '.skeleton-activity-row': {
          '@apply space-y-2 bg-white p-3 rounded-lg border border-gray-200': {},
        },
        '.card-study-area': {
          '@apply flex items-center justify-center min-h-[5.5rem] py-6 sm:py-8 flex-shrink-0': {},
        },
        /** Compact study CTA band on catalog cards — always mounted to avoid segment-toggle CLS. */
        '.card-study-area-compact': {
          '@apply flex shrink-0 items-center justify-center min-h-[3.25rem] py-3': {},
        },
        /**
         * PageHeader internals (machine source for brandbook §8).
         * Gap scale — do not re-declare `gap-*` in page templates; use these classes.
         * - stack compact (default): 0.5rem between major blocks (body / meta / toolbar)
         * - stack comfortable: 1rem (catalog intros, collection detail)
         * - body: 0.5rem between eyebrow / title-row / description
         * - title-row / trailing / actions: 0.5rem
         * - leading-row: 0.75rem → 1rem from `sm`
         */
        '.page-header-stack': {
          '@apply flex w-full min-w-0 flex-col gap-2': {},
        },
        '.page-header-stack[data-stack-gap="comfortable"]': {
          '@apply gap-4': {},
        },
        '.page-header-leading-row': {
          '@apply flex flex-row items-stretch gap-3 sm:gap-4': {},
        },
        '.page-header-body': {
          '@apply flex w-full min-w-0 flex-col gap-2 text-left': {},
        },
        '.page-header-title-row': {
          '@apply flex w-full min-w-0 flex-wrap items-center justify-between gap-2': {},
        },
        /** Split layout top band (leading+text | trailing) — slightly roomier than title-row. */
        '.page-header-split-row': {
          '@apply flex w-full min-w-0 flex-wrap items-center justify-between gap-4': {},
        },
        '.page-header-title-cluster': {
          '@apply flex min-w-0 flex-1 flex-wrap items-center gap-x-2 gap-y-1': {},
        },
        '.page-header-trailing': {
          '@apply flex shrink-0 flex-wrap items-center justify-end gap-2': {},
        },
        '.page-header-toolbar': {
          '@apply w-full min-w-0': {},
        },
        /** Button / link rows inside `#toolbar` or `#trailing` slot content. */
        '.page-header-actions': {
          '@apply flex flex-wrap items-center justify-end gap-2': {},
        },
        /** Intro copy; `div` stacks paragraphs with `gap-2`, bare `<p>` is typography only. */
        '.page-header-description': {
          '@apply max-w-2xl text-sm text-slate-600': {},
        },
        'div.page-header-description': {
          '@apply flex flex-col gap-2': {},
        },
        '.page-header-title': {
          '@apply my-0 min-w-0 max-w-full break-words font-bold text-gray-800 [overflow-wrap:anywhere] text-2xl sm:text-3xl':
            {},
        },
        '.page-header-title[data-tone="secondary"]': {
          '@apply text-xl sm:text-2xl': {},
        },
        '.page-header-meta': {
          '@apply flex w-full min-w-0 flex-wrap items-center gap-x-2 gap-y-2': {},
        },
        '.page-header-meta:has([data-slot="meta"]:not(:empty)):has([data-slot="meta-trailing"]:not(:empty))':
          {
            '@apply justify-between': {},
          },
        '.page-header-meta:not(:has([data-slot="meta"]:not(:empty))):has([data-slot="meta-trailing"]:not(:empty))':
          {
            '@apply justify-end': {},
          },
        '.page-header-meta-cluster': {
          '@apply flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1': {},
        },
        '.page-header-meta-trailing': {
          '@apply flex shrink-0 flex-wrap items-center justify-end gap-2': {},
        },
        /** Native / imported wiki article: dense reading layout (especially on mobile). */
        '.wiki-article': {
          '@apply flex flex-col gap-2 pb-2 sm:gap-3': {},
        },
        '.wiki-article-toolbar': {
          '@apply flex min-w-0 items-center gap-2': {},
        },
        '.wiki-article-card': {
          '@apply rounded-lg border border-gray-100 bg-white px-3 py-2.5 shadow-sm sm:px-4 sm:py-3':
            {},
        },
        '.wiki-article-header': {
          '@apply mb-2 border-b border-gray-100 pb-2': {},
        },
        '.wiki-article-title': {
          '@apply min-w-0 text-lg font-bold leading-tight text-gray-800 sm:text-xl': {},
        },
        '.wiki-article-actions': {
          '@apply flex shrink-0 flex-nowrap items-center gap-1.5': {},
        },
        '.wiki-article-meta': {
          '@apply mt-0.5 text-xs text-gray-500': {},
        },
        '.wiki-article-body': {
          '@apply max-w-none break-words text-sm leading-snug text-gray-700 sm:text-base': {},
          '& p': {
            marginTop: '0.375rem',
            marginBottom: '0.375rem',
          },
          '& p:first-child': {
            marginTop: '0',
          },
          '& p:last-child': {
            marginBottom: '0',
          },
          '& h1, & h2, & h3, & h4': {
            '@apply mb-1 mt-2.5 font-semibold': {},
            '&:first-child': {
              marginTop: '0',
            },
          },
          '& ul, & ol': {
            '@apply my-1.5 pl-5': {},
          },
          '& li': {
            '@apply my-0.5': {},
          },
          '& a': {
            '@apply font-medium text-blue-600 underline hover:text-blue-800': {},
          },
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
          /** Undo / restore — amber caution (yellow reserved for edit). */
          'ui-btn--revert': { aqua: 'btn-aqua-amber', flat: 'btn-warning' },
          'ui-btn--sort-amber': { aqua: 'btn-aqua-amber', flat: 'btn-warning' },
          'ui-btn--sort-blue': { aqua: 'btn-aqua-blue', flat: 'btn-insert' },
          'ui-btn--sort-emerald': { aqua: 'btn-aqua-emerald', flat: 'btn-create' },
          'ui-btn--sort-sky': { aqua: 'btn-aqua-sky', flat: 'btn-success' },
          'ui-btn--study-correct': { aqua: 'btn-aqua-teal', flat: 'btn-success' },
          'ui-btn--study-wrong': { aqua: 'btn-aqua-rose', flat: 'btn-error' },
          'ui-btn--success': { aqua: 'btn-aqua-emerald', flat: 'btn-success' },
          /** Edit / apply — yellow (flat `btn-update` / `btn-edit`, aqua `btn-aqua-yellow`). */
          'ui-btn--update': { aqua: 'btn-aqua-yellow', flat: 'btn-update' },
          /** Role: save edits / apply changes — synonym of `update`. */
          'ui-btn--edit': { aqua: 'btn-aqua-yellow', flat: 'btn-edit' },
          'ui-btn--warning': { aqua: 'btn-aqua-orange', flat: 'btn-warning' },
          'ui-btn--warning-orange': { aqua: 'btn-aqua-orange', flat: 'btn-warning' },
          /** Caution highlight — amber in flat theme (yellow reserved for edit/update). */
          'ui-btn--warning-yellow': { aqua: 'btn-aqua-yellow', flat: 'btn-warning' },
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
          '@apply aqua-base bg-pink-400 !h-12 !w-12 cursor-pointer rounded-full !text-lg !text-white':
            {},
          '--aqua-hue': '330',
        }
        flatRules[selectorFor('flat', 'ui-btn--fab')] = {
          '@apply btn-fab !h-12 !w-12 cursor-pointer rounded-full !text-lg': {},
        }
        /** Flat FAB: no outer ring; soft cornflower-tinted float shadow (ui-btn--fab is borderless). */
        flatRules[selectorFor('flat', 'fab-elevation-shell')] = {
          '@apply ring-0': {},
          boxShadow: `0 12px 28px -10px color-mix(in srgb, ${theme('colors.pink.700')} 38%, transparent), 0 8px 18px -10px color-mix(in srgb, ${theme('colors.pink.400')} 32%, transparent)`,
          '&:hover': {
            boxShadow: `0 16px 36px -12px color-mix(in srgb, ${theme('colors.pink.700')} 42%, transparent), 0 10px 22px -10px color-mix(in srgb, ${theme('colors.pink.400')} 38%, transparent)`,
          },
        }
        /** Aqua FAB: elevation matches cornflower hue (replaces rose-tinted shell). */
        aquaRules[selectorFor('aqua', 'fab-elevation-shell')] = {
          boxShadow: `0 10px 26px -8px rgba(15, 23, 42, 0.16), 0 6px 14px -6px color-mix(in srgb, ${theme('colors.pink.400')} 32%, transparent)`,
          '&:hover': {
            boxShadow: `0 18px 36px -10px rgba(15, 23, 42, 0.2), 0 10px 22px -8px color-mix(in srgb, ${theme('colors.pink.500')} 38%, transparent)`,
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
        for (const [cls, rule] of Object.entries(buildAquaToggleSwitchThemeRules())) {
          aquaRules[selectorFor('aqua', cls)] = rule
        }
        for (const [cls, rule] of Object.entries(buildFlatToggleSwitchThemeRules())) {
          flatRules[selectorFor('flat', cls)] = rule
        }
        /** Beats semantic ui-btn--* primitives so inner segments stay joined in forced groups (all breakpoints). */
        aquaRules[selectorFor('aqua', 'btn-group-forced .ui-btn--group-item')] =
          buildAquaUiBtnGroupItemGeometry()
        flatRules[selectorFor('flat', 'btn-group-forced .ui-btn--group-item')] = {
          /** Full borders on every segment; `-ml-px` overlaps adjacent 1px borders; hover/focus z-index reveals full outline.
           *  Round the first/last `button` or `a`, not `:first-child`/`:last-child`, so a lone control
           *  stays a pill when a non-control sibling (hidden spacer) sits in the group.
           *  Raw radii (not `@apply rounded-*-full`): Tailwind rewrites nested
           *  `:nth-child(1 of …)` `@apply` into `:first-child` / `:last-child`. */
          '@apply relative z-0 rounded-none border-r -ml-px': {},
          '&:nth-child(1 of button, a)': {
            marginLeft: '0',
            borderTopLeftRadius: '9999px',
            borderBottomLeftRadius: '9999px',
          },
          '&:nth-last-child(1 of button, a)': {
            borderTopRightRadius: '9999px',
            borderBottomRightRadius: '9999px',
          },
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
        /**
         * Action-menu trigger: Aqua matches toolbar glossy buttons (`btn-aqua-simple` chrome inlined
         * because this layer cannot `@apply` primitives from the earlier `addComponents` pass).
         * Flat matches `.dropdown-trigger` geometry without the inset field shadow.
         */
        aquaRules[selectorFor('aqua', 'dropdown-action-trigger')] = {
          '@apply aqua-base justify-center cursor-pointer': {},
          boxShadow:
            '0 0.35em 0.45em rgba(2, 10, 26, 0.28),' +
            '0 0.08em 0.18em hsla(var(--aqua-hue, 215), 85%, 46%, 0.42),' +
            'inset 0 0.22em 0.45em hsla(calc(var(--aqua-hue, 215) - 8), 75%, 22%, 0.65),' +
            'inset 0 0.4em 0.55em 0.2em hsla(var(--aqua-hue, 215), 95%, 68%, 0.28)',
          '&[aria-expanded="true"]': {
            '@apply brightness-105 saturate-150': {},
          },
        }
        flatRules[selectorFor('flat', 'dropdown-action-trigger')] = {
          '@apply justify-between text-sm font-normal text-gray-700 bg-white border border-gray-300 rounded-full focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 focus:z-50':
            {},
          boxShadow: 'none',
          '&:hover:not(:disabled)': {
            '@apply border-blue-400': {},
          },
          '&:focus-visible': {
            '@apply ring-1 ring-blue-500 border-blue-500': {},
          },
          '&[aria-expanded="true"]': {
            '@apply border-blue-500 ring-1 ring-blue-500 z-50': {},
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
        /**
         * Active reaction label (the count) is white in flat theme. `aqua-base` inlines
         * `text-black` into `.ui-btn--reaction-active`, and a later utility in that same
         * `@apply` does not replace it — this selector is more specific.
         */
        aquaRules['html[data-button-theme="aqua"] button.ui-btn--reaction-active'] = {
          color: theme('colors.white'),
        }
        Object.assign(rules, aquaRules, flatRules)
        return rules
      }
      addComponents(buttonUiThemeLayer())

      // --- Assistant page Button theme (used only by AssistantChat.vue) ---
      /**
       * Maps @packages/ui Button variants to the legacy assistant classes so the
       * chat page keeps its pre-refactor look while still using the shared Button.
       */
      addComponents({
        '.ui-btn--assistant-icon-soft': { '@apply assistant-icon-btn-soft': {} },
        '.ui-btn--assistant-new-chat': { '@apply assistant-new-chat-trigger': {} },
        '.ui-btn--assistant-session': {
          '@apply assistant-session-row': {},
          '& > span': { '@apply block w-full': {} },
        },
        '.ui-btn--assistant-icon-header': { '@apply assistant-icon-btn-header': {} },
        '.ui-btn--assistant-icon-panel': { '@apply assistant-icon-btn-panel': {} },
        '.ui-btn--assistant-bubble-action': { '@apply assistant-bubble-action': {} },
        '.ui-btn--assistant-composer-send': { '@apply assistant-composer-send': {} },
      })
    },
  ],
}
