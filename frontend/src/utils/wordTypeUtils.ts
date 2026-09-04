const classMap: Record<string, string> = {
  gismu: 'bg-green-100 text-green-800',
  cmavo: 'bg-blue-100 text-blue-800',
  lujvo: 'bg-yellow-100 text-yellow-800',
  'non-canonical lujvo': 'bg-amber-100 text-amber-900',
  "fu'ivla": 'bg-orange-100 text-orange-800',
  'experimental cmavo': 'bg-pink-100 text-pink-800',
  'experimental gismu': 'bg-pink-100 text-pink-800',
  'cmavo-compound': 'bg-indigo-100 text-indigo-800',
  definition: 'bg-blue-100 text-blue-800',
  etymology: 'bg-purple-100 text-purple-800',
  comment: 'bg-green-100 text-green-800',
  valsi: 'bg-yellow-100 text-yellow-800',
  message: 'bg-indigo-100 text-indigo-800',
  wiki: 'bg-teal-100 text-teal-800',
}

/** Tailwind classes for word type badges */
export const getTypeClass = (type: string): string => {
  return `select-none ${classMap[type] || 'select-none bg-gray-100 text-gray-800'}`
}

/** i18n key for a backend word-type descriptor (e.g. "fu'ivla" → "wordTypes.fuhivla") */
export const wordTypeLocaleKey = (typeName: string): string =>
  `wordTypes.${typeName.replace(/'/g, 'h').replace(/ /g, '-')}`

/** Localized label for a word type; falls back to the raw descriptor if untranslated */
export const getWordTypeLabel = (
  typeName: string | null | undefined,
  t: (key: string) => string
): string => {
  if (!typeName) return ''
  const key = wordTypeLocaleKey(typeName)
  const translated = t(key)
  if (translated === key) return typeName
  return translated
}
