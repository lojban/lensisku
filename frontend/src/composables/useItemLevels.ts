import { computed, ref } from 'vue'

import { getCollectionItemLevels, type ItemLevelEntry } from '@/api'

/**
 * Level memberships of a collection's items, loaded with a single batched request
 * (`GET /flashcards/collections/:id/item-levels`) and indexed by item id.
 */
export function useItemLevels(collectionId: () => number | string | null | undefined) {
  const entries = ref<ItemLevelEntry[]>([])

  const byItemId = computed(() => {
    const map = new Map<number, ItemLevelEntry[]>()
    for (const e of entries.value) {
      const list = map.get(e.item_id)
      if (list) {
        if (!list.some((x) => x.level_id === e.level_id)) list.push(e)
      } else {
        map.set(e.item_id, [e])
      }
    }
    return map
  })

  async function load() {
    const id = collectionId()
    if (id == null || id === '') return
    try {
      const { data } = await getCollectionItemLevels(id)
      entries.value = Array.isArray(data) ? data : []
    } catch (error) {
      console.error('Error loading item levels:', error)
      entries.value = []
    }
  }

  function levelsForItem(itemId: number | null | undefined) {
    if (itemId == null) return []
    return byItemId.value.get(Number(itemId)) ?? []
  }

  return { entries, byItemId, load, levelsForItem }
}
