import { ref } from 'vue'
import { getRecentChanges } from '@/api'

const key = 'lensisku:activity:last-seen:v1'
const hasNewActivity = ref(false)

export function useActivityUnread() {
  function observe(eventId?: string) {
    if (typeof window === 'undefined' || !eventId) return
    try {
      const lastSeen = localStorage.getItem(key)
      hasNewActivity.value = !!lastSeen && lastSeen !== eventId
    } catch {
      /* Private browsing may disable storage. */
    }
  }
  function markDisplayed(eventId?: string) {
    if (typeof window === 'undefined' || !eventId) return
    try {
      localStorage.setItem(key, eventId)
    } catch {
      /* Best effort. */
    }
    hasNewActivity.value = false
  }
  async function refresh() {
    try {
      const response = await getRecentChanges({ view: 'feed', limit: 1 })
      observe(response.data.changes[0]?.event_id)
    } catch {
      /* A badge failure must not disrupt navigation. */
    }
  }
  return { hasNewActivity, observe, markDisplayed, refresh }
}
