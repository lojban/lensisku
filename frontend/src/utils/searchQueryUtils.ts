/**
 * Normalize search query text by replacing curly/smart apostrophes (e.g. from iPad)
 * with ASCII single quote (U+0027) so URLs and API params work consistently.
 */
export function normalizeSearchQuery(str: string): string
export function normalizeSearchQuery(str: unknown): unknown
export function normalizeSearchQuery(str: unknown): unknown {
  if (typeof str !== 'string') return str
  return str
    .replace(/\u2019/g, "'")
    .replace(/\u2018/g, "'")
    .replace(/\u02BC/g, "'")
}
