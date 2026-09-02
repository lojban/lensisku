import { computed, ref } from 'vue'

import { getRecentChanges } from '@/api'

/** Unix seconds of last time the News tab was opened. */
export const NEWS_LAST_OPENED_KEY = 'news_last_opened_at'

/** Max changes fetched when computing the unread badge. */
const UNREAD_FETCH_LIMIT = 100

/** Shared across App / Home / RecentChanges (module singleton). */
const unreadCount = ref<number | null>(null)
const isFetching = ref(false)

function readLastOpenedAt(): number | null {
  if (typeof window === 'undefined') return null
  const raw = localStorage.getItem(NEWS_LAST_OPENED_KEY)
  if (raw == null || raw === '') return null
  const n = Number(raw)
  return Number.isFinite(n) && n > 0 ? n : null
}

/**
 * Unread wiki news since the user last opened the News tab.
 * Badge stays hidden until localStorage has a last-opened stamp and a fetch completes with count > 0.
 */
export function useNewsUnread() {
  const badgeCount = computed(() => {
    const n = unreadCount.value
    if (n == null || n <= 0) return null
    return n
  })

  const badgeLabel = computed(() => {
    const n = badgeCount.value
    if (n == null) return ''
    return n > 99 ? '99+' : String(n)
  })

  function markNewsOpened() {
    if (typeof window === 'undefined') return
    const now = Math.floor(Date.now() / 1000)
    localStorage.setItem(NEWS_LAST_OPENED_KEY, String(now))
    unreadCount.value = 0
  }

  async function fetchUnreadCount(signal?: AbortSignal) {
    const since = readLastOpenedAt()
    if (since == null) {
      unreadCount.value = null
      return
    }
    if (isFetching.value) return
    isFetching.value = true
    try {
      const response = await getRecentChanges(
        { limit: UNREAD_FETCH_LIMIT, types: 'wiki' },
        signal
      )
      const changes = (response.data?.changes ?? []) as Array<{ time?: number }>
      unreadCount.value = changes.filter((c) => typeof c.time === 'number' && c.time > since).length
    } catch (e: unknown) {
      const name = e && typeof e === 'object' && 'name' in e ? (e as { name?: string }).name : ''
      if (name !== 'AbortError' && name !== 'CanceledError') {
        console.error('Failed to fetch news unread count:', e)
      }
    } finally {
      isFetching.value = false
    }
  }

  return {
    unreadCount,
    badgeCount,
    badgeLabel,
    fetchUnreadCount,
    markNewsOpened,
    readLastOpenedAt,
  }
}
