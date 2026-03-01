/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
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
            top: '1px',
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
            filter: 'blur(1px)',
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
            '@apply px-4 py-1.5 text-sm h-8 text-gray-700 bg-white border border-gray-300 placeholder-blue-400 rounded-full transition-all focus:ring-1 focus:ring-blue-500 focus:border-blue-500 focus:outline-none focus:z-50 hover:z-50 whitespace-nowrap shadow-inner shadow-slate-200':
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
          '.btn-aqua-green': {
            '@apply aqua-base bg-green-300': {},
            '--aqua-hue': '142',
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
          '.btn-aqua-zinc': {
            '@apply aqua-base bg-zinc-300': {},
            '--aqua-hue': '240',
          },
          '.btn-aqua-slate': {
            '@apply aqua-base bg-zinc-300': {},
            '--aqua-hue': '212',
          },
          '.btn-aqua-white': {
            '@apply aqua-base bg-white': {},
            '--aqua-hue': '45',
            background: 'linear-gradient(rgba(160, 160, 160, 1), rgba(255, 255, 255, 0.625))',
            boxShadow:
              '0 0.375em 0.5em rgba(0, 0, 0, 0.2), 0 0.125em 0.125em rgba(0, 0, 0, 0.3), inset 0 0.25em 0.25em rgba(0, 0, 0, 0.4), inset 0 0.375em 0.5em 0.25em #BBBBBB',
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
            '@apply btn-base text-blue-700 bg-blue-50 enabled:hover:bg-blue-200 border-blue-600': {},
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
            border: '1px solid #333',
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
            '@screen md': {
              '@apply rounded-none first:rounded-l-full last:rounded-r-full first:border-l last:border-r':
                {},
              '&:not(:last-child)': {
                '@apply border-r-0': {},
              },
            },
          },
          '.btn-aqua-group-item': {
            borderRadius: 0,
            '&:first-child:not(:last-child)': {
              borderTopLeftRadius: '1000px',
              borderBottomLeftRadius: '1000px',
              borderTopRightRadius: 0,
              borderBottomRightRadius: 0,
              boxShadow:
              '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
              '0 0.125em 0.125em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.5),' +
              'inset 0 0.25em 0.5em hsla(calc(var(--aqua-hue, 215) + 4), 100%, 9.6%, 0.8),' +
              'inset 0 -0.25em 0.5em 0.25em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.75)',
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
              borderTopLeftRadius: 0,
              borderBottomLeftRadius: 0,
              borderTopRightRadius: '1000px',
              borderBottomRightRadius: '1000px',
              boxShadow:
              '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
              '0 0.125em 0.125em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.5),' +
              'inset 0 0.25em 0.5em hsla(calc(var(--aqua-hue, 215) + 4), 100%, 9.6%, 0.8),' +
              'inset 0 0.375em 0.5em -0.25em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.75)',
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
              boxShadow:
              '0 0.375em 0.5em rgba(0, 0, 0, 0.3),' +
              '0 0.125em 0.125em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.5),' +
              'inset 0 0.25em 0.5em hsla(calc(var(--aqua-hue, 215) + 4), 100%, 9.6%, 0.8),' +
              'inset 0 -0.25em 0.5em -0.25em hsla(var(--aqua-hue, 215), 100%, 36.7%, 0.75)',
              '&::before': {
                borderRadius: 0,
                left: 0,
                width: '100%',
                transform: 'none',
              },
              '&::after': {
                borderRadius: 0,
              },
            },
            '&:only-child': {
              borderRadius: '1000px',
            },
            '&:not(:disabled):active': {
              '@apply z-10 scale-[1.02]': {},
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
                'inset 0 0.25em 0.5em hsla(0, 0%, 9.6%, 0.8),' +
                'inset 0 0.375em 0.5em 0.25em hsla(0, 0%, 36.7%, 0.75)',
            },
          },
        })
    },
  ],
}
