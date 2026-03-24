/**
 * Anonymous (localStorage) progress for public collections.
 * Key: lensisku_anon_progress
 */

const STORAGE_KEY = 'lensisku_anon_progress'

export interface LevelProgressData {
  cards_completed: number
  correct_answers: number
  total_answers: number
  completed_at?: string | null
  card_attempts?: unknown[]
}

type RawProgress = Record<
  string,
  {
    levels?: Record<string, LevelProgressData & Record<string, unknown>>
  }
>

function readRaw(): RawProgress {
  if (typeof window === 'undefined') return {}
  try {
    const s = localStorage.getItem(STORAGE_KEY)
    return s ? (JSON.parse(s) as RawProgress) : {}
  } catch {
    return {}
  }
}

function writeRaw(data: RawProgress): void {
  if (typeof window === 'undefined') return
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data))
  } catch (e) {
    console.warn('useAnonymousProgress: write failed', e)
  }
}

export function getProgress(
  collectionId: string | number,
  levelId?: string | number | null
): Record<string, LevelProgressData> | LevelProgressData | null | Record<string, never> {
  const data = readRaw()
  const col = data[String(collectionId)]
  if (!col?.levels) return levelId == null ? {} : null
  if (levelId == null) return col.levels as Record<string, LevelProgressData>
  return (col.levels[String(levelId)] as LevelProgressData | undefined) ?? null
}

export function saveLevelProgress(
  collectionId: string | number,
  levelId: string | number,
  data: Partial<LevelProgressData>
): void {
  const all = readRaw()
  const cid = String(collectionId)
  const lid = String(levelId)
  if (!all[cid]) all[cid] = { levels: {} }
  if (!all[cid].levels) all[cid].levels = {}
  if (!all[cid].levels![lid])
    all[cid].levels![lid] = { cards_completed: 0, correct_answers: 0, total_answers: 0 }
  const cur = all[cid].levels![lid] as LevelProgressData
  cur.cards_completed = data.cards_completed ?? cur.cards_completed
  cur.correct_answers = data.correct_answers ?? cur.correct_answers
  cur.total_answers = data.total_answers ?? cur.total_answers
  if (data.completed_at !== undefined) cur.completed_at = data.completed_at
  if (data.card_attempts !== undefined) cur.card_attempts = data.card_attempts
  writeRaw(all)
}

export interface MergeLevelProgressRow {
  level_id: number
  cards_completed: number
  correct_answers: number
  total_answers: number
}

export interface MergeProgressPayload {
  collection_id: number
  level_progress: MergeLevelProgressRow[]
}

export function getAllProgressForMerge(): MergeProgressPayload[] {
  const data = readRaw()
  const out: MergeProgressPayload[] = []
  for (const [cid, col] of Object.entries(data)) {
    if (!col?.levels || typeof col.levels !== 'object') continue
    const levelProgress: MergeLevelProgressRow[] = []
    for (const [lid, lev] of Object.entries(col.levels)) {
      const l = lev as LevelProgressData
      if (l && (l.cards_completed > 0 || l.correct_answers > 0 || l.total_answers > 0)) {
        levelProgress.push({
          level_id: parseInt(lid, 10),
          cards_completed: l.cards_completed ?? 0,
          correct_answers: l.correct_answers ?? 0,
          total_answers: l.total_answers ?? 0,
        })
      }
    }
    if (levelProgress.length)
      out.push({ collection_id: parseInt(cid, 10), level_progress: levelProgress })
  }
  return out
}

export function clearAfterMerge(collectionId?: string | number | null): void {
  if (collectionId == null || collectionId === '') {
    if (typeof window !== 'undefined') localStorage.removeItem(STORAGE_KEY)
    return
  }
  const all = readRaw()
  delete all[String(collectionId)]
  writeRaw(all)
}

export function useAnonymousProgress() {
  return {
    getProgress,
    saveLevelProgress,
    getAllProgressForMerge,
    clearAfterMerge,
  }
}
