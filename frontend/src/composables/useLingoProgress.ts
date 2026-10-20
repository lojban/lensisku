import { computed, ref } from 'vue'
import { LINGO_MAX_HEARTS, LINGO_POINTS_TO_REFILL } from '@/config/lingoConstants'

const STORAGE_KEY = 'lingo_progress_v1'
const ACTIVE_COURSE_KEY = 'lingo_active_collection_id'

type LingoProgressState = {
  hearts: number
  points: number
  isPro: boolean
  activeCourseId: number | null
  activeCourseName: string
}

function readState(): LingoProgressState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<LingoProgressState>
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
  }
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
  const hasUnlimitedHearts = computed(() => state.value.isPro)

  function setActiveCourse(id: number | null, name = '') {
    state.value = {
      ...state.value,
      activeCourseId: id,
      activeCourseName: name || state.value.activeCourseName,
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
