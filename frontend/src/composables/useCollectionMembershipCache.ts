import { ref } from 'vue'

import { getCollectionMembershipBatch } from '@/api'

export type MembershipQuery = {
  definition_id?: number
  item_id?: number
}

const MAX_BATCH = 100

/** `i:123` / `d:456` → collection ids. Key present (even with `[]`) means the check finished. */
const membershipByKey = ref<Record<string, number[]>>({})

type Queued = { key: string } & MembershipQuery
let queued: Queued[] = []
const waiters = new Map<string, Array<(ids: number[]) => void>>()
let flushScheduled = false
let flushing = false

export function membershipCacheKey(q: MembershipQuery): string | null {
  if (q.item_id != null && q.item_id > 0) return `i:${q.item_id}`
  if (q.definition_id != null && q.definition_id > 0) return `d:${q.definition_id}`
  return null
}

/** Match CollectionWidget: collection items use `item_id`, dictionary rows use `definition_id`. */
export function membershipQueryFromCard(def: {
  item_id?: number | null
  collection_id?: number | null
  definitionid?: number | null
}): MembershipQuery {
  if (def.item_id != null && def.item_id > 0 && def.collection_id != null) {
    return { item_id: def.item_id }
  }
  if (def.definitionid != null && def.definitionid > 0) {
    return { definition_id: def.definitionid }
  }
  return {}
}

function setIds(key: string, ids: number[]) {
  membershipByKey.value = { ...membershipByKey.value, [key]: ids }
}

function resolveWaiters(key: string, ids: number[]) {
  const pending = waiters.get(key)
  waiters.delete(key)
  pending?.forEach((resolve) => resolve(ids))
}

async function flushQueue() {
  while (queued.length > 0) {
    const batch = queued.splice(0, MAX_BATCH)
    const items = batch.map(({ definition_id, item_id }) => ({
      ...(item_id ? { item_id } : {}),
      ...(definition_id && !item_id ? { definition_id } : {}),
    }))
    try {
      const response = await getCollectionMembershipBatch({ items })
      const results = (response.data?.results || []) as Array<{ collection_ids?: number[] }>
      batch.forEach((entry, index) => {
        const ids = (results[index]?.collection_ids || []) as number[]
        setIds(entry.key, ids)
        resolveWaiters(entry.key, ids)
      })
    } catch (error) {
      console.error('Error checking collection membership:', error)
      batch.forEach((entry) => {
        setIds(entry.key, [])
        resolveWaiters(entry.key, [])
      })
    }
  }
}

function scheduleFlush() {
  if (flushScheduled || flushing) return
  flushScheduled = true
  queueMicrotask(() => {
    flushScheduled = false
    void (async () => {
      if (flushing) return
      flushing = true
      try {
        await flushQueue()
      } finally {
        flushing = false
        if (queued.length > 0) scheduleFlush()
      }
    })()
  })
}

/**
 * Shared cache so every search-result star can resolve “already included”
 * before the modal opens. Concurrent preloads in the same tick are one batch request.
 */
export function useCollectionMembershipCache() {
  return {
    membershipByKey,
    isReady: (key: string | null) => {
      if (!key) return true
      return Object.prototype.hasOwnProperty.call(membershipByKey.value, key)
    },
    idsFor: (key: string | null) => {
      if (!key) return [] as number[]
      return membershipByKey.value[key] ?? []
    },
    /** Fetch if missing; no-op when already cached. */
    preload: (query: MembershipQuery) => {
      const key = membershipCacheKey(query)
      if (!key) return Promise.resolve([] as number[])
      if (Object.prototype.hasOwnProperty.call(membershipByKey.value, key)) {
        return Promise.resolve(membershipByKey.value[key] ?? [])
      }
      return new Promise<number[]>((resolve) => {
        const existing = waiters.get(key)
        if (existing) {
          existing.push(resolve)
          return
        }
        waiters.set(key, [resolve])
        queued.push({ key, ...query })
        scheduleFlush()
      })
    },
    /** Drop-in refetch; overwrites the cache when the request finishes. */
    refresh: (query: MembershipQuery) => {
      const key = membershipCacheKey(query)
      if (!key) return Promise.resolve([] as number[])
      return new Promise<number[]>((resolve) => {
        const existing = waiters.get(key)
        if (existing) {
          existing.push(resolve)
        } else {
          waiters.set(key, [resolve])
        }
        if (!queued.some((entry) => entry.key === key)) {
          queued.push({ key, ...query })
        }
        scheduleFlush()
      })
    },
    /** Optimistically record that this card is now in `collectionId`. */
    markIncluded: (query: MembershipQuery, collectionId: number) => {
      const key = membershipCacheKey(query)
      if (!key || !collectionId) return
      const current = membershipByKey.value[key] ?? []
      if (current.includes(collectionId)) return
      setIds(key, [collectionId, ...current])
    },
  }
}
