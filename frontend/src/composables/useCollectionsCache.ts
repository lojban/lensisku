import { ref } from 'vue'

import { getCollections } from '@/api'

/** Summary row from `GET /collections` used by add-to-collection UIs. */
export type CachedCollection = {
  collection_id: number
  name: string
  item_count: number
  description?: string
  is_public?: boolean
  [key: string]: unknown
}

const collections = ref<CachedCollection[]>([])
const hasLoaded = ref(false)
const isFetching = ref(false)
let inflight: Promise<CachedCollection[]> | null = null

async function loadCollections(): Promise<CachedCollection[]> {
  if (inflight) return inflight

  isFetching.value = true
  inflight = (async () => {
    try {
      const response = await getCollections()
      const list = (response.data?.collections || []) as CachedCollection[]
      collections.value = list
      hasLoaded.value = true
      return list
    } catch (error) {
      console.error('Error fetching collections:', error)
      return collections.value
    } finally {
      isFetching.value = false
      inflight = null
    }
  })()

  return inflight
}

/**
 * Shared cache for the logged-in user's collection list.
 * Preload once so CollectionWidget opens instantly; call `refresh` on open
 * to revalidate in the background and update the list in place.
 */
export function useCollectionsCache() {
  return {
    collections,
    hasLoaded,
    isFetching,
    /** Fetch if not yet loaded; concurrent callers share one request. */
    preload: () => {
      if (hasLoaded.value) return Promise.resolve(collections.value)
      return loadCollections()
    },
    /** Always refetch (deduped while a request is in flight). */
    refresh: () => loadCollections(),
    setCollections: (list: CachedCollection[]) => {
      collections.value = list
      hasLoaded.value = true
    },
    clear: () => {
      collections.value = []
      hasLoaded.value = false
      inflight = null
      isFetching.value = false
    },
  }
}
