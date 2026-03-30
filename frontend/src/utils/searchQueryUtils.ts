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

/** Valsi or gloss keyword equals the full query (case-insensitive). Aligns with jbovlaste semantic exact_match / dictionary gloss rank. */
export function isSemanticPreciseMatch(
  definition: {
    valsiword?: string
    word?: string
    gloss_keywords?: Array<{ word?: string }>
  },
  searchQuery: string
): boolean {
  const raw = normalizeSearchQuery(searchQuery)
  if (typeof raw !== 'string') return false
  const q = raw.trim()
  if (!q) return false
  const qLower = q.toLowerCase()
  const valsi = (definition.valsiword ?? definition.word ?? '').trim()
  if (valsi.length > 0 && valsi.toLowerCase() === qLower) return true
  const gloss = definition.gloss_keywords
  if (!gloss?.length) return false
  return gloss.some((k) => (k.word ?? '').trim().toLowerCase() === qLower)
}
