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

function queryCsv(v: string | string[] | null | undefined): string {
  if (v == null) return ''
  if (Array.isArray(v)) {
    return v
      .flatMap((part) => String(part).split(','))
      .map((s) => s.trim())
      .filter(Boolean)
      .join(',')
  }
  return String(v)
}

/** Drop empty/undefined keys so vue-router actually removes them from the URL. */
export function compactQuery(query: Record<string, unknown>): Record<string, string> {
  const out: Record<string, string> = {}
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined || value === null || value === '') continue
    if (Array.isArray(value)) {
      const joined = queryCsv(value as string[])
      if (!joined) continue
      out[key] = joined
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
  const langs = queryCsv(query.langs)
  const collections = queryCsv(query.collections)
  const username = queryCsv(query.username)
  const exclude = queryCsv(query.exclude_usernames)
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

export function pickHomeQuery(
  query: LocationQuery | Record<string, unknown>
): Record<string, string> {
  const out: Record<string, string> = {}
  for (const key of HOME_PRESERVED_QUERY_KEYS) {
    const s = queryCsv(query[key] as string | string[] | null | undefined)
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
    return pickHomeQuery(parsed as Record<string, unknown>)
  } catch {
    return {}
  }
}

function readJsonArray(key: string): unknown[] | null {
  if (typeof window === 'undefined') return null
  try {
    const raw = localStorage.getItem(key)
    if (!raw) return null
    const parsed = JSON.parse(raw) as unknown
    return Array.isArray(parsed) ? parsed : null
  } catch {
    return null
  }
}

function writeLastHomeQuery(record: Record<string, string>): void {
  if (typeof window === 'undefined') return
  try {
    const payload = JSON.stringify(record)
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

/** Older per-field keys, used only when the unified snapshot is missing that param. */
function backfillLegacyHomeQuery(query: Record<string, string>): Record<string, string> {
  if (typeof window === 'undefined') return query
  const out = { ...query }
  const csvFromIds = (key: string, min = 0): string => {
    const ids = readJsonArray(key)
    if (!ids?.length) return ''
    return ids
      .map(Number)
      .filter((n) => Number.isFinite(n) && n >= min)
      .join(',')
  }
  const csvFromNames = (key: string): string => {
    const names = readJsonArray(key)
    if (!names?.length) return ''
    return names.map(String).filter(Boolean).join(',')
  }
  if (!out.langs) {
    const langs = csvFromIds('selectedLanguages')
    if (langs) out.langs = langs
  }
  if (!out.collections) {
    const collections = csvFromIds('selectedCollections', 1)
    if (collections) out.collections = collections
  }
  if (!out.username) {
    const username = csvFromNames('selectedUsernames')
    if (username) out.username = username
  }
  if (!out.exclude_usernames) {
    const exclude = csvFromNames('excludeUsernames')
    if (exclude) out.exclude_usernames = exclude
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
  if (!out.searchInPhrases && localStorage.getItem('searchInPhrases') === 'false') {
    out.searchInPhrases = 'false'
  }
  if (!out.wave_source) {
    const stored = localStorage.getItem('waveSource')
    if (stored) out.wave_source = stored
  }
  if (!out.group_by_thread && localStorage.getItem('mailSearch_groupByThread') === 'true') {
    out.group_by_thread = 'true'
  }
  return out
}

/**
 * One restore rule for every home param (`langs`, `collections`, `username`, …):
 * stored snapshot first, current URL keys overlay (URL wins).
 */
export function resolveHomeQuery(current: LocationQuery): Record<string, string> {
  return { ...backfillLegacyHomeQuery(loadLastHomeQuery()), ...pickHomeQuery(current) }
}

export const mergeQueryForHomeNavigation = resolveHomeQuery

/** Persist a complete home URL. Bare `/` (no preserved keys) does not wipe storage. */
export function saveLastHomeQuery(
  query: LocationQuery | Record<string, unknown>
): Record<string, string> {
  const picked = pickHomeQuery(query)
  if (Object.keys(picked).length === 0) return picked
  writeLastHomeQuery(picked)
  return picked
}

/** Compact, persist, and return the query to `router.push`. */
export function commitHomeQuery(query: Record<string, unknown>): Record<string, string> {
  const compacted = compactQuery(query)
  saveLastHomeQuery(compacted)
  return compacted
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
