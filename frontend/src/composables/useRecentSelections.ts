import { ref, type Ref } from 'vue'

const RECENT_MAX = 3

/**
 * Last-N selected items persisted in localStorage, most recent first.
 * Used as “suggestion copies” at the top of language, collection, and author dropdowns.
 */
export function useRecentSelections<T>(storageKey: string, getId: (item: T) => string | number) {
  const recent = ref(load()) as Ref<T[]>

  function load(): T[] {
    if (typeof window === 'undefined') return []
    try {
      const raw = localStorage.getItem(storageKey)
      const parsed = raw ? JSON.parse(raw) : []
      return Array.isArray(parsed) ? (parsed as T[]).slice(0, RECENT_MAX) : []
    } catch {
      return []
    }
  }

  function persist() {
    if (typeof window === 'undefined') return
    try {
      localStorage.setItem(storageKey, JSON.stringify(recent.value.slice(0, RECENT_MAX)))
    } catch (e) {
      console.error('Failed to persist recent selections:', e)
    }
  }

  function record(item: T) {
    const id = getId(item)
    recent.value = [item, ...recent.value.filter((x) => getId(x) !== id)].slice(0, RECENT_MAX)
    persist()
  }

  return { recent, record }
}
