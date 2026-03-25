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
        '.aqua-base': {
          '@apply flex items-center justify-center h-6 select-none whitespace-nowrap text-black select-none text-sm font-medium transition-all px-4 gap-2':
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
            '@apply hover:brightness-110 active:scale-[0.98]': {},
          },
          '& span': {
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
              '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
              '0 0.125em 0.125em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.5),' +
              'inset 0 0.25em 0.5em hsla(calc(var(--aqua-hue, 215) + 4), 100%, 9.6%, 0.8),' +
              'inset 0 0.375em 0.5em 0.25em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.75),' +
              '0 0 0.5em hsla(var(--aqua-hue, 215), 75.8%, 54.7%, 0.5)',
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
          '&.secondary': {
            background: 'linear-gradient(rgba(160, 160, 160, 0.625), rgba(255, 255, 255, 0.625))',
            boxShadow:
              '0 0.375em 0.5em rgba(0, 0, 0, 0.2), 0 0.125em 0.125em rgba(0, 0, 0, 0.3), inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4), inset 0 0.375em 0.5em 0.25em #BBBBBB',
            '&:focus, &:active': {
              boxShadow:
                '0 0.375em 0.5em rgba(0, 0, 0, 0.2), 0 0.125em 0.125em rgba(0, 0, 0, 0.3), inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4), inset 0 0.375em 0.5em 0.25em #BBBBBB, 0 0 0.5em rgba(0, 0, 0, 0.25)',
            },
          },
        },
      }),
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
          '.btn-aqua-purple': {
            '@apply aqua-base bg-purple-400': {},
            '--aqua-hue': '270',
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
          '.btn-aqua-slate': {
            '@apply aqua-base bg-zinc-300': {},
            '--aqua-hue': '212',
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
            '&.enabled': {
              '@apply text-white border-blue-500 bg-blue-500': {},
            },
            '&.enabled:hover:not(:disabled)': {
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
          '.btn-get': {
            '@apply btn-base text-blue-500 bg-slate-50 enabled:hover:bg-slate-200 border-blue-400':
              {},
          },
          '.btn-market': {
            '@apply btn-base text-rose-400 bg-white enabled:hover:bg-rose-200 border-rose-400': {},
          },
          '.btn-cancel': {
            '@apply btn-base text-gray-700 bg-gray-50 enabled:hover:bg-gray-200 border-gray-500':
              {},
          },
          '.btn-empty': {
            '@apply btn-base text-gray-600 bg-gray-50 enabled:hover:bg-gray-300 border-gray-300 enabled:hover:border-gray-400':
              {},
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
            // Tailwind's `max-sm` variant; `@screen` only accepts `theme.screens` keys (not max-*).
            '@media (max-width: 639px)': {
              '@apply w-auto': {},
            },
            '&:not(:disabled):active': {
              '@apply scale-[1.02]': {},
            },
            '@screen md': {
              '@apply rounded-none first:rounded-l-full last:rounded-r-full first:border-l last:border-r':
                {},
              '&:not(:last-child)': {
                '@apply border-r-0': {},
              },
            },
          },
          '.btn-group-forced': {
            '& .btn-group-item': {
              '@apply rounded-none first:rounded-l-full last:rounded-r-full': {},
              '&:not(:last-child)': {
                '@apply border-r-0': {},
              },
              '&.btn-group-item-selected': {
                '@apply bg-blue-100 border-blue-500 text-blue-700 ring-1 ring-blue-400 ring-inset':
                  {},
              },
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
          '.btn-aqua-group-item': {
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
        })
    },
  ],
}
