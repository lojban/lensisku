import { computed, ref } from 'vue'
import { LINGO_MAX_HEARTS, LINGO_POINTS_TO_REFILL } from '@/config/lingoConstants'

const STORAGE_KEY = 'lingo_progress_v1'
const ACTIVE_COURSE_KEY = 'lingo_active_collection_id'
const MAX_RECENT_COURSES = 5

export type LingoRecentCourse = {
  id: number
  name: string
}

type LingoProgressState = {
  hearts: number
  points: number
  isPro: boolean
  activeCourseId: number | null
  activeCourseName: string
  recentCourses: LingoRecentCourse[]
}

function readState(): LingoProgressState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<LingoProgressState>
      let recentCourses = parseRecentCourses(parsed.recentCourses)
      if (
        recentCourses.length === 0 &&
        typeof parsed.activeCourseId === 'number' &&
        Number.isFinite(parsed.activeCourseId)
      ) {
        recentCourses = pushRecentCourse(
          [],
          parsed.activeCourseId,
          typeof parsed.activeCourseName === 'string' ? parsed.activeCourseName : ''
        )
      }
      return {
        hearts:
          typeof parsed.hearts === 'number'
            ? Math.min(LINGO_MAX_HEARTS, Math.max(0, parsed.hearts))
            : LINGO_MAX_HEARTS,
        points: typeof parsed.points === 'number' ? Math.max(0, parsed.points) : 0,
        isPro: !!parsed.isPro,
        activeCourseId: typeof parsed.activeCourseId === 'number' ? parsed.activeCourseId : null,
        activeCourseName:
          typeof parsed.activeCourseName === 'string' ? parsed.activeCourseName : '',
        recentCourses,
      }
    }
  } catch {
    /* ignore */
  }
  let activeCourseId: number | null = null
  try {
    const id = sessionStorage.getItem(ACTIVE_COURSE_KEY)
    if (id) activeCourseId = Number(id)
  } catch {
    /* ignore */
  }
  return {
    hearts: LINGO_MAX_HEARTS,
    points: 0,
    isPro: false,
    activeCourseId,
    activeCourseName: '',
    recentCourses: [],
  }
}

function parseRecentCourses(raw: unknown): LingoRecentCourse[] {
  if (!Array.isArray(raw)) return []
  const out: LingoRecentCourse[] = []
  for (const entry of raw) {
    if (!entry || typeof entry !== 'object') continue
    const { id, name } = entry as { id?: unknown; name?: unknown }
    if (typeof id !== 'number' || !Number.isFinite(id)) continue
    out.push({
      id,
      name: typeof name === 'string' && name.trim() ? name : '',
    })
    if (out.length >= MAX_RECENT_COURSES) break
  }
  return out
}

function pushRecentCourse(
  recent: LingoRecentCourse[],
  id: number,
  name: string
): LingoRecentCourse[] {
  const label = name.trim() || recent.find((c) => c.id === id)?.name || ''
  const without = recent.filter((c) => c.id !== id)
  return [{ id, name: label }, ...without].slice(0, MAX_RECENT_COURSES)
}

const state = ref<LingoProgressState>(readState())

function persist() {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state.value))
  } catch {
    /* ignore */
  }
  try {
    if (state.value.activeCourseId != null) {
      sessionStorage.setItem(ACTIVE_COURSE_KEY, String(state.value.activeCourseId))
    } else {
      sessionStorage.removeItem(ACTIVE_COURSE_KEY)
    }
  } catch {
    /* ignore */
  }
}

/** Shared Lingo XP / hearts / active course (Duolingo-clone parity, localStorage-backed). */
export function useLingoProgress() {
  const hearts = computed(() => state.value.hearts)
  const points = computed(() => state.value.points)
  const isPro = computed(() => state.value.isPro)
  const activeCourseId = computed(() => state.value.activeCourseId)
  const activeCourseName = computed(() => state.value.activeCourseName)
  const recentCourses = computed(() => state.value.recentCourses)
  const hasUnlimitedHearts = computed(() => state.value.isPro)

  function setActiveCourse(id: number | null, name = '') {
    const nextName = name || state.value.activeCourseName
    const recentCourses =
      id != null
        ? pushRecentCourse(state.value.recentCourses, id, nextName)
        : state.value.recentCourses
    state.value = {
      ...state.value,
      activeCourseId: id,
      activeCourseName: nextName,
      recentCourses,
    }
    persist()
  }

  function addPoints(amount: number) {
    if (!amount) return
    state.value = { ...state.value, points: state.value.points + amount }
    persist()
  }

  function loseHeart() {
    if (state.value.isPro) return
    if (state.value.hearts <= 0) return
    state.value = { ...state.value, hearts: state.value.hearts - 1 }
    persist()
  }

  function refillHearts(): boolean {
    if (state.value.isPro) return true
    if (state.value.hearts >= LINGO_MAX_HEARTS) return false
    if (state.value.points < LINGO_POINTS_TO_REFILL) return false
    state.value = {
      ...state.value,
      points: state.value.points - LINGO_POINTS_TO_REFILL,
      hearts: LINGO_MAX_HEARTS,
    }
    persist()
    return true
  }

  function enablePro() {
    state.value = { ...state.value, isPro: true, hearts: LINGO_MAX_HEARTS }
    persist()
  }

  function disablePro() {
    state.value = { ...state.value, isPro: false }
    persist()
  }

  /** Award session XP when leaving a finished lesson (10 XP per card answered). */
  function awardSessionXp(cardsAnswered: number) {
    const xp = Math.max(0, cardsAnswered) * 10
    if (xp > 0) addPoints(xp)
    return xp
  }

  return {
    hearts,
    points,
    isPro,
    hasUnlimitedHearts,
    activeCourseId,
    activeCourseName,
    recentCourses,
    setActiveCourse,
    addPoints,
    loseHeart,
    refillHearts,
    enablePro,
    disablePro,
    awardSessionXp,
    maxHearts: LINGO_MAX_HEARTS,
    pointsToRefill: LINGO_POINTS_TO_REFILL,
  }
}
