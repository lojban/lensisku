import { computed, ref } from 'vue'

import { getCollectionItemLevels, getLevels, type ItemLevelEntry } from '@/api'

export interface CourseLevelPrerequisite {
  level_id: number
  name: string
  is_completed?: boolean
}

export interface CourseLevel {
  level_id: number
  name: string
  description: string | null
  min_cards: number
  min_success_rate: number
  position: number
  prerequisites: CourseLevelPrerequisite[]
  card_count: number
  is_locked: boolean
}

/**
 * Levels of a collection plus the ordered flashcard ids of each level, used by the
 * course-order view and the flashcards page (shared level management).
 */
export function useCourseLevels(collectionId: () => number | string | null | undefined) {
  const levels = ref<CourseLevel[]>([])
  const entries = ref<ItemLevelEntry[]>([])
  const isLoaded = ref(false)

  /** level_id -> ordered flashcard ids */
  const cardsByLevel = computed(() => {
    const map = new Map<number, number[]>()
    for (const l of levels.value) map.set(l.level_id, [])
    for (const e of entries.value) {
      const list = map.get(e.level_id)
      if (list && !list.includes(e.flashcard_id)) list.push(e.flashcard_id)
    }
    return map
  })

  const assignedFlashcardIds = computed(() => new Set(entries.value.map((e) => e.flashcard_id)))

  async function load() {
    const id = collectionId()
    if (id == null || id === '') return
    try {
      const [lv, il] = await Promise.all([getLevels(id), getCollectionItemLevels(id)])
      const list = (lv.data?.levels ?? []) as CourseLevel[]
      levels.value = [...list].sort((a, b) => a.position - b.position || a.level_id - b.level_id)
      entries.value = Array.isArray(il.data) ? il.data : []
    } catch (error) {
      console.error('Error loading course levels:', error)
    } finally {
      isLoaded.value = true
    }
  }

  return { levels, entries, cardsByLevel, assignedFlashcardIds, isLoaded, load }
}
