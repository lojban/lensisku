import type { LocationQuery } from 'vue-router'

/** First string from a vue-router query value (may be `string[]`). */
export function queryStr(v: string | string[] | null | undefined): string {
  if (v == null) return ''
  return Array.isArray(v) ? (v[0] ?? '') : v
}

/** First segment from a route param (may be `string[]`). */
export function paramStr(v: string | string[] | null | undefined): string {
  return queryStr(v)
}

/** CombinedFilters fields that are mirrored in the URL. */
export type CombinedFiltersUrlState = {
  selectedLanguages: number[]
  selectedCollections: number[]
  usernames: string[]
  excludeUsernames: string[]
  selmaho: string
  word_type: number | null
  source_langid: number
  searchInPhrases: boolean
  isExpanded: boolean
}

const HOME_SEARCH_MODES = new Set(['dictionary', 'semantic', 'comments', 'messages', 'muplis'])

/** Query keys restored when navigating home via the logo (filters + search, not page chrome). */
export const HOME_PRESERVED_QUERY_KEYS = [
  'q',
  'mode',
  'langs',
  'collections',
  'selmaho',
  'username',
  'exclude_usernames',
  'word_type',
  'source_langid',
  'searchInPhrases',
  'isExpanded',
  'group_by_thread',
  'wave_source',
  'definition_id',
] as const

const LAST_HOME_QUERY_KEY = 'lensisku:lastHomeQuery'

function splitCsv(value: string): string[] {
  return value
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean)
}

function csvOrUndefined(values: Array<string | number> | undefined): string | undefined {
  if (!values?.length) return undefined
  return values.join(',')
}

/** Drop empty/undefined keys so vue-router actually removes them from the URL. */
export function compactQuery(query: Record<string, unknown>): Record<string, string> {
  const out: Record<string, string> = {}
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined || value === null || value === '') continue
    if (Array.isArray(value)) {
      const first = value.find((v) => v != null && v !== '')
      if (first == null || first === '') continue
      out[key] = String(first)
      continue
    }
    const s = String(value)
    if (s === '' || s === 'undefined') continue
    out[key] = s
  }
  return out
}

export function combinedFiltersToQuery(
  filters: CombinedFiltersUrlState
): Record<string, string | undefined> {
  return {
    langs: csvOrUndefined(filters.selectedLanguages),
    collections: csvOrUndefined(filters.selectedCollections),
    username: csvOrUndefined(filters.usernames),
    exclude_usernames: csvOrUndefined(filters.excludeUsernames),
    selmaho: filters.selmaho?.trim() || undefined,
    word_type:
      filters.word_type != null && Number.isFinite(filters.word_type)
        ? String(filters.word_type)
        : undefined,
    source_langid:
      filters.source_langid && filters.source_langid !== 1
        ? String(filters.source_langid)
        : undefined,
    searchInPhrases: filters.searchInPhrases === false ? 'false' : undefined,
    isExpanded: filters.isExpanded ? 'true' : undefined,
  }
}

export function combinedFiltersFromQuery(query: LocationQuery): CombinedFiltersUrlState {
  const langs = queryStr(query.langs)
  const collections = queryStr(query.collections)
  const username = queryStr(query.username)
  const exclude = queryStr(query.exclude_usernames)
  const wordType = queryStr(query.word_type)
  const source = queryStr(query.source_langid)
  return {
    selectedLanguages: langs
      ? langs
          .split(',')
          .map(Number)
          .filter((n) => Number.isFinite(n))
      : [],
    selectedCollections: collections
      ? collections
          .split(',')
          .map(Number)
          .filter((n) => Number.isFinite(n) && n > 0)
      : [],
    usernames: username ? splitCsv(username) : [],
    excludeUsernames: exclude ? splitCsv(exclude) : [],
    selmaho: queryStr(query.selmaho),
    word_type: wordType ? Number(wordType) : null,
    source_langid: source ? parseInt(source, 10) || 1 : 1,
    searchInPhrases: query.searchInPhrases !== 'false',
    isExpanded: query.isExpanded === 'true',
  }
}

/** Filters that should list dictionary results even when the search box is empty. */
export function hasActiveSearchFilters(filters: CombinedFiltersUrlState): boolean {
  return Boolean(
    filters.selmaho?.trim() ||
      filters.usernames?.length ||
      filters.excludeUsernames?.length ||
      filters.word_type ||
      filters.selectedCollections?.length ||
      (filters.source_langid && filters.source_langid !== 1) ||
      filters.searchInPhrases === false
  )
}

export function pickHomeQuery(query: LocationQuery): Record<string, string> {
  const out: Record<string, string> = {}
  for (const key of HOME_PRESERVED_QUERY_KEYS) {
    const s = queryStr(query[key])
    if (!s) continue
    if (key === 'mode' && !HOME_SEARCH_MODES.has(s)) continue
    out[key] = s
  }
  return out
}

function parseStoredQueryRecord(raw: string | null): Record<string, string> {
  if (!raw) return {}
  try {
    const parsed = JSON.parse(raw) as unknown
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return {}
    const out: Record<string, string> = {}
    for (const [key, value] of Object.entries(parsed as Record<string, unknown>)) {
      if (typeof value === 'string' && value) out[key] = value
    }
    return out
  } catch {
    return {}
  }
}

export function saveLastHomeQuery(query: LocationQuery): void {
  if (typeof window === 'undefined') return
  try {
    const payload = JSON.stringify(pickHomeQuery(query))
    localStorage.setItem(LAST_HOME_QUERY_KEY, payload)
    sessionStorage.setItem(LAST_HOME_QUERY_KEY, payload)
  } catch {
    /* ignore quota / private mode */
  }
}

export function loadLastHomeQuery(): Record<string, string> {
  if (typeof window === 'undefined') return {}
  try {
    const fromLocal = parseStoredQueryRecord(localStorage.getItem(LAST_HOME_QUERY_KEY))
    if (Object.keys(fromLocal).length) return fromLocal
    return parseStoredQueryRecord(sessionStorage.getItem(LAST_HOME_QUERY_KEY))
  } catch {
    return {}
  }
}

/** Fill keys still missing after the last-home snapshot, from older per-field localStorage. */
function backfillHomeQueryFromLegacyStorage(query: Record<string, string>): Record<string, string> {
  if (typeof window === 'undefined') return query
  const out = { ...query }
  try {
    if (!out.langs) {
      const stored = localStorage.getItem('selectedLanguages')
      const ids = stored ? (JSON.parse(stored) as unknown) : null
      if (Array.isArray(ids) && ids.length) {
        const langs = ids.filter((n) => Number.isFinite(Number(n))).join(',')
        if (langs) out.langs = langs
      }
    }
  } catch {
    /* ignore */
  }
  if (!out.mode) {
    const storedMode = localStorage.getItem('searchMode')
    if (storedMode) {
      const normalized =
        storedMode === 'messages' ? 'comments' : storedMode === 'muplis' ? 'semantic' : storedMode
      if (HOME_SEARCH_MODES.has(normalized)) out.mode = normalized
    }
  }
  if (!out.q) {
    const storedQuery = localStorage.getItem('searchQuery')
    if (storedQuery) out.q = storedQuery
  }
  if (!out.searchInPhrases) {
    const stored = localStorage.getItem('searchInPhrases')
    if (stored === 'false') out.searchInPhrases = 'false'
  }
  if (!out.wave_source) {
    const stored = localStorage.getItem('waveSource')
    if (stored) out.wave_source = stored
  }
  if (!out.group_by_thread) {
    const stored = localStorage.getItem('mailSearch_groupByThread')
    if (stored === 'true') out.group_by_thread = 'true'
  }
  return out
}

/** Stored home filters, overlaid with any keys already in the URL (URL wins). */
export function mergeQueryForHomeNavigation(current: LocationQuery): Record<string, string> {
  return {
    ...backfillHomeQueryFromLegacyStorage(loadLastHomeQuery()),
    ...pickHomeQuery(current),
  }
}

/** Copy filter fields from the URL only when those keys are present. */
export function applyCombinedFiltersFromQuery(
  current: CombinedFiltersUrlState,
  query: LocationQuery
): CombinedFiltersUrlState {
  const fromQuery = combinedFiltersFromQuery(query)
  const next = { ...current }
  if (query.langs !== undefined) next.selectedLanguages = fromQuery.selectedLanguages
  if (query.collections !== undefined) next.selectedCollections = fromQuery.selectedCollections
  if (query.username !== undefined) next.usernames = fromQuery.usernames
  if (query.exclude_usernames !== undefined) next.excludeUsernames = fromQuery.excludeUsernames
  if (query.selmaho !== undefined) next.selmaho = fromQuery.selmaho
  if (query.word_type !== undefined) next.word_type = fromQuery.word_type
  if (query.source_langid !== undefined) next.source_langid = fromQuery.source_langid
  if (query.searchInPhrases !== undefined) next.searchInPhrases = fromQuery.searchInPhrases
  if (query.isExpanded !== undefined) next.isExpanded = fromQuery.isExpanded
  return next
}
