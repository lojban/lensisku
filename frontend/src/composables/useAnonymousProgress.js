/**
 * Anonymous (localStorage) progress for public collections.
 * Key: lensisku_anon_progress
 * Shape: { [collectionId]: { levels: { [levelId]: { cards_completed, correct_answers, total_answers, completed_at?, card_attempts? } } } }
 */

const STORAGE_KEY = 'lensisku_anon_progress'

function readRaw () {
  if (typeof window === 'undefined') return {}
  try {
    const s = localStorage.getItem(STORAGE_KEY)
    return s ? JSON.parse(s) : {}
  } catch {
    return {}
  }
}

function writeRaw (data) {
  if (typeof window === 'undefined') return
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data))
  } catch (e) {
    console.warn('useAnonymousProgress: write failed', e)
  }
}

/**
 * @param {string|number} collectionId
 * @param {string|number} [levelId] - if omitted, returns all levels for the collection
 * @returns {Object} progress for the collection, or for the level if levelId provided
 */
export function getProgress (collectionId, levelId) {
  const data = readRaw()
  const col = data[String(collectionId)]
  if (!col?.levels) return levelId == null ? {} : null
  if (levelId == null) return col.levels
  return col.levels[String(levelId)] ?? null
}

/**
 * @param {string|number} collectionId
 * @param {string|number} levelId
 * @param {{ cards_completed: number, correct_answers: number, total_answers: number, completed_at?: string|null, card_attempts?: Array }} data
 */
export function saveLevelProgress (collectionId, levelId, data) {
  const all = readRaw()
  const cid = String(collectionId)
  const lid = String(levelId)
  if (!all[cid]) all[cid] = { levels: {} }
  if (!all[cid].levels[lid]) all[cid].levels[lid] = { cards_completed: 0, correct_answers: 0, total_answers: 0 }
  const cur = all[cid].levels[lid]
  cur.cards_completed = data.cards_completed ?? cur.cards_completed
  cur.correct_answers = data.correct_answers ?? cur.correct_answers
  cur.total_answers = data.total_answers ?? cur.total_answers
  if (data.completed_at !== undefined) cur.completed_at = data.completed_at
  if (data.card_attempts !== undefined) cur.card_attempts = data.card_attempts
  writeRaw(all)
}

/**
 * Returns all progress in the shape expected by the merge API: { collection_id, level_progress: [ { level_id, cards_completed, correct_answers, total_answers } ] }[]
 * @returns {Array<{ collection_id: number, level_progress: Array<{ level_id: number, cards_completed: number, correct_answers: number, total_answers: number }> }>}
 */
export function getAllProgressForMerge () {
  const data = readRaw()
  const out = []
  for (const [cid, col] of Object.entries(data)) {
    if (!col?.levels || typeof col.levels !== 'object') continue
    const levelProgress = []
    for (const [lid, lev] of Object.entries(col.levels)) {
      if (lev && (lev.cards_completed > 0 || lev.correct_answers > 0 || lev.total_answers > 0)) {
        levelProgress.push({
          level_id: parseInt(lid, 10),
          cards_completed: lev.cards_completed ?? 0,
          correct_answers: lev.correct_answers ?? 0,
          total_answers: lev.total_answers ?? 0
        })
      }
    }
    if (levelProgress.length) out.push({ collection_id: parseInt(cid, 10), level_progress: levelProgress })
  }
  return out
}

/**
 * Remove progress for one collection or all after successful merge.
 * @param {string|number|null} [collectionId] - if null/undefined, clear all
 */
export function clearAfterMerge (collectionId) {
  if (collectionId == null || collectionId === '') {
    if (typeof window !== 'undefined') localStorage.removeItem(STORAGE_KEY)
    return
  }
  const all = readRaw()
  delete all[String(collectionId)]
  writeRaw(all)
}

export function useAnonymousProgress () {
  return {
    getProgress,
    saveLevelProgress,
    getAllProgressForMerge,
    clearAfterMerge
  }
}
