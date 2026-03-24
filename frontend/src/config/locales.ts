export const supportedLocales = ['en', 'jbo', 'ru'] as const

export type SupportedLocale = (typeof supportedLocales)[number] // "en" | "jbo"

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
