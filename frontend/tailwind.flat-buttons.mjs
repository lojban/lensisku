/**
 * Flat theme button primitives for Tailwind `addBase`.
 * Layers sit **behind** label text (z-index: -1) so hover fills never cover copy.
 */

const flatRing = (theme, color) => `color-mix(in srgb, ${color} 50%, transparent)`

const flatStandardHue = (theme, hue) => ({
  '--bf-a': theme(`colors.${hue}.100`),
  '--bf-b': theme(`colors.${hue}.50`),
  '--bf-br': theme(`colors.${hue}.400`),
  '--bf-ha': theme(`colors.${hue}.200`),
  '--bf-hb': theme(`colors.${hue}.100`),
  '--bf-hbr': theme(`colors.${hue}.600`),
  '--bf-aa': theme(`colors.${hue}.300`),
  '--bf-ab': theme(`colors.${hue}.200`),
  '--bf-abr': theme(`colors.${hue}.700`),
  '--bf-ring': flatRing(theme, theme(`colors.${hue}.400`)),
})

/** Shared `.btn-flat-surface` + semantic `btn-*` rows (only `color` + `--bf-*` differ). */
export function buildFlatButtonLayer(theme) {
  const ring = (c) => flatRing(theme, c)
  const emptyRippleBorder = ring(theme('colors.slate.500'))

  const surface = {
    '.btn-flat-surface': {
      '@apply btn-base border-solid': {},
      position: 'relative',
      isolation: 'isolate',
      overflow: 'hidden',
      backgroundColor: 'transparent',
      borderColor: 'var(--bf-br)',
      transitionProperty: 'border-color, color',
      transitionDuration: '200ms',
      transitionTimingFunction: 'cubic-bezier(0.4, 0, 0.2, 1)',
      /** Default fill — behind text (see `::after`). */
      '&::before': {
        content: '""',
        position: 'absolute',
        inset: 0,
        borderRadius: 'inherit',
        zIndex: -1,
        pointerEvents: 'none',
        backgroundImage: 'linear-gradient(to bottom, var(--bf-a), var(--bf-b))',
      },
      /** Hover / active fill — over `::before`; hover uses animated vertical gradient (`btn-flat-gradient-shift`). */
      '&::after': {
        content: '""',
        position: 'absolute',
        inset: 0,
        borderRadius: 'inherit',
        zIndex: -1,
        pointerEvents: 'none',
        backgroundImage: 'linear-gradient(to bottom, var(--bf-ha), var(--bf-hb), var(--bf-ha))',
        backgroundSize: '100% 300%',
        backgroundPosition: '50% 0%',
        backgroundRepeat: 'no-repeat',
        opacity: 0,
        animation: 'none',
        transition: 'opacity 220ms cubic-bezier(0.4, 0, 0.2, 1)',
      },
      '&:hover:not(:disabled)': { borderColor: 'var(--bf-hbr)' },
      /** Hover fill + ripple on the host `::after` only (same for `button`, `a` / `RouterLink`, etc.). */
      '&:hover:not(:disabled):not([aria-disabled="true"])::after': {
        opacity: 1,
        animation:
          'btn-flat-gradient-shift 2.2s ease-in-out infinite alternate, btn-flat-ripple 1.6s ease-out infinite',
      },
      '&:active:not(:disabled)': { borderColor: 'var(--bf-abr)' },
      '&:active:not(:disabled)::after': {
        backgroundImage: 'linear-gradient(to bottom, var(--bf-aa), var(--bf-ab))',
        backgroundSize: '100% 100%',
        backgroundPosition: '50% 50%',
        opacity: 1,
        animation: 'none',
      },
      '&:focus-visible:not(:disabled)': {
        '@apply outline-none ring-2 ring-offset-1 ring-offset-white': {},
        '--tw-ring-color': 'var(--bf-ring)',
      },
      '@media (prefers-reduced-motion: reduce)': {
        '&::after': { transition: 'opacity 0.01ms', animation: 'none' },
        '&:hover:not(:disabled)::after': {
          backgroundImage: 'linear-gradient(to bottom, var(--bf-ha), var(--bf-hb))',
          backgroundSize: '100% 100%',
          backgroundPosition: '50% 50%',
        },
      },
    },
  }

  const tint = (colorPath, hue) => ({
    '@apply btn-flat-surface': {},
    color: theme(`colors.${colorPath}`),
    ...flatStandardHue(theme, hue),
  })

  const tinted = {
    '.btn-get': tint('cyan.700', 'cyan'),
    '.btn-update': tint('teal.700', 'teal'),
    '.btn-delete, .btn-error': tint('red.700', 'red'),
    '.btn-create, .btn-success': tint('green.700', 'green'),
    '.btn-warning': tint('amber.700', 'amber'),
    '.btn-revert': tint('yellow.700', 'yellow'),
    '.btn-history': tint('purple.700', 'purple'),
    '.btn-link': tint('blue.700', 'blue'),
    '.btn-cancel, .btn-previous, .btn-next': tint('gray.700', 'gray'),
    '.btn-reply': tint('sky.700', 'sky'),
    /**
     * Flat `ui-btn--palette-*` — one tinted surface per hue (parallels `btn-aqua-*` in aqua theme).
     * `palette-white` is neutral chrome (`btn-empty`), not a saturated `tint()`.
     */
    '.btn-palette-amber': tint('amber.700', 'amber'),
    '.btn-palette-blue': tint('blue.700', 'blue'),
    '.btn-palette-cyan': tint('cyan.700', 'cyan'),
    '.btn-palette-emerald': tint('emerald.700', 'emerald'),
    '.btn-palette-fuchsia': tint('fuchsia.700', 'fuchsia'),
    '.btn-palette-indigo': tint('indigo.700', 'indigo'),
    '.btn-palette-lime': tint('lime.700', 'lime'),
    '.btn-palette-orange': tint('orange.700', 'orange'),
    '.btn-palette-pink': tint('pink.700', 'pink'),
    '.btn-palette-purple': tint('purple.700', 'purple'),
    '.btn-palette-red': tint('red.700', 'red'),
    '.btn-palette-rose': tint('rose.700', 'rose'),
    '.btn-palette-sky': tint('sky.700', 'sky'),
    '.btn-palette-slate': tint('slate.700', 'slate'),
    '.btn-palette-teal': tint('teal.700', 'teal'),
    '.btn-palette-violet': tint('violet.700', 'violet'),
    '.btn-palette-yellow': tint('yellow.700', 'yellow'),
    '.btn-palette-zinc': tint('zinc.700', 'zinc'),
  }

  const cta = (extra = '') => ({
    [`@apply btn-flat-surface ${extra}`.trim()]: {},
    color: theme('colors.white'),
    '--bf-a': theme('colors.blue.400'),
    '--bf-b': theme('colors.blue.500'),
    '--bf-br': theme('colors.blue.400'),
    '--bf-ha': theme('colors.blue.500'),
    '--bf-hb': theme('colors.blue.600'),
    '--bf-hbr': theme('colors.blue.600'),
    '--bf-aa': theme('colors.blue.600'),
    '--bf-ab': theme('colors.blue.700'),
    '--bf-abr': theme('colors.blue.700'),
    '--bf-ring': ring(theme('colors.blue.400')),
  })

  const custom = {
    '.btn-insert': cta('shadow-sm !border-0'),
    /**
     * Reactions: solid fill on the button (no ::before/::after layers).
     * Pseudo-gradients used the same border-radius as the outer box while sitting in the padding box,
     * which drew a sub-pixel hairline at rounded corners; padding-box background matches the stroke.
     */
    '.btn-reaction-active': {
      '@apply btn-flat-surface shadow-sm': {},
      /** Same hue as stroke — fill must match border (flat surface defaults to transparent). */
      '@apply !bg-[var(--bf-br)]': {},
      color: theme('colors.white'),
      '--bf-a': theme('colors.blue.500'),
      '--bf-b': theme('colors.blue.500'),
      '--bf-br': theme('colors.blue.500'),
      '--bf-ha': theme('colors.blue.600'),
      '--bf-hb': theme('colors.blue.700'),
      '--bf-hbr': theme('colors.blue.700'),
      '--bf-aa': theme('colors.blue.700'),
      '--bf-ab': theme('colors.blue.800'),
      '--bf-abr': theme('colors.blue.800'),
      '--bf-ring': ring(theme('colors.blue.400')),
      transitionProperty: 'border-color, color, background-color',
      '&::before': { display: 'none' },
      '&::after': { display: 'none' },
      '&:hover:not(:disabled)': {
        borderColor: 'var(--bf-hbr)',
        '@apply !bg-[var(--bf-hbr)]': {},
      },
      '&:active:not(:disabled)': {
        borderColor: 'var(--bf-abr)',
        '@apply !bg-[var(--bf-abr)]': {},
      },
    },
    '.btn-reaction': {
      '@apply btn-flat-surface shadow-sm': {},
      color: theme('colors.gray.700'),
      '--bf-a': theme('colors.gray.100'),
      '--bf-b': theme('colors.gray.50'),
      '--bf-br': theme('colors.gray.300'),
      '--bf-ha': theme('colors.gray.200'),
      '--bf-hb': theme('colors.gray.100'),
      '--bf-hbr': theme('colors.gray.400'),
      '--bf-aa': theme('colors.gray.300'),
      '--bf-ab': theme('colors.gray.200'),
      '--bf-abr': theme('colors.gray.500'),
      '--bf-ring': ring(theme('colors.gray.400')),
      backgroundColor: 'var(--bf-b)',
      transitionProperty: 'border-color, color, background-color',
      '&::before': { display: 'none' },
      '&::after': { display: 'none' },
      '&:hover:not(:disabled)': {
        borderColor: 'var(--bf-hbr)',
        backgroundColor: 'var(--bf-hb)',
      },
      '&:active:not(:disabled)': {
        borderColor: 'var(--bf-abr)',
        backgroundColor: 'var(--bf-ab)',
      },
    },
    /**
     * Primary FAB (flat theme): solid cornflower, no visible border, white glyph.
     * Paired with `.fab-elevation-shell` overrides in `buttonUiThemeLayer`.
     */
    '.btn-fab': {
      '@apply btn-base-core cursor-pointer rounded-full border-transparent text-white shadow-none':
        {},
      backgroundColor: theme('colors.cornflower.400'),
      transitionProperty: 'background-color, color, transform, border-color',
      '--tw-ring-color': ring(theme('colors.cornflower.500')),
      '&:not(:disabled):hover': {
        '@apply border-transparent ring-0 shadow-none': {},
        backgroundColor: theme('colors.cornflower.500'),
      },
      '&:not(:disabled):active': {
        '@apply border-transparent': {},
        backgroundColor: theme('colors.cornflower.600'),
      },
      '&:focus-visible:not(:disabled)': {
        '@apply outline-none ring-2 ring-offset-2 ring-offset-white': {},
        borderColor: 'transparent',
      },
      '@media (prefers-reduced-motion: reduce)': {
        '&:not(:disabled):hover': { transitionDuration: '0.01ms' },
      },
    },
    '.btn-market': {
      '@apply btn-flat-surface': {},
      color: theme('colors.rose.700'),
      '--bf-a': theme('colors.rose.50'),
      '--bf-b': theme('colors.white'),
      '--bf-br': theme('colors.rose.300'),
      '--bf-ha': theme('colors.rose.100'),
      '--bf-hb': theme('colors.rose.50'),
      '--bf-hbr': theme('colors.rose.500'),
      '--bf-aa': theme('colors.rose.200'),
      '--bf-ab': theme('colors.rose.100'),
      '--bf-abr': theme('colors.rose.600'),
      '--bf-ring': ring(theme('colors.rose.400')),
    },
    '.btn-action': {
      '@apply btn-flat-surface': {},
      color: theme('colors.pink.700'),
      '--bf-a': theme('colors.pink.50'),
      '--bf-b': theme('colors.white'),
      '--bf-br': theme('colors.pink.300'),
      '--bf-ha': theme('colors.pink.100'),
      '--bf-hb': theme('colors.pink.50'),
      '--bf-hbr': theme('colors.pink.500'),
      '--bf-aa': theme('colors.pink.200'),
      '--bf-ab': theme('colors.pink.100'),
      '--bf-abr': theme('colors.pink.600'),
      '--bf-ring': ring(theme('colors.pink.400')),
    },
    /** Flat theme: subscription-style toggle (pairs with `.btn-aqua-toggle` in aqua module). */
    '.btn-flat-toggle': {
      '&.active': {
        '@apply btn-error': {},
      },
      '&:not(.active)': {
        '@apply btn-empty': {},
      },
    },
    '.btn-empty': {
      '@apply btn-base-core text-gray-700 border-gray-400': {},
      /** Ripple keyframes read `var(--bf-ring)`; tinted surfaces set it via `flatStandardHue`. */
      '--bf-ring': emptyRippleBorder,
      position: 'relative',
      isolation: 'isolate',
      overflow: 'hidden',
      backgroundColor: 'transparent',
      boxShadow:
        'inset 0 1px 0 rgba(255, 255, 255, 0.9), inset 0 -1px 0 rgba(148, 163, 184, 0.18), 0 1px 2px rgba(15, 23, 42, 0.08)',
      transitionProperty: 'border-color, color',
      transitionDuration: '200ms',
      transitionTimingFunction: 'cubic-bezier(0.4, 0, 0.2, 1)',
      '&::before': {
        content: '""',
        position: 'absolute',
        inset: 0,
        borderRadius: 'inherit',
        zIndex: -1,
        pointerEvents: 'none',
        backgroundImage: `linear-gradient(to bottom, ${theme('colors.white')}, ${theme('colors.slate.50')})`,
      },
      '&::after': {
        content: '""',
        position: 'absolute',
        inset: 0,
        borderRadius: 'inherit',
        zIndex: -1,
        pointerEvents: 'none',
        backgroundImage: `linear-gradient(to bottom, ${theme('colors.white')}, ${theme('colors.slate.100')}, ${theme('colors.white')})`,
        backgroundSize: '100% 300%',
        backgroundPosition: '50% 0%',
        backgroundRepeat: 'no-repeat',
        opacity: 0,
        animation: 'none',
        transition: 'opacity 220ms cubic-bezier(0.4, 0, 0.2, 1)',
      },
      '&:not(:disabled):hover': {
        /** Match flat gray `--bf-hbr` / `--bf-abr` (see `flatStandardHue`) for border read with other `btn-flat-surface` neutrals. */
        '@apply ring-0 border-gray-600 text-gray-800': {},
        boxShadow:
          'inset 0 1px 0 rgba(255, 255, 255, 0.95), inset 0 -1px 0 rgba(148, 163, 184, 0.22), 0 2px 4px rgba(15, 23, 42, 0.1)',
      },
      '&:not(:disabled):not([aria-disabled="true"]):hover::after': {
        opacity: 1,
        animation:
          'btn-flat-gradient-shift 2.2s ease-in-out infinite alternate, btn-flat-ripple 1.6s ease-out infinite',
      },
      '&:not(:disabled):active': {
        '@apply ring-0 border-gray-700 text-gray-800': {},
        boxShadow:
          'inset 0 1px 2px rgba(15, 23, 42, 0.16), inset 0 -1px 0 rgba(255, 255, 255, 0.45), 0 0 0 rgba(15, 23, 42, 0)',
      },
      '&:not(:disabled):active::after': {
        backgroundImage: `linear-gradient(to bottom, ${theme('colors.slate.100')}, ${theme('colors.slate.200')})`,
        backgroundSize: '100% 100%',
        backgroundPosition: '50% 50%',
        opacity: 1,
        animation: 'none',
      },
      '&:focus-visible:not(:disabled)': {
        '@apply ring-2 ring-blue-400/45 ring-offset-1 ring-offset-white': {},
      },
      '@media (prefers-reduced-motion: reduce)': {
        '&::after': { transition: 'opacity 0.01ms', animation: 'none' },
        '&:not(:disabled):hover::after': {
          backgroundImage: `linear-gradient(to bottom, ${theme('colors.white')}, ${theme('colors.slate.100')})`,
          backgroundSize: '100% 100%',
          backgroundPosition: '50% 50%',
        },
      },
    },
    /** Flat counterpart to `btn-aqua-white` — light neutral pill, not a saturated hue. */
    '.btn-palette-white': {
      '@apply btn-empty': {},
    },
  }

  return {
    '@keyframes btn-flat-gradient-shift': {
      '0%': { backgroundPosition: '50% 0%' },
      '100%': { backgroundPosition: '50% 100%' },
    },
    /**
     * Inset pulse on the hover layer (`::after`) so the wave stays inside `overflow: hidden` pills
     * for both `button` and `a` / `RouterLink` (outset shadows are easy to miss or clip).
     */
    '@keyframes btn-flat-ripple': {
      '0%': { boxShadow: 'inset 0 0 0 0 transparent' },
      '45%': {
        boxShadow: 'inset 0 0 1.75rem 1.75rem color-mix(in srgb, var(--bf-ring) 38%, transparent)',
      },
      '100%': { boxShadow: 'inset 0 0 0 0 transparent' },
    },
    ...surface,
    ...tinted,
    ...custom,
  }
}
