/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{vue,js,ts,jsx,tsx}',
    './packages/**/*.{vue,js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {},
  },
  plugins: [
    function ({ addComponents, addBase }) {
      addBase({
        /** Icon + label: use this flex `gap-*` only—do not put `mr-*` / `ml-*` on icons or labels (breaks when labels are hidden). */
        '.btn-base': {
          '@apply gap-2 px-4 py-1.5 text-xs font-medium flex items-center justify-center h-6 border rounded-full transition-all shadow-sm shadow-slate-200 disabled:opacity-40 select-none disabled:cursor-not-allowed whitespace-nowrap focus:outline-none':
            {},
          '&:not(:disabled)': {
            '--tw-ring-color': 'var(--btn-color, currentColor)',
            '@apply active:scale-[0.98]': {},
          },
          '&:not(:disabled):hover': {
            background:
              'linear-gradient(to bottom, rgba(255,255,255,0.8) 0%, rgba(255,255,255,0.3) 50%, rgba(255,255,255,0.1) 100%)',
            '@apply ring-0 shadow-none': {},
            // background: "radial-gradient(circle at center, rgba(255,255,255,0.8) 0%, rgba(255,255,255,0) 70%)",
          },
        },
        /** Same as `btn-base`: icon + label spacing via `gap-*` on the control, not margin on children. */
        '.aqua-base': {
          '@apply flex items-center justify-center h-6 select-none whitespace-nowrap text-black select-none text-sm font-medium transition-all px-3 gap-2':
            {},
          textOverflow: 'ellipsis',
          gridRow: '1',
          fontFamily: '"open sans", system-ui, tahoma',
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
            '@apply hover:brightness-105 hover:saturate-150 active:scale-[0.98]': {},
          },
          /** Do not target `.sr-only` — that would override `position:absolute` and break flex centering on icon-only controls. */
          '& span:not(.sr-only)': {
            position: 'relative',
            top: '0.75px',
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
      });
      /**
       * Shared aqua segment geometry (radii + glossy ::before/::after), without extra segment box-shadows —
       * elevation stays on `aqua-base` / semantic primitives so wrappers and joins do not stack duplicate shadows.
       * Applied with higher specificity in buttonUiThemeLayer for `.btn-group-forced` (see addPair order).
       */
      const aquaUiBtnGroupItemGeometry = {
        borderRadius: 0,
        '&::after': {
          background: 'none',
        },
        '&:first-child:not(:last-child)': {
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
        '&:last-child:not(:first-child)': {
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
        '&:not(:first-child):not(:last-child)': {
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
        '&:only-child': {
          borderRadius: '1000px',
        },
      }
      addComponents({
        blockquote: {
          '@apply pl-4 border-l-4 border-gray-300 my-4 text-sm italic': {},
          '& p': {
            '@apply text-gray-600': {},
          },
        },
        '.navbar-item': {
          '@apply gap-2 py-1.5 text-base flex items-center h-9 min-w-12 flex items-center justify-center px-2 md:px-4 py-1.5 font-medium text-gray-600 rounded-full transition-colors select-none whitespace-nowrap':
            {},
          '&:not(.primary):not(.nav-link-active)': {
            '@apply hover:bg-gray-200 text-[#007bff]': {},
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
        '.read-box': {
          '@apply shadow-inner shadow-slate-200': {},
        },
        /** Full-viewport loading veil: blur content behind spinner (used by LoadingSpinner page variant). */
        '.page-loading-overlay': {
          '@apply fixed inset-0 z-50 flex min-h-0 items-center justify-center bg-white/50 backdrop-blur-sm':
            {},
        },
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
        /** Closed trigger for toolbar dropdowns (search mode, language multiselect): matches input-field visual language at h-10. */
        '.dropdown-trigger': {
          '@apply w-full h-10 min-h-[2.5rem] shrink-0 flex items-center justify-between gap-2 px-3 text-left text-sm font-normal text-gray-700 bg-white border border-gray-300 rounded-full transition-all focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 focus:z-50 shadow-inner shadow-slate-200':
            {},
          '&:hover:not(:disabled)': {
            '@apply border-blue-400': {},
          },
        },
        /** Leading segment of the home search bar (flush join with the query input). */
        '.dropdown-trigger--search-bar-leading': {
          '@apply rounded-l-full rounded-r-none': {},
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
        '.btn-aqua': {
          '@apply aqua-base': {},
        },
        /** Blue glossy “primary” fill (`button.primary`); background only — shadows follow `--aqua-hue`. */
        '.btn-aqua-primary': {
          '@apply aqua-base': {},
          background:
            'linear-gradient(rgba(0, 65, 184, 0.625), rgba(45, 115, 199, 0.625), rgba(33, 160, 196, 0.625))',
          '--aqua-hue': '217',
        },
        /** Gray glossy “secondary” look (was `.aqua-base.secondary`). */
        '.btn-aqua-secondary': {
          '@apply aqua-base': {},
          background: 'linear-gradient(rgba(160, 160, 160, 0.625), rgba(255, 255, 255, 0.625))',
          boxShadow:
            '0 0.375em 0.5em rgba(0, 0, 0, 0.2), 0 0.125em 0.125em rgba(0, 0, 0, 0.3), inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4), inset 0 0.375em 0.5em 0.25em #BBBBBB',
          '&:focus, &:active': {
            boxShadow:
              '0 0.375em 0.5em rgba(0, 0, 0, 0.2), 0 0.125em 0.125em rgba(0, 0, 0, 0.3), inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4), inset 0 0.375em 0.5em 0.25em #BBBBBB, 0 0 0.5em rgba(0, 0, 0, 0.25)',
          },
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
        /** Aqua “get / open” — lighter sky-blue than btn-aqua-sky; used by ui-btn--get only. */
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
          boxShadow: '0 0.35em 0.45em rgba(2, 10, 26, 0.28),' +
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
        /** Distinct from btn-aqua-white / zinc: cooler, slightly deeper fill so “slate” ≠ generic gray. */
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
          '@apply aqua-base bg-white': {},
          background: 'linear-gradient(rgba(180, 180, 180, 1), rgba(255, 255, 255, 0.625))',
          boxShadow:
            '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
            '0 0.125em 0.125em hsla(0, 0%, 36.7%, 0.5),' +
            'inset 0 0.15em 0.35em rgba(255, 255, 255, 0.7),' +
            'inset 0 -0.05em 0.2em rgba(0, 0, 0, 0.06)',
          '&:not(:disabled)': {
            '@apply hover:brightness-110 active:scale-[0.98]': {},
          },
          '&:focus, &:active': {
            boxShadow:
              '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
              '0 0.125em 0.125em hsla(0, 0%, 36.7%, 0.5),' +
              'inset 0 0.15em 0.35em rgba(255, 255, 255, 0.7),' +
              'inset 0 -0.05em 0.2em rgba(0, 0, 0, 0.06),' +
              '0 0 0.5em hsla(0, 0%, 54.7%, 0.5)',
            '&:disabled': {
              boxShadow:
                '0 0.375em 0.5em rgba(0, 0, 0, 0.2), 0 0.125em 0.125em rgba(0, 0, 0, 0.3), inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4), inset 0 0.375em 0.5em 0.25em #BBBBBB',
            },
          },
        },
        '.btn-insert': {
          '@apply btn-base text-white bg-gradient-to-b from-blue-400 to-blue-500 border-blue-500 text-white enabled:hover:text-blue-500 enabled:hover:bg-gradient-to-b enabled:hover:from-white enabled:hover:to-white':
            {},
        },
        '.btn-reaction': {
          '@apply btn-base text-gray-600 bg-gray-50 border-gray-300 shadow-sm': {},
          '&:hover:not(:disabled)': {
            background: 'rgb(128 128 128)',
            '@apply text-white': {},
          },
        },
        '.btn-reaction-active': {
          '@apply btn-base text-white border-blue-500 bg-blue-500': {},
          '&:hover:not(:disabled)': {
            background: 'rgb(37, 99, 235)',
            '@apply border-blue-600': {},
          },
        },
        '.btn-create': {
          '@apply btn-base text-green-700 bg-gradient-to-b from-green-100 to-green-50 enabled:hover:from-green-200 enabled:hover:to-green-100 border-green-500 enabled:hover:border-green-700':
            {},
        },
        '.btn-update': {
          '@apply btn-base text-teal-700 bg-teal-50 enabled:hover:bg-green-200 border-teal-600':
            {},
        },
        '.btn-delete': {
          '@apply btn-base text-red-700 bg-red-50 enabled:hover:bg-red-200 border-red-600': {},
        },
        /** Flat “get / slate-neutral”: blue surface so it does not read like btn-empty (white/gray). */
        '.btn-get': {
          '@apply btn-base text-blue-700 bg-blue-50 enabled:hover:bg-blue-100 border-blue-500':
            {},
        },
        '.btn-market': {
          '@apply btn-base text-rose-400 bg-white enabled:hover:bg-rose-200 border-rose-400': {},
        },
        '.btn-cancel': {
          '@apply btn-base text-gray-700 bg-gray-50 enabled:hover:bg-gray-200 border-gray-500':
            {},
        },
        // Neutral “dismiss / clear” (brandbook §1.B): subtle raised/embossed default with pressed active state.
        '.btn-empty': {
          '@apply btn-base text-gray-700 bg-gradient-to-b from-white to-slate-50 border-gray-300': {},
          boxShadow:
            'inset 0 1px 0 rgba(255, 255, 255, 0.9), inset 0 -1px 0 rgba(148, 163, 184, 0.18), 0 1px 2px rgba(15, 23, 42, 0.08)',
          '&:not(:disabled):hover': {
            '@apply bg-gradient-to-b from-white to-slate-100 border-gray-400 text-gray-800': {},
            boxShadow:
              'inset 0 1px 0 rgba(255, 255, 255, 0.95), inset 0 -1px 0 rgba(148, 163, 184, 0.22), 0 2px 4px rgba(15, 23, 42, 0.1)',
          },
          '&:not(:disabled):active': {
            '@apply bg-gradient-to-b from-slate-100 to-slate-200 border-gray-400 text-gray-800': {},
            boxShadow:
              'inset 0 1px 2px rgba(15, 23, 42, 0.16), inset 0 -1px 0 rgba(255, 255, 255, 0.45), 0 0 0 rgba(15, 23, 42, 0)',
          },
          '&:focus-visible:not(:disabled)': {
            '@apply ring-2 ring-blue-400/45 ring-offset-1 ring-offset-white': {},
          },
        },
        '.btn-error': {
          '@apply btn-base text-red-700 bg-red-50 enabled:hover:bg-red-200 border-red-600': {},
        },
        '.btn-warning': {
          '@apply btn-base text-amber-700 bg-amber-50 enabled:hover:bg-amber-200 border-amber-600':
            {},
        },
        '.btn-success': {
          '@apply btn-base text-green-700 bg-green-50 enabled:hover:bg-green-200 border-green-600':
            {},
        },
        '.btn-revert': {
          '@apply btn-base text-yellow-700 bg-yellow-50 enabled:hover:bg-yellow-200 border-yellow-600':
            {},
        },
        '.btn-history': {
          '@apply btn-base text-purple-700 bg-purple-50 enabled:hover:bg-purple-200 border-purple-600':
            {},
        },
        '.btn-link': {
          '@apply btn-base text-blue-700 bg-blue-50 enabled:hover:bg-blue-200 border-blue-600':
            {},
        },
        '.btn-previous, .btn-next': {
          '@apply btn-base text-gray-700 bg-gray-50 enabled:hover:bg-gray-200 border-gray-500':
            {},
        },
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
        '.btn-reply': {
          '@apply btn-base text-sky-700 bg-sky-50 hover:bg-sky-200 border-sky-600': {},
        },
        '.btn-action': {
          '@apply btn-base text-pink-600 bg-white border-pink-600 enabled:hover:bg-pink-50 enabled:hover:text-pink-700':
            {},
        },
        '.btn-group-item': {
          '@apply border rounded-full': {},
          '&:not(:disabled):active': {
            '@apply scale-[1.02]': {},
          },
          /** Segment bar from 512px up (aligned with `.btn-group-forced`). */
          '@media (min-width: 512px)': {
            '@apply rounded-none first:rounded-l-full last:rounded-r-full first:border-l last:border-r':
              {},
            '&:not(:last-child)': {
              '@apply border-r-0': {},
            },
          },
        },
        '.btn-group-forced': {
          '@apply gap-0': {},
          '& .btn-group-item, & .ui-btn--group-item': {
            '@apply rounded-none first:rounded-l-full last:rounded-r-full': {},
            '&:not(:last-child)': {
              '@apply border-r-0': {},
            },
          },
          '&.btn-group-item-selected': {
            '@apply bg-blue-100 border-blue-500 text-blue-700 ring-1 ring-blue-400 ring-inset':
              {},
          },
        },
        // Collection list sort: selected = option color + glow; others = btn-aqua-white
        '.btn-aqua-sort-idle': {
          '@apply z-10 transition-[filter] duration-200 ease-out': {},
        },
        '.btn-aqua-sort-active': {
          '@apply z-20 relative transition-[filter] duration-200 ease-out': {},
          filter:
            'drop-shadow(0 0 0.3rem hsla(var(--aqua-hue, 215), 82%, 58%, 0.75)) drop-shadow(0 0 0.9rem hsla(var(--aqua-hue, 215), 78%, 55%, 0.5))',
          '&:hover:not(:disabled)': {
            filter:
              'brightness(1.1) drop-shadow(0 0 0.3rem hsla(var(--aqua-hue, 215), 82%, 58%, 0.8)) drop-shadow(0 0 0.9rem hsla(var(--aqua-hue, 215), 78%, 55%, 0.55))',
          },
        },
        /** Segment joins from 512px up; below that, `ui-btn--*` primitives stay full pills (avoids broken wraps). */
        '.btn-aqua-group-item': {
          '@media (min-width: 512px)': aquaUiBtnGroupItemGeometry,
        },
        '.btn-aqua-toggle': {
          '&.active': {
            '@apply aqua-base bg-red-400': {},
            '--aqua-hue': '0',
          },
          '&:not(.active)': {
            '@apply btn-aqua-white': {},
            boxShadow:
              '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
              '0 0.125em 0.125em hsla(0, 0%, 36.7%, 0.5),' +
              'inset 0 0.15em 0.35em rgba(255, 255, 255, 0.7),' +
              'inset 0 -0.05em 0.2em rgba(0, 0, 0, 0.06)',
          },
        },
        /** Flat theme: collection list sort chips (replaces btn-aqua-sort-* when data-button-theme=flat). */
        '.btn-flat-sort-idle': {
          '@apply z-10 transition-[filter,box-shadow] duration-200 ease-out': {},
        },
        '.btn-flat-sort-active': {
          '@apply z-20 relative ring-2 ring-blue-400 ring-offset-1 ring-offset-white': {},
        },
        /** Flat theme: subscription toggle (replaces btn-aqua-toggle). */
        '.btn-flat-toggle': {
          '&.active': {
            '@apply btn-error': {},
          },
          '&:not(.active)': {
            '@apply btn-empty': {},
          },
        },
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
        '.card-title': {
          '@apply text-lg font-semibold text-gray-800 block truncate transition-colors': {},
          '&:hover': { '@apply text-blue-600': {} },
          '&:focus': { '@apply text-blue-600 outline-none': {} },
        },
        '.card-description': {
          '@apply text-gray-600 text-sm line-clamp-2 leading-snug': {},
        },
        '.card-footer-inner': {
          '@apply flex flex-wrap items-center justify-between gap-2': {},
        },
        '.card-badges': {
          '@apply flex flex-wrap gap-2': {},
        },
        '.badge': {
          '@apply inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium': {},
        },
        '.badge-public': {
          '@apply bg-green-100 text-green-700': {},
        },
        '.badge-private': {
          '@apply bg-gray-100 text-gray-600': {},
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
        '.card-nav-buttons': {
          '@apply flex flex-nowrap items-center justify-center gap-0 rounded-xl overflow-hidden border border-gray-200 bg-gray-50/50':
            {},
        },
        '.card-nav-btn': {
          '@apply flex items-center gap-2 px-4 py-2.5 text-sm font-medium text-gray-600 flex-1 justify-center border-r border-gray-200 last:border-r-0 transition-colors':
            {},
          '&:hover': { '@apply bg-gray-100 text-gray-800': {} },
        },
        '.card-study-area': {
          '@apply flex items-center justify-center min-h-[5.5rem] py-6 sm:py-8 flex-shrink-0': {},
        },
        /** Next to the main FAB: overrides aqua-base / btn-base compact h-6 + text-sm for touch-friendly targets. */
        '.fab-menu-action': {
          '@apply !min-h-[2rem] !h-auto !py-2 !px-6 !text-lg !gap-3': {},
        },
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
          aqua: 'html[data-button-theme="aqua"] .CLASS, html:not([data-button-theme]) .CLASS',
          flat: 'html[data-button-theme="flat"] .CLASS',
        }
        const selectorFor = (theme, cls) => themeSelectors[theme].replace(/CLASS/g, cls)
        const addThemeMap = (themeMap) => {
          for (const [cls, classesByTheme] of Object.entries(themeMap)) {
            for (const [theme, primitives] of Object.entries(classesByTheme)) {
              rules[selectorFor(theme, cls)] = { [`@apply ${primitives}`]: {} }
            }
          }
        }
        const buttonThemeClassMap = {
          'ui-btn--insert': { aqua: 'btn-aqua-blue', flat: 'btn-insert' },
          'ui-btn--create': { aqua: 'btn-aqua-emerald', flat: 'btn-create' },
          'ui-btn--update': { aqua: 'btn-aqua-teal', flat: 'btn-update' },
          'ui-btn--delete': { aqua: 'btn-aqua-red', flat: 'btn-delete' },
          'ui-btn--cancel': { aqua: 'btn-aqua-zinc', flat: 'btn-cancel' },
          'ui-btn--empty': { aqua: 'btn-aqua-white', flat: 'btn-empty' },
          'ui-btn--reaction': { aqua: 'btn-aqua-cyan', flat: 'btn-reaction' },
          'ui-btn--reaction-active': { aqua: 'btn-aqua-blue', flat: 'btn-reaction-active' },
          'ui-btn--get': { aqua: 'btn-aqua-get', flat: 'btn-get' },
          'ui-btn--market': { aqua: 'btn-aqua-rose', flat: 'btn-market' },
          'ui-btn--error': { aqua: 'btn-aqua-red', flat: 'btn-error' },
          'ui-btn--warning': { aqua: 'btn-aqua-orange', flat: 'btn-warning' },
          'ui-btn--success': { aqua: 'btn-aqua-emerald', flat: 'btn-success' },
          'ui-btn--revert': { aqua: 'btn-aqua-yellow', flat: 'btn-revert' },
          'ui-btn--history': { aqua: 'btn-aqua-purple', flat: 'btn-history' },
          'ui-btn--link': { aqua: 'btn-aqua-sky', flat: 'btn-link' },
          'ui-btn--reply': { aqua: 'btn-aqua-sky', flat: 'btn-reply' },
          'ui-btn--action': { aqua: 'btn-aqua-pink', flat: 'btn-action' },
          'ui-btn--previous': { aqua: 'btn-aqua-zinc', flat: 'btn-previous' },
          'ui-btn--next': { aqua: 'btn-aqua-zinc', flat: 'btn-next' },
          'ui-btn--group-item': { aqua: 'btn-aqua-group-item', flat: 'btn-group-item' },
          'ui-btn--aqua-default': { aqua: 'btn-aqua-simple', flat: 'btn-get' },
          'ui-btn--accent-purple': { aqua: 'btn-aqua-purple', flat: 'btn-link' },
          'ui-btn--danger-rose': { aqua: 'btn-aqua-rose', flat: 'btn-delete' },
          'ui-btn--warning-orange': { aqua: 'btn-aqua-orange', flat: 'btn-warning' },
          'ui-btn--warning-yellow': { aqua: 'btn-aqua-yellow', flat: 'btn-warning' },
          'ui-btn--amber': { aqua: 'btn-aqua-amber', flat: 'btn-warning' },
          'ui-btn--auth-login': { aqua: 'btn-aqua-orange', flat: 'btn-warning' },
          'ui-btn--auth-signup': { aqua: 'btn-aqua-teal', flat: 'btn-create' },
          'ui-btn--continue': { aqua: 'btn-aqua', flat: 'btn-get' },
          'ui-btn--study-wrong': { aqua: 'btn-aqua-rose', flat: 'btn-error' },
          'ui-btn--study-correct': { aqua: 'btn-aqua-teal', flat: 'btn-success' },
          'ui-btn--sort-sky': { aqua: 'btn-aqua-sky', flat: 'btn-get' },
          'ui-btn--sort-blue': { aqua: 'btn-aqua-blue', flat: 'btn-insert' },
          'ui-btn--sort-amber': { aqua: 'btn-aqua-amber', flat: 'btn-warning' },
          'ui-btn--sort-emerald': { aqua: 'btn-aqua-emerald', flat: 'btn-create' },
          'ui-btn--sort-active': { aqua: 'btn-aqua-sort-active', flat: 'btn-flat-sort-active' },
          'ui-btn--sort-idle': { aqua: 'btn-aqua-sort-idle', flat: 'btn-flat-sort-idle' },
          'ui-btn--neutral': { aqua: 'btn-aqua-white', flat: 'btn-get' },
          'ui-btn--neutral-muted': { aqua: 'btn-aqua-zinc', flat: 'btn-cancel' },
          'ui-btn--neutral-slate': { aqua: 'btn-aqua-slate', flat: 'btn-get' },
          'ui-btn--primary': { aqua: 'btn-aqua-emerald', flat: 'btn-create' },
          'ui-btn--palette-white': { aqua: 'btn-aqua-white', flat: 'btn-empty' },
          'ui-btn--palette-red': { aqua: 'btn-aqua-red', flat: 'btn-delete' },
          'ui-btn--palette-orange': { aqua: 'btn-aqua-orange', flat: 'btn-warning' },
          'ui-btn--palette-amber': { aqua: 'btn-aqua-amber', flat: 'btn-warning' },
          'ui-btn--palette-yellow': { aqua: 'btn-aqua-yellow', flat: 'btn-warning' },
          'ui-btn--palette-lime': { aqua: 'btn-aqua-lime', flat: 'btn-success' },
          'ui-btn--palette-teal': { aqua: 'btn-aqua-teal', flat: 'btn-create' },
          'ui-btn--palette-emerald': { aqua: 'btn-aqua-emerald', flat: 'btn-create' },
          'ui-btn--palette-cyan': { aqua: 'btn-aqua-cyan', flat: 'btn-insert' },
          'ui-btn--palette-sky': { aqua: 'btn-aqua-sky', flat: 'btn-get' },
          'ui-btn--palette-blue': { aqua: 'btn-aqua-blue', flat: 'btn-insert' },
          'ui-btn--palette-indigo': { aqua: 'btn-aqua-indigo', flat: 'btn-link' },
          'ui-btn--palette-violet': { aqua: 'btn-aqua-violet', flat: 'btn-link' },
          'ui-btn--palette-purple': { aqua: 'btn-aqua-purple', flat: 'btn-history' },
          'ui-btn--palette-fuchsia': { aqua: 'btn-aqua-fuchsia', flat: 'btn-market' },
          'ui-btn--palette-pink': { aqua: 'btn-aqua-pink', flat: 'btn-action' },
          'ui-btn--palette-rose': { aqua: 'btn-aqua-rose', flat: 'btn-market' },
          'ui-btn--palette-slate': { aqua: 'btn-aqua-slate', flat: 'btn-get' },
          'ui-btn--palette-zinc': { aqua: 'btn-aqua-zinc', flat: 'btn-cancel' },
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
        /** Outer elevation for FAB + overflow menu (does not replace aqua glossy box-shadow). Add per future [data-button-theme]. */
        const fabOuterShadowAqua =
          'drop-shadow(0 14px 32px hsla(215, 42%, 12%, 0.44)) drop-shadow(0 4px 12px hsla(215, 38%, 22%, 0.28))'
        const fabOuterShadowFlat =
          'drop-shadow(0 12px 28px rgba(15, 23, 42, 0.22)) drop-shadow(0 4px 14px rgba(15, 23, 42, 0.14))'
        const fabMenuOuterShadowAqua =
          'drop-shadow(0 10px 24px hsla(215, 38%, 14%, 0.38)) drop-shadow(0 3px 10px hsla(215, 34%, 20%, 0.24))'
        const fabMenuOuterShadowFlat =
          'drop-shadow(0 10px 22px rgba(15, 23, 42, 0.17)) drop-shadow(0 3px 10px rgba(15, 23, 42, 0.11))'
        const fabShell = {
          minHeight: '52px',
          width: '52px',
          height: '52px',
          borderRadius: '1000px',
          flexShrink: '0',
          boxSizing: 'border-box',
          padding: '0.5rem',
        }
        const aquaRules = {}
        const flatRules = {}
        aquaRules[selectorFor('aqua', 'ui-btn--fab')] = {
          '@apply btn-aqua-rose': {},
          ...fabShell,
          filter: fabOuterShadowAqua,
        }
        flatRules[selectorFor('flat', 'ui-btn--fab')] = {
          '@apply btn-market': {},
          ...fabShell,
          filter: fabOuterShadowFlat,
        }
        aquaRules[selectorFor('aqua', 'fab-menu-action')] = {
          filter: fabMenuOuterShadowAqua,
        }
        flatRules[selectorFor('flat', 'fab-menu-action')] = {
          filter: fabMenuOuterShadowFlat,
        }
        const toggleOffShadow =
          '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
          '0 0.125em 0.125em hsla(0, 0%, 36.7%, 0.5),' +
          'inset 0 0.15em 0.35em rgba(255, 255, 255, 0.7),' +
          'inset 0 -0.05em 0.2em rgba(0, 0, 0, 0.06)'
        aquaRules[selectorFor('aqua', 'ui-btn--toggle.active')] = {
          '@apply aqua-base bg-red-400': {},
          '--aqua-hue': '0',
        }
        aquaRules[selectorFor('aqua', 'ui-btn--toggle:not(.active)')] = {
          '@apply btn-aqua-white': {},
          boxShadow: toggleOffShadow,
        }
        flatRules[selectorFor('flat', 'ui-btn--toggle.active')] = {
          '@apply btn-error': {},
        }
        flatRules[selectorFor('flat', 'ui-btn--toggle:not(.active)')] = {
          '@apply btn-empty': {},
        }
        /** Beats semantic ui-btn--* primitives (loaded above) so inner segments stay square in forced groups. */
        aquaRules[selectorFor('aqua', 'btn-group-forced .ui-btn--group-item')] = aquaUiBtnGroupItemGeometry
        flatRules[selectorFor('flat', 'btn-group-forced .ui-btn--group-item')] = {
          '@apply rounded-none first:rounded-l-full last:rounded-r-full': {},
          '&:not(:last-child)': {
            '@apply border-r-0': {},
          },
          '&.btn-group-item-selected': {
            '@apply bg-blue-100 border-blue-500 text-blue-700 ring-1 ring-blue-400 ring-inset':
              {},
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
