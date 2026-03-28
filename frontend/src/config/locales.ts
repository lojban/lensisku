export const supportedLocales = ['en', 'jbo', 'ru', 'ja', 'zh'] as const

export type SupportedLocale = (typeof supportedLocales)[number]

/** Endonym (native name) for each UI locale, used in language selectors. */
export const localeNativeNames = {
  en: 'English',
  jbo: 'lojban',
  ru: 'Русский',
  ja: '日本語',
  zh: '中文',
} as const satisfies Record<SupportedLocale, string>

export function localeNativeName(code: string): string {
  return code in localeNativeNames
    ? localeNativeNames[code as SupportedLocale]
    : code
}

export const defaultLocale: SupportedLocale = 'en'

// Regex for matching locale prefixes in paths, e.g., /en/some/path
// Used to check if a path starts with a supported locale.
export const localePrefixRegex = new RegExp(`^/(${supportedLocales.join('|')})`)

// Regex for capturing the locale group from the path, e.g., for extracting "en" from "/en/some/path"
// Useful for extracting the locale string itself.
export const localeCaptureGroupRegex = new RegExp(`^/(${supportedLocales.join('|')})`)

// Default language tags for filters.
// These are assumed to be the same as i18n locale codes for now.
export const defaultFilterLanguageTags: SupportedLocale[] = ['en', 'jbo']
