/**
 * Aqua (glossy) theme: `aqua-base`, semantic `btn-aqua-*`, segment geometry, toggle shadow.
 * Imported by `tailwind.config.js` — keep machine details here, not scattered in the main config.
 */

/** Controls in a button bar (segment joins). */
export const btnGroupControlOfList = 'button, a'

/** Toggle “off” shadow (`buttonUiThemeLayer` + `btn-aqua-white`). */
export const aquaToggleOffShadow =
  '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
  '0 0.125em 0.125em hsla(0, 0%, 36.7%, 0.5),' +
  'inset 0 0.15em 0.35em rgba(255, 255, 255, 0.7),' +
  'inset 0 -0.05em 0.2em rgba(0, 0, 0, 0.06)'

/**
 * Shared aqua segment geometry for `.btn-group-forced .ui-btn--group-item` (aqua theme).
 */
export function buildAquaUiBtnGroupItemGeometry(list = btnGroupControlOfList) {
  return {
    borderRadius: 0,
    '&::after': {
      background: 'none',
    },
    [`&:nth-child(1 of ${list}):not(:nth-last-child(1 of ${list}))`]: {
      '@apply z-10': {},
      borderTopLeftRadius: '1000px',
      borderBottomLeftRadius: '1000px',
      borderTopRightRadius: 0,
      borderBottomRightRadius: 0,
      boxShadow:
        '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
        '0 0.125em 0.125em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.5),' +
        'inset 0.25em 0.25em 0.5em hsla(calc(var(--aqua-hue, 215) + 4), 100%, 9.6%, 0.8),' +
        'inset 0.25em 0.375em 0.5em -0.125em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.75)',
      '&::before': {
        borderRadius: '2em 0 0 0.5em',
        left: '0.4375em',
        width: 'calc(100% - 0.4375em)',
        transform: 'none',
      },
      '&::after': {
        borderRadius: '0.75em 0 0 0.75em',
        left: '0.4375em',
        width: 'calc(100% - 0.4375em)',
        transform: 'none',
      },
    },
    [`&:nth-last-child(1 of ${list}):not(:nth-child(1 of ${list}))`]: {
      '@apply z-10': {},
      borderTopLeftRadius: 0,
      borderBottomLeftRadius: 0,
      borderTopRightRadius: '1000px',
      borderBottomRightRadius: '1000px',
      boxShadow:
        '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
        '0 0.125em 0.125em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.5),' +
        'inset -0.25em 0.25em 0.5em hsla(calc(var(--aqua-hue, 215) + 4), 100%, 9.6%, 0.8),' +
        'inset -0.25em 0.375em 0.5em -0.125em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.75)',
      '&::before': {
        borderRadius: '0 2em 0.5em 0',
        left: 'auto',
        right: '0.4375em',
        width: 'calc(100% - 0.4375em)',
        transform: 'none',
      },
      '&::after': {
        borderRadius: '0 0.75em 0.75em 0',
        left: 'auto',
        right: '0.4375em',
        width: 'calc(100% - 0.4375em)',
        transform: 'none',
      },
    },
    [`&:not(:nth-child(1 of ${list})):not(:nth-last-child(1 of ${list}))`]: {
      '@apply z-10': {},
      boxShadow:
        '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
        '0 0.125em 0.125em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.5),' +
        'inset 0 0.25em 0.5em -0.25em hsla(calc(var(--aqua-hue, 215) + 4), 100%, 9.6%, 0.8),' +
        'inset 0 0.375em 0.5em -0.25em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.75)',
      '&::before': {
        borderRadius: 0,
        left: 0,
        width: '100%',
        transform: 'none',
      },
      '&::after': {
        borderRadius: 0,
        left: 0,
        width: '100%',
        transform: 'none',
      },
    },
    [`&:nth-child(1 of ${list}):nth-last-child(1 of ${list})`]: {
      borderRadius: '1000px',
    },
  }
}

/** Glossy chrome bases (Tailwind `addBase`). */
export function buildAquaBaseLayer() {
  return {
    '.aqua-base': {
      '@apply flex items-center justify-center h-6 select-none whitespace-nowrap text-black select-none text-sm font-medium font-sans transition-all px-3 gap-2':
        {},
      textOverflow: 'ellipsis',
      gridRow: '1',
      borderRadius: '1000px',
      position: 'relative',
      overflow: 'hidden',
      cursor: 'default',
      outline: 'none',
      boxShadow:
        '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
        '0 0.125em 0.125em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.5),' +
        'inset 0 0.25em 0.5em hsla(calc(var(--aqua-hue, 215) + 4), 100%, 9.6%, 0.8),' +
        'inset 0 0.375em 0.5em 0.25em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.75)',
      '&:not(:disabled)': {
        '@apply hover:brightness-105 hover:saturate-150 active:scale-x-[1.02]': {},
      },
      '& span:not(.sr-only)': {
        position: 'relative',
        top: 0,
        zIndex: 1,
        whiteSpace: 'nowrap',
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        maxWidth: '100%',
      },
      '&:before': {
        content: "''",
        position: 'absolute',
        left: '50%',
        transform: 'translateX(-50%)',
        height: '33%',
        background: 'linear-gradient(rgba(255, 255, 255, 0.9), rgba(255, 255, 255, 0.3))',
        width: 'calc(100% - 0.875em)',
        borderRadius: '2em 2em 0.5em 0.5em',
        top: '5%',
        zIndex: '2',
      },
      '&.rounded-corner:before': {
        borderRadius: '30em 30em 2em 2em',
      },
      '&:after': {
        content: "''",
        position: 'absolute',
        left: '50%',
        transform: 'translateX(-50%)',
        height: '33%',
        background: 'linear-gradient(rgba(255, 255, 255, 0.2), rgba(255, 255, 255, 0.5))',
        width: 'calc(100% - 1.25em)',
        borderRadius: '0.75em',
        bottom: '10%',
        filter: 'blur(0.75px)',
      },
      '&:focus, &:active': {
        boxShadow:
          '0 0.35em 0.45em rgba(2, 10, 26, 0.28),' +
          '0 0.08em 0.18em hsla(var(--aqua-hue, 215), 85%, 46%, 0.42),' +
          'inset 0 0.22em 0.45em hsla(calc(var(--aqua-hue, 215) - 8), 75%, 22%, 0.65),' +
          'inset 0 0.4em 0.55em 0.2em hsla(var(--aqua-hue, 215), 95%, 68%, 0.28),' +
          '0 0 0.5em hsla(var(--aqua-hue, 215), 78%, 58%, 0.45)',
        '&:disabled': {
          boxShadow:
            '0 0.375em 0.5em rgba(0, 0, 0, 0.2), 0 0.125em 0.125em rgba(0, 0, 0, 0.3), inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4), inset 0 0.375em 0.5em 0.25em #BBBBBB',
        },
      },
      '&[disabled]:not([disabled="false"]), .disabled': {
        opacity: 0.5,
        background:
          'linear-gradient(rgba(160, 160, 160, 0.625), rgba(255, 255, 255, 0.625)) !important',
        boxShadow:
          '0 0.375em 0.5em rgba(0, 0, 0, 0.2), 0 0.125em 0.125em rgba(0, 0, 0, 0.3), inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4), inset 0 0.375em 0.5em 0.25em #BBBBBB !important',
        '&:hover, &:focus, &:active': {
          transform: 'none !important',
          filter: 'none !important',
          background:
            'linear-gradient(rgba(160, 160, 160, 0.625), rgba(255, 255, 255, 0.625)) !important',
          boxShadow:
            '0 0.375em 0.5em rgba(0, 0, 0, 0.2), 0 0.125em 0.125em rgba(0, 0, 0, 0.3), inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4), inset 0 0.375em 0.5em 0.25em #BBBBBB !important',
        },
      },
    },
    '.aqua-base-secondary': {
      '@apply flex items-center justify-center h-6 select-none whitespace-nowrap text-black select-none text-sm font-medium font-sans transition-all px-3 gap-2':
        {},
      textOverflow: 'ellipsis',
      gridRow: '1',
      borderRadius: '1000px',
      position: 'relative',
      overflow: 'hidden',
      cursor: 'default',
      outline: 'none',
      boxShadow:
        '0 0.375em 0.5em rgba(0, 0, 0, 0.2),' +
        '0 0.125em 0.125em rgba(0, 0, 0, 0.3),' +
        'inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4),' +
        'inset 0 0.375em 0.5em 0.25em #BBBBBB',
      '&:not(:disabled)': {
        '@apply hover:brightness-110 active:scale-x-[1.02]': {},
      },
      '& span:not(.sr-only)': {
        position: 'relative',
        zIndex: 1,
        textShadow: '0 0.25em 0.2em rgba(0, 0, 0, 0.25)',
        whiteSpace: 'nowrap',
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        maxWidth: '100%',
      },
      '&:before': {
        content: "''",
        position: 'absolute',
        left: '50%',
        transform: 'translateX(-50%)',
        height: '33%',
        background: 'linear-gradient(rgba(255, 255, 255, 0.9), rgba(255, 255, 255, 0.3))',
        width: 'calc(100% - 0.875em)',
        borderRadius: '2em 2em 0.5em 0.5em',
        top: '5%',
        filter: 'blur(1px)',
        zIndex: '2',
      },
      '&.rounded-corner:before': {
        borderRadius: '30em 30em 2em 2em',
      },
      '&:after': {
        content: "''",
        position: 'absolute',
        left: '50%',
        transform: 'translateX(-50%)',
        height: '33%',
        background: 'linear-gradient(rgba(255, 255, 255, 0.2), rgba(255, 255, 255, 0.5))',
        width: 'calc(100% - 1.25em)',
        borderRadius: '0.75em',
        bottom: '10%',
        filter: 'blur(3px)',
      },
      '&:focus, &:active': {
        boxShadow:
          '0 0.375em 0.5em rgba(0, 0, 0, 0.2),' +
          '0 0.125em 0.125em rgba(0, 0, 0, 0.3),' +
          'inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4),' +
          'inset 0 0.375em 0.5em 0.25em #BBBBBB,' +
          '0 0 0.5em rgba(0, 0, 0, 0.25)',
        '&:disabled': {
          boxShadow:
            '0 0.375em 0.5em rgba(0, 0, 0, 0.2), 0 0.125em 0.125em rgba(0, 0, 0, 0.3), inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4), inset 0 0.375em 0.5em 0.25em #BBBBBB',
        },
      },
      '&[disabled]:not([disabled="false"]), .disabled': {
        opacity: 0.5,
        background:
          'linear-gradient(rgba(160, 160, 160, 0.625), rgba(255, 255, 255, 0.625)) !important',
        boxShadow:
          '0 0.375em 0.5em rgba(0, 0, 0, 0.2), 0 0.125em 0.125em rgba(0, 0, 0, 0.3), inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4), inset 0 0.375em 0.5em 0.25em #BBBBBB !important',
        '&:hover, &:focus, &:active': {
          transform: 'none !important',
          filter: 'none !important',
          background:
            'linear-gradient(rgba(160, 160, 160, 0.625), rgba(255, 255, 255, 0.625)) !important',
          boxShadow:
            '0 0.375em 0.5em rgba(0, 0, 0, 0.2), 0 0.125em 0.125em rgba(0, 0, 0, 0.3), inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4), inset 0 0.375em 0.5em 0.25em #BBBBBB !important',
        },
      },
    },
  }
}

/** Semantic `btn-aqua-*` fills (`addComponents`). */
export function buildAquaButtonPrimitives() {
  return {
    '.btn-aqua': {
      '@apply aqua-base': {},
    },
    '.btn-aqua-primary': {
      '@apply aqua-base': {},
      background:
        'linear-gradient(rgba(0, 65, 184, 0.625), rgba(45, 115, 199, 0.625), rgba(33, 160, 196, 0.625))',
      '--aqua-hue': '217',
    },
    '.btn-aqua-secondary': {
      '@apply aqua-base-secondary': {},
      background: 'linear-gradient(rgba(160, 160, 160, 0.625), rgba(255, 255, 255, 0.625))',
    },
    '.btn-aqua-yellow': {
      '@apply aqua-base bg-yellow-500': {},
      '--aqua-hue': '45',
    },
    '.btn-aqua-blue': {
      '@apply aqua-base bg-blue-400': {},
      '--aqua-hue': '217',
    },
    '.btn-aqua-sky': {
      '@apply aqua-base bg-sky-400': {},
      '--aqua-hue': '199',
    },
    '.btn-aqua-get': {
      '@apply aqua-base bg-sky-300': {},
      '--aqua-hue': '199',
    },
    '.btn-aqua-purple': {
      '@apply aqua-base bg-purple-400': {},
      '--aqua-hue': '270',
    },
    '.btn-aqua-simple': {
      '@apply aqua-base': {},
      boxShadow:
        '0 0.35em 0.45em rgba(2, 10, 26, 0.28),' +
        '0 0.08em 0.18em hsla(var(--aqua-hue, 215), 85%, 46%, 0.42),' +
        'inset 0 0.22em 0.45em hsla(calc(var(--aqua-hue, 215) - 8), 75%, 22%, 0.65),' +
        'inset 0 0.4em 0.55em 0.2em hsla(var(--aqua-hue, 215), 95%, 68%, 0.28)',
    },
    '.btn-aqua-emerald': {
      '@apply aqua-base bg-emerald-400': {},
      '--aqua-hue': '160',
    },
    '.btn-aqua-rose': {
      '@apply aqua-base bg-rose-400': {},
      '--aqua-hue': '350',
    },
    '.btn-aqua-cornflower': {
      '@apply aqua-base bg-cornflower-400': {},
      '--aqua-hue': '219',
    },
    '.btn-aqua-orange': {
      '@apply aqua-base bg-orange-400': {},
      '--aqua-hue': '27',
    },
    '.btn-aqua-red': {
      '@apply aqua-base bg-red-400': {},
      '--aqua-hue': '0',
    },
    '.btn-aqua-amber': {
      '@apply aqua-base bg-amber-400': {},
      '--aqua-hue': '38',
    },
    '.btn-aqua-lime': {
      '@apply aqua-base bg-lime-400': {},
      '--aqua-hue': '84',
    },
    '.btn-aqua-teal': {
      '@apply aqua-base bg-teal-400': {},
      '--aqua-hue': '173',
    },
    '.btn-aqua-cyan': {
      '@apply aqua-base bg-cyan-400': {},
      '--aqua-hue': '188',
    },
    '.btn-aqua-indigo': {
      '@apply aqua-base bg-indigo-400': {},
      '--aqua-hue': '243',
    },
    '.btn-aqua-violet': {
      '@apply aqua-base bg-violet-400': {},
      '--aqua-hue': '260',
    },
    '.btn-aqua-fuchsia': {
      '@apply aqua-base bg-fuchsia-400': {},
      '--aqua-hue': '292',
    },
    '.btn-aqua-pink': {
      '@apply aqua-base bg-pink-400': {},
      '--aqua-hue': '330',
    },
    '.btn-aqua-zinc': {
      '@apply aqua-base bg-zinc-300': {},
      '--aqua-hue': '240',
    },
    '.btn-aqua-slate': {
      '@apply aqua-base bg-slate-400': {},
      '--aqua-hue': '215',
    },
    '.btn-aqua-gray': {
      '@apply aqua-base bg-gray-800 !text-white': {},
      boxShadow:
        '0 0.375em 0.5em rgba(0, 0, 0, 0.4),' +
        '0 0.125em 0.125em rgba(0, 0, 0, 0.3),' +
        'inset 0 0.25em 0.5em rgba(0, 0, 0, 0.8),' +
        'inset 0 0.375em 0.5em 0.25em rgba(255, 255, 255, 0.1)',
      '&:focus, &:active': {
        boxShadow:
          '0 0.375em 0.5em rgba(0, 0, 0, 0.4),' +
          '0 0.125em 0.125em rgba(0, 0, 0, 0.3),' +
          'inset 0 0.25em 0.5em rgba(0, 0, 0, 0.8),' +
          'inset 0 0.375em 0.5em 0.25em rgba(255, 255, 255, 0.15),' +
          '0 0 0.5em rgba(255, 255, 255, 0.2)',
      },
    },
    '.btn-aqua-white': {
      '@apply aqua-base-secondary': {},
      boxShadow: aquaToggleOffShadow,
      background: 'linear-gradient(rgba(180, 180, 180, 1), rgba(255, 255, 255, 0.625))',
      '&::after': {
        background: 'none',
      },
    },
  }
}

/**
 * Pill row layout (both themes) + aqua segment hooks (`addComponents`).
 * `.btn-group-forced` uses `btnGroupControlOfList` for `:nth-child` / `:is()` selectors.
 */
export function buildAquaButtonGroupLayer() {
  const list = btnGroupControlOfList
  return {
    '.btn-group-item': {
      '@apply border rounded-full': {},
      '&:not(:disabled):active': {
        '@apply scale-x-[1.02]': {},
      },
    },
    /**
     * Default group: wrapped pill row(s), gap between actions — no horizontal scrollbar, no broken
     * segment joins when wrapping. For a single fused segment bar (filters, compact controls), use
     * `.btn-group-forced`.
     */
    '.btn-group': {
      '@apply flex flex-wrap gap-2 min-w-0': {},
    },
    /** Fused segment control at every breakpoint (spacing + item joins; use where wrapping must stay one bar). */
    '.btn-group-forced': {
      '@apply gap-0': {},
      [`& > :nth-child(1 of ${list})`]: {
        '@apply rounded-l-full': {},
      },
      [`& > :nth-last-child(1 of ${list})`]: {
        '@apply rounded-r-full': {},
      },
      [`& > :is(${list}):not(:nth-last-child(1 of ${list}))`]: {
        '@apply border-r-0': {},
      },
      '& .btn-group-item, & .ui-btn--group-item': {
        '@apply rounded-none': {},
      },
    },
    /** Hook for aqua theme map (`ui-btn--group-item`); stacked primitives supply visuals; segment CSS only in `.btn-group-forced`. */
    '.btn-aqua-group-item': {
      '@apply shrink-0': {},
    },
    '.btn-aqua-toggle': {
      '&.active': {
        '@apply aqua-base bg-red-400': {},
        '--aqua-hue': '0',
      },
      '&:not(.active)': {
        '@apply btn-aqua-white': {},
      },
    },
  }
}
