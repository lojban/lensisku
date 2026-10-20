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
    /** Course row in the Lingo rail: cover + title + description. */
    '.lingo-btn--sidebar-course': {
      '@apply h-auto min-h-0 w-full items-center justify-start gap-2 rounded-xl border border-transparent bg-transparent px-1.5 py-1.5 text-left text-slate-600 normal-case tracking-normal transition-none hover:bg-slate-100':
        {},
    },
    '.lingo-btn--sidebar-course-active': {
      '@apply h-auto min-h-0 w-full items-center justify-start gap-2 rounded-xl border border-sky-300 bg-sky-500/15 px-1.5 py-1.5 text-left text-sky-700 shadow-sm shadow-sky-500/10 normal-case tracking-normal transition-none hover:bg-sky-500/20':
        {},
    },


    '.lingo-sidebar-course__media': {
      '@apply flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-neutral-200 bg-neutral-50':
        {},
    },
    '.lingo-sidebar-course__copy': {
      '@apply flex min-w-0 flex-1 flex-col justify-center gap-0.5 self-stretch': {},
    },
    /** Title-only rows: vertically center against the cover thumb. */
    '.lingo-sidebar-course__copy--solo': {
      '@apply justify-center': {},
    },
    '.lingo-sidebar-course__title': {
      '@apply block min-w-0 truncate text-sm font-bold leading-snug text-inherit': {},
    },
    '.lingo-sidebar-course__desc': {
      '@apply block min-w-0 truncate text-xs font-normal leading-snug text-slate-500': {},
    },

    '.lingo-btn--sidebar-course-active .lingo-sidebar-course__desc': {
      '@apply text-sky-700/70': {},
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
    /**
     * Mobile top bar — same chrome as `.app-header-bar` (logo left + logout),
     * sticky in-flow so `.lingo-main` needs no top offset. Drawer toggle only.
     */
    '.lingo-mobile-header': {
      '@apply sticky top-0 z-50 w-full shrink-0 border-b border-gray-200 bg-white lg:hidden': {},
    },
    '.lingo-mobile-header-inner': {
      '@apply mx-auto w-full max-w-none px-1 sm:px-2': {},
    },
    /**
     * Desktop left rail (fixed) + drawer body on small screens.
     * z-40 keeps the rail above main content; never rely on in-flow placement alone.
     */
    '.lingo-sidebar-rail': {
      '@apply flex h-full w-full flex-col border-r-2 border-slate-200 bg-white px-2 lg:fixed lg:left-0 lg:top-0 lg:z-40 lg:h-svh lg:w-[256px]':
        {},
    },
    '.lingo-sidebar-nav': {
      /* Desktop rail needs its own top inset; drawer gets that from `.lingo-drawer-toolbar`. */
      '@apply flex min-h-0 flex-1 flex-col gap-y-1 overflow-y-auto pt-2 pb-2 lg:pt-3 lg:pb-3': {},
    },
    /** Divides “all courses” from the recent/popular course list. */
    '.lingo-sidebar-nav__separator': {
      '@apply my-2 shrink-0 border-t-2 border-slate-200': {},
    },
    /** Pushes the classic-collections link to the bottom of the rail. */
    '.lingo-sidebar-nav__spacer': {
      '@apply min-h-2 flex-1': {},
    },

    /** Main column: left inset for desktop rail. No bottom pad (tab bar removed). */
    '.lingo-main': {
      '@apply min-h-0 flex-1 pb-0 lg:pl-[256px]': {},
    },
    /**
     * Width shell only (no padding). Inset lives on the scroll/content child:
     * `.feed-page__body` or `.lingo-feed-layout` — never both.
     */
    '.lingo-main-inner': {
      '@apply mx-auto flex h-full min-h-0 w-full max-w-[1056px] flex-col': {},
    },
    /**
     * Fill+scroll Lingo pages (`feed-page` + `FeedPageBody`): body is the pad owner.
     * Slightly roomier than dictionary `px-3 pt-3`, with bottom breath (no FAB gutter).
     */
    '.lingo-main-inner .feed-page__header': {
      '@apply px-4 lg:px-6': {},
    },
    '.lingo-main-inner .feed-page__body': {
      '@apply px-4 pt-4 pb-4 lg:px-6': {},
    },
    '.lingo-main-inner .feed-page__pagination': {
      '@apply px-4 pr-4 lg:px-6': {},
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
    /**
     * Shared Lingo card surface — thin border + soft shadow (not chunky clone borders).
     * No padding: callers add `p-4` (see LingoPromo / StudyStreak / LingoQuestsPanel).
     */
    '.lingo-card': {
      '@apply rounded-xl border border-slate-200/90 bg-white shadow-sm shadow-slate-200/60': {},
    },
    /**
     * Course pick tile (clone: courses/card.tsx) — plain button, not ui-btn.
     * Top-aligned fixed slots (media → title → desc) so a grid of cards stays even.
     * Active = currently selected course (clone Check); smaller corner badge + sky border.
     */
    '.lingo-course-card': {
      '@apply relative flex h-full min-h-[217px] min-w-0 w-full cursor-pointer flex-col items-center justify-start rounded-xl border border-slate-200/90 bg-white p-3 pb-5 text-left shadow-sm shadow-slate-200/60 transition hover:-translate-y-0.5 hover:bg-slate-50/80 hover:shadow-md hover:shadow-slate-200/80 active:translate-y-0 active:shadow-sm disabled:pointer-events-none disabled:opacity-50 lg:min-w-[200px]':
        {},
    },
    '.lingo-course-card--active': {
      '@apply border-sky-400 bg-sky-500/5 shadow-md shadow-sky-500/15': {},
    },

    '.lingo-course-card__media': {
      '@apply relative mt-1 flex h-[70px] w-[93px] shrink-0 items-center justify-center overflow-hidden rounded-lg border border-neutral-200/80 bg-neutral-50 shadow-sm':
        {},
    },
    /**
     * Current-course mark — direct child of the card button (inside padding).
     * Dumbbell matches Learn / “actively studying this course”.
     */
    '.lingo-course-card__active-badge': {
      '@apply absolute right-3.5 top-3.5 z-10 flex h-7 w-7 items-center justify-center rounded-lg bg-green-600 text-white shadow-sm':
        {},
    },
    '.lingo-course-card__copy': {
      '@apply mt-3 flex w-full min-w-0 flex-col items-center gap-1 px-0.5 text-center': {},
    },
    /** Two-line title slot (always reserved) with ellipsis. */
    '.lingo-course-card__title': {
      '@apply w-full min-h-[2.75rem] overflow-hidden text-sm font-bold leading-snug text-neutral-700 line-clamp-2':
        {},
    },
    /** Two-line description slot (always reserved, even when empty) with ellipsis. */
    '.lingo-course-card__desc': {
      '@apply card-description w-full min-h-[2.5rem] overflow-hidden text-center leading-snug line-clamp-2':
        {},
    },
    /** Unit / collection banner on the learn ladder. */
    '.lingo-unit-banner': {
      '@apply flex w-full flex-col gap-4 rounded-xl bg-green-500 p-5 text-white shadow-md shadow-green-600/25 sm:flex-row sm:items-center sm:justify-between':
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
    /** In-flow close row (not absolute overlay) — brandbook: `.lingo-icon-btn`, no Aqua pills. */
    '.lingo-drawer-toolbar': {
      '@apply flex shrink-0 items-center justify-end border-b border-slate-200 px-3 py-2': {},
    },
    '.lingo-drawer-backdrop': {
      '@apply fixed inset-0 z-[100] bg-black/50 lg:hidden': {},
    },
    /**
     * Optional main + aside (leaderboard, levels, quests).
     * Pad owner when not using `feed-page` — do not also pad `.lingo-main-inner`.
     */
    '.lingo-feed-layout': {
      '@apply flex w-full flex-col gap-8 px-4 py-4 lg:px-6': {},
      '--lingo-feed-inline-pad': '1rem',
      '--lingo-feed-aside-w': '368px',
      '--lingo-feed-col-gap': '3rem',
      '@screen lg': {
        '--lingo-feed-inline-pad': '1.5rem',
      },
    },
    /**
     * DOM: aside then main.
     * Mobile `flex-col`: streak/hearts first (visible without scrolling past the grid).
     * Desktop `flex-row-reverse`: main | aside (rail on the right).
     */
    '.lingo-feed-layout__body': {
      '@apply flex w-full flex-col gap-6 lg:flex-row-reverse lg:gap-12': {},
    },
    /**
     * Levels: sticky full-bleed header, then ladder.
     * Drop outer vertical pad on this variant — header sits flush under the app bar.
     */
    '.lingo-feed-layout--with-header': {
      '@apply gap-0 py-0': {},
    },
    '.lingo-feed-layout--with-header .lingo-feed-layout__body': {
      '@apply pt-3 lg:pt-6': {},
    },
    /** Keep streak card below the sticky header while scrolling. */
    '.lingo-feed-layout--with-header .lingo-feed-layout__aside': {
      '@screen lg': {
        top: '5rem',
      },
    },
    /**
     * Full-bleed sticky top bar. Owns the Lensisku header/footer stripe wash
     * (same as App.vue `header, footer`) across the whole feed width.
     * Inset: `p-2` only — no extra pad on `__header-row`.
     */
    '.lingo-feed-layout__header-band': {
      '@apply sticky top-0 z-40 shrink-0 p-2 lg:z-50': {},
      position: 'sticky',
      width: 'calc(100% + 2 * var(--lingo-feed-inline-pad))',
      maxWidth: 'none',
      marginLeft: 'calc(-1 * var(--lingo-feed-inline-pad))',
      marginRight: 'calc(-1 * var(--lingo-feed-inline-pad))',
      background: 'repeating-linear-gradient(#f8f8f8, #f8f8f8 4px, #ffffff 4px, #ffffff 8px)',
      boxShadow: 'inset 0 -2px 0 0 rgb(226 232 240), 0 4px 12px -6px rgb(15 23 42 / 0.12)',
    },

    '.lingo-feed-layout__header-row': {
      '@apply relative z-[1] flex w-full flex-col gap-2 lg:flex-row lg:items-center lg:gap-3':
        {},
      '@screen lg': {
        gap: 'var(--lingo-feed-col-gap)',
      },
    },
    '.lingo-feed-layout__header-main': {
      '@apply min-w-0 flex-1': {},
    },
    '.lingo-feed-layout__header-aside': {
      '@apply hidden w-full shrink-0 lg:block': {},
      '@screen lg': {
        width: 'var(--lingo-feed-aside-w)',
      },
    },
    /** Title/actions only — stripe wash lives on `__header-band`, not here. */
    '.lingo-levels-header': {
      '@apply flex w-full items-center justify-between gap-3': {},
      background: 'transparent',
      backgroundImage: 'none',
    },


    /** Progress + streak: above main on small screens; sticky right rail on lg. */
    '.lingo-feed-layout__aside': {
      '@apply w-full shrink-0 lg:sticky lg:top-4 lg:w-[368px] lg:self-start': {},
    },
    '.lingo-feed-layout__main': {
      '@apply relative min-w-0 flex-1': {},
    },

    /** Empty / placeholder panels inside Lingo. */
    '.lingo-panel': {
      '@apply w-full rounded-xl border border-slate-200/90 bg-white p-4 text-center text-slate-500 shadow-sm shadow-slate-200/60':
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
    /** Portrait LTD icons: fixed box + background image (avoids CLS from late image layout). */
    '.lingo-study-mascot': {
      '@apply h-16 w-14 shrink-0 bg-contain bg-center bg-no-repeat sm:h-20 sm:w-[4.25rem] lg:h-[88px] lg:w-[72px]':
        {},
    },
    '.lingo-study-question-bubble': {
      '@apply relative min-w-0 flex-1 rounded-xl border border-neutral-200/90 bg-white px-4 py-2 text-sm text-neutral-700 shadow-sm shadow-slate-200/50 lg:text-base':
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
