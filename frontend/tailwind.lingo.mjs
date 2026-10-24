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
    /** Defer / snooze — amber so it stays distinct from Check (green) and Forgot (rose). */
    '.lingo-btn--warn': {
      '@apply border-b-4 border-amber-600 bg-amber-400 text-amber-950 hover:bg-amber-400/90 active:border-b-0':
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
     * Hearts readout — same shell as `.lingo-icon-btn` / header back, but not a control
     * (no hover fill, no focus ring). Pair with `.icon-btn-header-back--compact` on study.
     */
    '.lingo-hearts-badge': {
      '@apply lingo-icon-btn min-w-10 cursor-default gap-0.5 px-1.5 text-sm font-bold text-rose-500 hover:bg-transparent focus-visible:ring-0 focus-visible:ring-offset-0':
        {},
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
     * Course rail. Drawer body on small screens (full width of the drawer).
     * Desktop placement (sticky, beside the column, not the viewport edge)
     * lives on `.lingo-shell-cluster > .lingo-sidebar-rail`.
     */
    '.lingo-sidebar-rail': {
      '@apply flex h-full w-full flex-col border-r-2 border-slate-200 bg-white px-2': {},
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

    /**
     * Full-width scrollport (scrollbar on the viewport edge).
     * The cluster is the rail + page column, centered, so the rail stays
     * against the column even when that pair is inset from the viewport.
     */
    '.lingo-shell-scroll': {
      '@apply min-h-0 w-full flex-1 overscroll-y-contain bg-white': {},
      overflowX: 'hidden',
      overflowY: 'auto',
    },
    '.lingo-shell-cluster': {
      display: 'grid',
      gridTemplateColumns: 'minmax(0, 1056px)',
      justifyContent: 'center',
      width: '100%',
      '@screen lg': {
        gridTemplateColumns: '256px minmax(0, 1056px)',
        width: 'max-content',
        maxWidth: '100%',
        marginInline: 'auto',
        justifyContent: 'start',
      },
    },
    '.lingo-shell-cluster > .lingo-sidebar-rail': {
      '@screen lg': {
        position: 'sticky',
        top: '0',
        zIndex: '40',
        alignSelf: 'start',
        height: '100svh',
        width: '256px',
      },
    },

    /**
     * Page column beside the rail. No left inset: the rail is in the same row.
     * Width cap is the cluster grid (256 + 1056), not padding.
     */
    '.lingo-main': {
      '@apply min-h-0 min-w-0 pb-0': {},
    },
    /**
     * Width shell only (no padding). Height follows the page so the shell
     * scrollport can grow. Inset lives on `.lingo-feed-layout`, not here.
     */
    '.lingo-main-inner': {
      '@apply mx-auto flex w-full max-w-[1056px] flex-col': {},
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
     * No padding: callers add their own (`StudyStreak` uses `.card-streak` pad).
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
    /** Three-line description slot (always reserved, even when empty) with ellipsis. */
    '.lingo-course-card__desc': {
      '@apply card-description w-full min-h-[3.75rem] overflow-hidden text-center leading-snug line-clamp-3':
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
    /**
     * Open node. Hue comes from `--lingo-node-h` (hash of the ladder index).
     * Saturation and lightness stay at the old green-500 / green-600 pair.
     */
    '.lingo-lesson-node--open': {
      '@apply text-white active:border-b-0': {},
      backgroundColor: 'hsl(var(--lingo-node-h, 142) 71% 45%)',
      borderColor: 'hsl(var(--lingo-node-h, 142) 76% 36%)',
    },
    '.lingo-lesson-node--open:hover': {
      backgroundColor: 'hsl(var(--lingo-node-h, 142) 71% 40%)',
    },
    /** Current-lesson progress ring; same hue as the node inside it. */
    '.lingo-lesson-progress': {
      stroke: 'hsl(var(--lingo-node-h, 142) 71% 60%)',
    },
    '.lingo-lesson-node--locked': {
      '@apply border-neutral-400 bg-neutral-200 text-neutral-400 active:border-b-0': {},
    },
    /**
     * Level name under a ladder node — always visible (not hover-only).
     * Wide enough for a short sentence of context, clamped to three lines.
     * Mobile stays narrower so a zig-zag node near the edge is not clipped.
     */
    '.lingo-lesson-label': {
      '@apply mt-1.5 w-[11rem] max-w-[11rem] break-words text-center text-xs font-bold leading-snug text-neutral-700 line-clamp-3 sm:w-[14rem] sm:max-w-[14rem]':
        {},
    },
    '.lingo-lesson-label--locked': {
      '@apply text-neutral-400': {},
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
     * Courses / levels / leaderboard shell.
     * The scrollport is `.lingo-shell-scroll` (viewport edge). This shell does not
     * scroll. Content inset uses `--lingo-feed-inline-pad` on header / toolbar / `__body`.
     */
    '.lingo-feed-layout': {
      '@apply flex w-full flex-col gap-8': {},
      overflow: 'visible',
      '--lingo-feed-inline-pad': '0.5rem',
      '--lingo-feed-aside-w': '368px',
      '--lingo-feed-col-gap': '3rem',
      '@screen lg': {
        '--lingo-feed-inline-pad': '1.5rem',
      },
    },
    /** Non-header pages: light vertical breath (inline pad is on `__body`). */
    '.lingo-feed-layout:not(.lingo-feed-layout--with-header)': {
      paddingTop: '0.5rem',
      paddingBottom: '0.5rem',
    },
    /**
     * Body wrapper. Scrolling happens on `.lingo-shell-scroll`, not here,
     * so the scrollbar stays on the viewport edge.
     */
    '.lingo-feed-layout__scroll': {
      '@apply min-w-0': {},
      overflow: 'visible',
    },
    /** Header + optional toolbar, stuck to the top of the shell scrollport. */
    '.lingo-feed-layout__pin': {
      position: 'sticky',
      top: '0',
      zIndex: '30',
      backgroundColor: '#fff',
    },
    /**
     * DOM: aside then main. Owns the content inline pad (not the scroller).
     * Mobile `flex-col`: streak first. Desktop `flex-row-reverse`: main | aside.
     */
    '.lingo-feed-layout__body': {
      '@apply flex w-full flex-col gap-6 pb-4 lg:flex-row-reverse lg:gap-12 lg:pb-6': {},
      paddingLeft: 'var(--lingo-feed-inline-pad)',
      paddingRight: 'var(--lingo-feed-inline-pad)',
    },
    /** Header flush under app bar; gap lives on toolbar / scroll body. */
    '.lingo-feed-layout--with-header': {
      '@apply gap-0': {},
    },
    '.lingo-feed-layout--with-header:not(.lingo-feed-layout--with-toolbar) .lingo-feed-layout__scroll':
      {
        '@apply pt-3 lg:pt-6': {},
      },
    /**
     * Pinned full-bleed header stripe. Content inset on `__header-row`.
     */
    '.lingo-feed-layout__header-band': {
      '@apply relative z-40 shrink-0 py-2 lg:z-50': {},
      background: 'repeating-linear-gradient(#f8f8f8, #f8f8f8 4px, #ffffff 4px, #ffffff 8px)',
      boxShadow: 'inset 0 -2px 0 0 rgb(226 232 240), 0 4px 12px -6px rgb(15 23 42 / 0.12)',
    },

    '.lingo-feed-layout__header-row': {
      '@apply relative z-[1] flex w-full min-w-0 items-center gap-2': {},
      paddingLeft: 'var(--lingo-feed-inline-pad)',
      paddingRight: 'var(--lingo-feed-inline-pad)',
      '@screen lg': {
        gap: 'var(--lingo-feed-col-gap)',
      },
    },
    '.lingo-feed-layout__header-row > :first-child': {
      '@apply min-w-0 flex-1': {},
    },
    '.lingo-feed-layout__header-aside': {
      '@apply w-auto shrink-0': {},
      '@screen lg': {
        width: 'var(--lingo-feed-aside-w)',
      },
    },
    /** Pinned search / filters — same inline pad as body content. */
    '.lingo-feed-layout__toolbar': {
      '@apply shrink-0 py-3': {},
      paddingLeft: 'var(--lingo-feed-inline-pad)',
      paddingRight: 'var(--lingo-feed-inline-pad)',
    },
    /** Courses / collections switch on the Lingo catalog. */
    /** Same 40px height as `.lingo-icon-btn` so the header row does not CLS with sort. */
    '.lingo-catalog-tabs': {
      '@apply inline-flex h-10 w-fit max-w-full items-stretch self-center overflow-hidden rounded-xl border-2 border-b-4 border-slate-200 bg-white':
        {},
      boxSizing: 'border-box',
    },
    '.lingo-catalog-tabs__tab': {
      '@apply cursor-pointer border-0 bg-transparent px-4 py-0 text-xs font-bold uppercase tracking-wide text-slate-500 hover:bg-slate-50':
        {},
    },
    '.lingo-catalog-tabs__tab + .lingo-catalog-tabs__tab': {
      borderLeft: '2px solid rgb(226 232 240)',
    },
    '.lingo-catalog-tabs__tab--active': {
      '@apply bg-green-500 text-white hover:bg-green-500': {},
    },
    /** Native sort menu, same chunky shell as `.lingo-btn`. */
    '.lingo-select__control': {
      appearance: 'none',
      WebkitAppearance: 'none',
      height: '2.75rem',
      width: '100%',
      maxWidth: '100%',
      cursor: 'pointer',
      borderRadius: '0.75rem',
      border: '2px solid rgb(226 232 240)',
      borderBottomWidth: '4px',
      backgroundColor: '#fff',
      backgroundImage:
        "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='%2364748b' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E\")",
      backgroundRepeat: 'no-repeat',
      backgroundPosition: 'right 0.75rem center',
      backgroundSize: '1rem',
      padding: '0 2.25rem 0 0.875rem',
      fontSize: '0.875rem',
      fontWeight: '700',
      color: 'rgb(64 64 64)',
      '@screen sm': {
        width: 'auto',
        minWidth: '14rem',
      },
    },
    '.lingo-select__control:focus': {
      outline: 'none',
      boxShadow: '0 0 0 2px #fff, 0 0 0 4px rgb(34 197 94)',
    },
    /** Levels toolbar — flex only; L/R inset is on `__header-row`. */
    '.lingo-levels-header': {
      '@apply flex w-full items-center justify-between gap-3': {},
    },

    /** Progress + streak: above main on small screens; right rail on lg. */
    '.lingo-feed-layout__aside': {
      '@apply w-full shrink-0 lg:w-[368px] lg:self-start': {},
    },
    '.lingo-feed-layout__main': {
      '@apply relative min-w-0 flex-1': {},
    },

    /** Empty / placeholder panels inside Lingo. */
    '.lingo-panel': {
      '@apply w-full rounded-xl border border-slate-200/90 bg-white p-4 text-center text-slate-500 shadow-sm shadow-slate-200/60':
        {},
    },


    /**
     * Fixed footer shell (clone `footer.tsx`) — Tailwind spacing scale only (`h-24` /
     * `lg:h-36`). Both action + meta rows stay `h-9` so Check↔Rate does not CLS.
     */
    '.lingo-study-footer--compact': {
      '@apply h-24': {},
    },
    '.lingo-study-footer--roomy': {
      '@apply h-24 lg:h-36': {},
    },
    /** Same max-w as `.lingo-study-chrome-inner`; packs to the bottom of the shell. */
    '.lingo-study-footer-chrome': {
      '@apply mx-auto box-border flex h-full min-h-0 w-full max-w-[600px] flex-col items-stretch justify-end gap-y-1.5 overflow-hidden px-2 py-1.5 sm:px-3':
        {},
    },
    /** Three equal action cells; Check shares the third cell with Easy. */
    '.lingo-study-rate-row': {
      '@apply grid h-9 w-full shrink-0 grid-cols-3 items-center gap-1.5 sm:gap-2': {},
    },
    '.lingo-study-footer-meta': {
      '@apply grid h-9 w-full shrink-0 grid-cols-[auto_1fr_auto] items-center gap-2': {},
    },
    '.lingo-study-footer-meta__prompt': {
      '@apply min-w-0 truncate text-center text-[10px] font-bold uppercase tracking-wide text-slate-500 sm:text-xs':
        {},
    },
    /** Clone lesson title (`quiz.tsx` h1). */
    '.lingo-study-lesson-title': {
      '@apply shrink-0 text-center text-lg font-bold text-neutral-700 lg:text-start lg:text-3xl':
        {},
    },
    /**
     * Study dialogue rows: mascot + bubble.
     * Question and reply use different LTD prefixes (`lin` / `lif` / `bla`).
     * Caret sits in the gap (not clipped by the scroll stack).
     * Opposite-side inset so Q/A rows stagger like a chat dialogue.
     */
    '.lingo-study-question-row': {
      '@apply flex items-center gap-x-3 sm:gap-x-4': {},
    },
    /** Question (sprite left): leave room on the right. */
    '.lingo-study-question-row:not(.lingo-study-question-row--reply)': {
      '@apply pr-12 sm:pr-16 lg:pr-20': {},
    },
    /** Reply (sprite right): leave room on the left. */
    '.lingo-study-question-row--reply': {
      '@apply pl-12 sm:pl-16 lg:pl-20': {},
    },
    /** Portrait LTD icons: fixed box + background image (avoids CLS from late image layout). */
    '.lingo-study-mascot': {
      '@apply h-16 w-14 shrink-0 bg-contain bg-center bg-no-repeat sm:h-20 sm:w-[4.25rem] lg:h-[88px] lg:w-[72px]':
        {},
    },
    /** Stimulus / reply speech bubble. */
    '.lingo-study-question-bubble': {
      '@apply relative z-0 min-w-0 flex-1 rounded-xl border border-neutral-200/90 bg-white px-4 py-2 text-sm text-neutral-700 shadow-sm shadow-slate-200/50 lg:text-base':
        {},
    },
    /**
     * Speech-bubble caret toward the mascot (in the row gap).
     * Raw borders — @apply on ::before is flaky across Tailwind builds.
     */
    '.lingo-study-question-bubble::before': {
      content: '""',
      position: 'absolute',
      left: '-0.75rem',
      top: '50%',
      width: '0',
      height: '0',
      borderLeft: '8px solid transparent',
      borderRight: '8px solid transparent',
      borderTop: '8px solid #e5e5e5',
      transform: 'translateY(-50%) rotate(90deg)',
      zIndex: '1',
    },
    /**
     * Empty reply invite (mascot on the right). Caret flips to point at the sprite.
     * Reserved two-line body so the bubble is not a smashed one-liner before Check.
     */
    '.lingo-study-reply-bubble': {
      '@apply flex items-center': {},
    },
    '.lingo-study-reply-bubble::before': {
      content: '""',
      position: 'absolute',
      left: 'auto',
      right: '-0.75rem',
      top: '50%',
      width: '0',
      height: '0',
      borderLeft: '8px solid transparent',
      borderRight: '8px solid transparent',
      borderTop: '8px solid #e5e5e5',
      transform: 'translateY(-50%) rotate(-90deg)',
      zIndex: '1',
    },
    '.lingo-study-reply-invite': {
      '@apply w-full min-h-[2.75rem] text-sm leading-snug sm:min-h-[3rem] sm:text-base': {},
    },
    /** Revealed answer — same bubble geometry as the reply invite (no layout jump). */
    '.lingo-study-answer-bubble': {
      '@apply border-green-200 bg-green-50/80': {},
    },
    '.lingo-study-answer-bubble::before': {
      content: '""',
      position: 'absolute',
      left: 'auto',
      right: '-0.75rem',
      top: '50%',
      width: '0',
      height: '0',
      borderLeft: '8px solid transparent',
      borderRight: '8px solid transparent',
      borderTop: '8px solid #bbf7d0', // green-200
      transform: 'translateY(-50%) rotate(-90deg)',
      zIndex: '1',
    },
    '.lingo-study-answer-label': {
      '@apply mb-0.5 text-[10px] font-bold uppercase tracking-wide text-green-700 sm:text-xs': {},
    },
    /** Borderless field inside the reply bubble (two-line tall like the empty invite). */
    '.lingo-study-reply-input': {
      '@apply w-full min-h-[2.75rem] resize-none border-0 bg-transparent p-0 text-center text-sm leading-snug shadow-none outline-none ring-0 focus:border-0 focus:outline-none focus:ring-0 sm:min-h-[3rem] sm:text-base':
        {},
    },
    '.lingo-study-stimulus': {
      '@apply text-sm font-medium text-neutral-700 lg:text-base': {},
    },
    /**
     * Top-anchored column (not vertically centered) so revealing the answer bubble
     * does not shove the question mid-screen (clone keeps title + bubble fixed).
     */
    '.lingo-study-lesson-stack': {
      '@apply flex w-full max-h-full min-h-0 flex-col gap-y-4 pt-2 sm:gap-y-5 sm:pt-3 lg:gap-y-6 lg:pt-4':
        {},
    },
    /** Leaderboard / list avatar ring (clone: green border circle). */
    '.lingo-avatar': {
      '@apply mr-4 flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-full border-2 border-green-500 bg-green-100 text-sm font-bold text-green-700':
        {},
    },
    '.lingo-avatar img': {
      '@apply h-full w-full object-cover': {},
    },
    /**
     * Lingo dialog shell (study discussions, etc.) — not Lensisku `ModalComponent` /
     * Aqua close buttons. Pair with `LingoModal.vue`.
     */
    '.lingo-modal-backdrop': {
      '@apply fixed inset-0 z-[60] flex items-center justify-center bg-black/50 p-3 sm:p-4': {},
    },
    '.lingo-modal-panel': {
      '@apply flex max-h-[90vh] w-full max-w-2xl flex-col overflow-hidden rounded-2xl border-2 border-slate-200 bg-white shadow-lg':
        {},
    },
    '.lingo-modal-header': {
      '@apply flex shrink-0 items-center justify-between gap-3 border-b-2 border-slate-200 px-3 py-2.5 sm:px-4 sm:py-3':
        {},
    },
    '.lingo-modal-title': {
      '@apply min-w-0 truncate text-base font-bold text-neutral-700 sm:text-lg': {},
    },
    '.lingo-modal-body': {
      '@apply min-h-0 flex-1 overflow-y-auto px-3 py-3 sm:px-4 sm:py-4': {},
    },
    '.lingo-modal-footer': {
      '@apply shrink-0 border-t-2 border-slate-200 px-3 py-2.5 sm:px-4 sm:py-3': {},
    },
  }
}
