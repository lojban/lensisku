/**
 * Lingo (Duolingo-clone) theme layer.
 * Chunky 2D controls with thick bottom borders — not Aqua gloss or Flat pills.
 * Imported into `tailwind.config.js` `addComponents`. See `docs/brandbook.md` §10.3.
 */

export function buildLingoThemeLayer() {
  return {
    /**
     * Base geometry for Lingo chunky buttons (clone: `buttonVariants` root).
     * Pair with a `--*` variant; do not mix with `ui-btn--*` / `.aqua-base` / `.btn-flat-surface`.
     */
    '.lingo-btn': {
      '@apply inline-flex cursor-pointer select-none items-center justify-center gap-2 whitespace-nowrap rounded-xl border-2 px-4 py-2 text-sm font-bold uppercase tracking-wide transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-green-500 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50':
        {},
      minHeight: '2.75rem', // h-11
    },
    '.lingo-btn--default': {
      '@apply border-b-4 border-slate-200 bg-white text-slate-500 hover:bg-slate-100 active:border-b-2':
        {},
    },
    '.lingo-btn--primary': {
      '@apply border-b-4 border-sky-500 bg-sky-400 text-white hover:bg-sky-400/90 active:border-b-0':
        {},
    },
    '.lingo-btn--secondary': {
      '@apply border-b-4 border-green-600 bg-green-500 text-white hover:bg-green-500/90 active:border-b-0':
        {},
    },
    '.lingo-btn--danger': {
      '@apply border-b-4 border-rose-600 bg-rose-500 text-white hover:bg-rose-500/90 active:border-b-0':
        {},
    },
    '.lingo-btn--locked': {
      '@apply border-b-4 border-neutral-400 bg-neutral-200 text-neutral-400 hover:bg-neutral-200/90 active:border-b-0':
        {},
    },
    '.lingo-btn--ghost': {
      '@apply border-0 border-transparent bg-transparent text-slate-500 hover:bg-slate-100': {},
    },
    '.lingo-btn--sm': {
      '@apply h-9 min-h-9 px-3 py-1.5 text-xs': {},
    },
    '.lingo-btn--lg': {
      '@apply h-12 min-h-12 px-8': {},
    },
    '.lingo-btn--icon': {
      '@apply h-10 w-10 min-h-10 min-w-10 shrink-0 p-0': {},
    },
    '.lingo-btn--rounded': {
      '@apply rounded-full': {},
    },
    /** Sidebar nav row (inactive). */
    '.lingo-btn--sidebar': {
      '@apply h-[52px] w-full justify-start rounded-xl border-2 border-transparent bg-transparent px-4 text-slate-500 transition-none hover:bg-slate-100':
        {},
    },
    /** Sidebar nav row (active) — sky outline like the clone. */
    '.lingo-btn--sidebar-active': {
      '@apply h-[52px] w-full justify-start rounded-xl border-2 border-sky-300 bg-sky-500/15 px-4 text-sky-500 transition-none hover:bg-sky-500/20':
        {},
    },
    /** Icon-only chrome (menu, close, back) — never Aqua/Flat pills. */
    '.lingo-icon-btn': {
      '@apply inline-flex h-10 w-10 shrink-0 cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent p-0 text-inherit transition-colors hover:bg-black/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-green-500 focus-visible:ring-offset-2':
        {},
    },
    '.lingo-icon-btn--on-green': {
      '@apply text-white hover:bg-white/20 focus-visible:ring-white': {},
    },
    '.lingo-icon-btn--muted': {
      '@apply text-slate-500 hover:bg-slate-100 focus-visible:ring-green-500': {},
    },
    /** Fixed green mobile top bar (clone: h-[50px]). */
    '.lingo-mobile-header': {
      '@apply fixed left-0 right-0 top-0 z-50 flex h-[50px] w-full items-center gap-3 border-b border-green-600/25 bg-green-500 px-4 lg:hidden':
        {},
    },
    /**
     * Desktop left rail (fixed) + drawer body on small screens.
     * z-40 keeps the rail above main content; never rely on in-flow placement alone.
     */
    '.lingo-sidebar-rail': {
      '@apply flex h-full w-full flex-col border-r-2 border-slate-200 bg-white px-4 lg:fixed lg:left-0 lg:top-0 lg:z-40 lg:h-svh lg:w-[256px]':
        {},
    },
    '.lingo-sidebar-brand': {
      '@apply flex items-center gap-x-3 pb-7 pl-4 pt-8 text-2xl font-extrabold tracking-wide text-green-600':
        {},
    },
    '.lingo-sidebar-nav': {
      '@apply flex flex-1 flex-col gap-y-2': {},
    },
    /**
     * Main column: top offset for mobile header, bottom for tab bar, left for desktop rail.
     * Safe-area padding is raw CSS (commas in `env()` break `@apply`).
     */
    '.lingo-main': {
      '@apply min-h-0 flex-1 pt-[50px] lg:pl-[256px] lg:pt-0': {},
      paddingBottom: 'calc(4.25rem + env(safe-area-inset-bottom, 0px))',
      '@screen lg': {
        paddingBottom: '0',
      },
    },
    '.lingo-main-inner': {
      '@apply mx-auto h-full w-full max-w-[1056px] px-4 pt-6 lg:px-6': {},
    },
    /**
     * Persistent mobile / tablet tab bar — always reachable under lg.
     * Do not `@apply lg:hidden` here (responsive variants in addComponents can
     * collapse to unconditional `display:none`).
     */
    '.lingo-bottom-nav': {
      '@apply fixed inset-x-0 bottom-0 z-50 flex h-[4.25rem] items-stretch justify-around gap-0 border-t-2 border-slate-200 bg-white px-1':
        {},
      paddingBottom: 'env(safe-area-inset-bottom, 0px)',
      '@media (min-width: 1024px)': {
        display: 'none',
      },
    },
    '.lingo-bottom-nav-item': {
      '@apply flex min-w-0 flex-1 flex-col items-center justify-center gap-0.5 px-1 py-1 text-[10px] font-bold uppercase tracking-wide text-slate-500 no-underline transition-colors':
        {},
    },
    '.lingo-bottom-nav-item--active': {
      '@apply text-green-600': {},
    },
    /** Course pick tile (clone: courses/card.tsx) — plain button, not ui-btn. */
    '.lingo-course-card': {
      '@apply flex h-full min-h-[217px] min-w-0 w-full cursor-pointer flex-col items-center justify-between rounded-xl border-2 border-b-[4px] border-neutral-200 bg-white p-3 pb-6 text-left transition hover:bg-black/5 active:border-b-2 disabled:pointer-events-none disabled:opacity-50 lg:min-w-[200px]':
        {},
    },
    /** Unit / collection banner on the learn ladder. */
    '.lingo-unit-banner': {
      '@apply flex w-full flex-col gap-4 rounded-xl bg-green-500 p-5 text-white sm:flex-row sm:items-center sm:justify-between':
        {},
    },
    /** Ladder lesson circle (clone: secondary / locked + border-b-8). */
    '.lingo-lesson-node': {
      '@apply flex h-[70px] w-[70px] shrink-0 items-center justify-center rounded-full border-2 border-b-8 transition-[transform,box-shadow] duration-150':
        {},
    },
    '.lingo-lesson-node--open': {
      '@apply border-green-600 bg-green-500 text-white hover:bg-green-500/90 active:border-b-0': {},
    },
    '.lingo-lesson-node--locked': {
      '@apply border-neutral-400 bg-neutral-200 text-neutral-400 active:border-b-0': {},
    },
    /** Owner gear beside a lesson node — flat, not Aqua. */
    '.lingo-lesson-menu-trigger': {
      '@apply lingo-icon-btn lingo-icon-btn--muted h-8 w-8 rounded-full': {},
    },
    /** Drawer panel chrome. */
    '.lingo-drawer-panel': {
      '@apply fixed left-0 top-0 z-[101] flex h-full w-[min(100%,16rem)] flex-col overflow-y-auto bg-white shadow-xl lg:hidden':
        {},
    },
    /** Close control host inside the drawer (relative positioning context). */
    '.lingo-drawer-panel > .lingo-sidebar-rail': {
      '@apply relative min-h-full': {},
    },
    '.lingo-drawer-backdrop': {
      '@apply fixed inset-0 z-[100] bg-black/50 lg:hidden': {},
    },
    /** Empty / placeholder panels inside Lingo. */
    '.lingo-panel': {
      '@apply w-full rounded-xl border-2 border-slate-200 bg-white p-4 text-center text-slate-500':
        {},
    },
    /** Study footer SRS grades (Forgot / Good / Easy). */
    '.lingo-study-rate-row': {
      '@apply flex w-full items-stretch gap-1.5 sm:gap-2': {},
    },
    /** Clone lesson title (`quiz.tsx` h1). */
    '.lingo-study-lesson-title': {
      '@apply shrink-0 text-center text-lg font-bold text-neutral-700 lg:text-start lg:text-3xl':
        {},
    },
    /** Clone `QuestionBubble`: mascot + speech bubble for the stimulus. */
    '.lingo-study-question-row': {
      '@apply mb-2 flex items-center gap-x-3 sm:mb-4 sm:gap-x-4': {},
    },
    /** Portrait LTD icons: fixed max height, width follows aspect (object-contain). */
    '.lingo-study-mascot': {
      '@apply h-16 w-auto max-h-16 max-w-[3.5rem] shrink-0 object-contain object-center sm:h-20 sm:max-h-20 sm:max-w-[4.25rem] lg:h-[88px] lg:max-h-[88px] lg:max-w-[72px]':
        {},
    },
    '.lingo-study-question-bubble': {
      '@apply relative min-w-0 flex-1 rounded-xl border-2 border-neutral-200 bg-white px-4 py-2 text-sm text-neutral-700 lg:text-base':
        {},
    },
    /** Speech-bubble caret (clone: rotated border triangle on the left). */
    '.lingo-study-question-bubble::before': {
      content: '""',
      '@apply absolute -left-3 top-1/2 h-0 w-0 -translate-y-1/2 rotate-90 border-x-8 border-t-8 border-x-transparent border-t-neutral-200':
        {},
    },
    '.lingo-study-stimulus': {
      '@apply text-sm font-medium text-neutral-700 lg:text-base': {},
    },
    '.lingo-study-lesson-stack': {
      '@apply flex w-full max-h-full min-h-0 flex-col gap-y-6 sm:gap-y-8 lg:gap-y-12': {},
    },
    /** Leaderboard / list avatar ring (clone: green border circle). */
    '.lingo-avatar': {
      '@apply mr-4 flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-full border-2 border-green-500 bg-green-100 text-sm font-bold text-green-700':
        {},
    },
    '.lingo-avatar img': {
      '@apply h-full w-full object-cover': {},
    },
  }
}
