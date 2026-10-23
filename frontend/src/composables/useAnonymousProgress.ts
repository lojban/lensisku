/**
 * Anonymous (localStorage) progress for public collections.
 * Key: lensisku_anon_progress
 *
 * Unlock for guests is derived from prerequisites + each level's `completed_at`.
 * `active_level_id` remembers the current playable level (first unlocked incomplete).
 */

import { ref } from 'vue'

const STORAGE_KEY = 'lensisku_anon_progress'

/** Bumped on every write so Vue computeds that read localStorage re-evaluate. */
export const anonProgressVersion = ref(0)

export interface LevelProgressData {
  cards_completed: number
  correct_answers: number
  total_answers: number
  completed_at?: string | null
  card_attempts?: unknown[]
}

export interface AnonLevelLike {
  level_id: number | string
  prerequisites?: Array<{ level_id: number | string }> | null
  position?: number
}

type RawProgress = Record<
  string,
  {
    levels?: Record<string, LevelProgressData & Record<string, unknown>>
    /** Current playable level for this course (prereq-aware). */
    active_level_id?: number | null
  }
>

function bumpVersion(): void {
  anonProgressVersion.value += 1
}

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
    bumpVersion()
  } catch (e) {
    console.warn('useAnonymousProgress: write failed', e)
  }
}

export function getProgress(
  collectionId: string | number,
  levelId?: string | number | null
): Record<string, LevelProgressData> | LevelProgressData | null | Record<string, never> {
  // Touch reactive version so callers in computed() re-run after saves.
  void anonProgressVersion.value
  const data = readRaw()
  const col = data[String(collectionId)]
  if (!col?.levels) return levelId == null ? {} : null
  if (levelId == null) return col.levels as Record<string, LevelProgressData>
  return (col.levels[String(levelId)] as LevelProgressData | undefined) ?? null
}

/** True when every prerequisite has `completed_at` in localStorage (or no prereqs). */
export function isLevelUnlockedForAnon(
  collectionId: string | number,
  level: AnonLevelLike | null | undefined
): boolean {
  void anonProgressVersion.value
  if (!level?.prerequisites?.length) return true
  return level.prerequisites.every((p) => {
    const local = getProgress(collectionId, p.level_id) as LevelProgressData | null
    return local?.completed_at != null
  })
}

export function isLevelCompletedForAnon(
  collectionId: string | number,
  levelId: string | number
): boolean {
  void anonProgressVersion.value
  const local = getProgress(collectionId, levelId) as LevelProgressData | null
  return local?.completed_at != null
}

/**
 * First unlocked, incomplete level by position; falls back to first unlocked,
 * then first level. Also persists `active_level_id`.
 */
export function resolveAndRememberActiveLevelId(
  collectionId: string | number,
  levels: AnonLevelLike[]
): number | null {
  void anonProgressVersion.value
  if (!levels.length) {
    setActiveLevelId(collectionId, null)
    return null
  }
  const sorted = [...levels].sort((a, b) => (a.position ?? 0) - (b.position ?? 0))
  const next =
    sorted.find(
      (l) =>
        isLevelUnlockedForAnon(collectionId, l) &&
        !isLevelCompletedForAnon(collectionId, l.level_id)
    ) ??
    sorted.find((l) => isLevelUnlockedForAnon(collectionId, l)) ??
    sorted[0]
  const id = next?.level_id != null ? Number(next.level_id) : null
  const activeId = id != null && Number.isFinite(id) ? id : null
  setActiveLevelId(collectionId, activeId)
  return activeId
}

export function getActiveLevelId(collectionId: string | number): number | null {
  void anonProgressVersion.value
  const col = readRaw()[String(collectionId)]
  const id = col?.active_level_id
  return typeof id === 'number' && Number.isFinite(id) ? id : null
}

export function setActiveLevelId(collectionId: string | number, levelId: number | null): void {
  const all = readRaw()
  const cid = String(collectionId)
  if (!all[cid]) all[cid] = { levels: {} }
  if (all[cid].active_level_id === levelId) return
  all[cid].active_level_id = levelId
  writeRaw(all)
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
  // Studying this level makes it the remembered active node until completion advances.
  const numericLid = Number(levelId)
  if (Number.isFinite(numericLid) && !cur.completed_at) {
    all[cid].active_level_id = numericLid
  }
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
    bumpVersion()
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
    isLevelUnlockedForAnon,
    isLevelCompletedForAnon,
    resolveAndRememberActiveLevelId,
    getActiveLevelId,
    setActiveLevelId,
    anonProgressVersion,
  }
}
