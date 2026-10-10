import { onBeforeUnmount, onMounted, ref, shallowRef, watch, type Ref } from 'vue'
import { getRecentChanges } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useActivityUnread } from '@/composables/useActivityUnread'

export type ActivityRow = {
  event_id: string
  change_type: string
  source: 'native' | 'imported'
  source_name: string
  word: string
  time: number
  content: unknown
  definition_id?: number
  comment_id?: number
  thread_id?: number
  username: string
  is_wiki?: boolean
  diff?: { new_content?: { definition?: string }; changes?: unknown[] }
  [key: string]: unknown
}
type Snapshot = { rows: ActivityRow[]; cursor: string | null; scroll: number }
const snapshots = new Map<string, Snapshot>()
export function invalidateActivityFeed() {
  snapshots.clear()
}
const types: Record<string, string> = {
  all: 'comment,definition,wiki,message',
  discussions: 'comment,message',
  definitions: 'definition',
  wiki: 'wiki',
}

export function useActivityFeed(filter: Ref<string>, location: Ref<string>) {
  const auth = useAuth()
  const unread = useActivityUnread()
  const rows = shallowRef<ActivityRow[]>([])
  const cursor = ref<string | null>(null)
  const loading = ref(false)
  const error = ref(false)
  const newActivity = ref(false)
  const restoredScroll = ref(0)
  let controller: AbortController | undefined
  let generation = 0
  let mounted = false
  let currentKey = ''
  let latestId = ''
  let failedAppend = false
  let pollTimer: ReturnType<typeof setInterval> | undefined
  const key = () => `${auth.state.userId || 'anonymous'}:${location.value}:${filter.value}`
  const params = () => ({
    view: 'feed',
    source: 'all',
    types: types[filter.value],
    limit: 20,
  })
  const save = (scroll?: number) => {
    if (!currentKey || !rows.value.length) return
    snapshots.set(currentKey, {
      rows: rows.value,
      cursor: cursor.value,
      scroll: scroll ?? snapshots.get(currentKey)?.scroll ?? restoredScroll.value,
    })
    if (snapshots.size > 12) snapshots.delete(snapshots.keys().next().value)
  }
  async function load(append = false) {
    if (filter.value === 'news') return
    if (append && (loading.value || !cursor.value)) return
    controller?.abort()
    controller = new AbortController()
    const signal = controller.signal
    const request = ++generation
    loading.value = true
    failedAppend = append
    error.value = false
    try {
      const response = await getRecentChanges(
        { ...params(), ...(append && { after: cursor.value }) },
        signal
      )
      if (request !== generation) return
      const incoming = response.data.changes as ActivityRow[]
      const seen = new Set(append ? rows.value.map((row) => row.event_id) : [])
      rows.value = [
        ...(append ? rows.value : []),
        ...incoming.filter((row) => {
          if (seen.has(row.event_id)) return false
          seen.add(row.event_id)
          return true
        }),
      ]
      cursor.value = response.data.next_cursor || null
      if (!append) {
        latestId = incoming[0]?.event_id || ''
        newActivity.value = false
        if (filter.value === 'all') unread.markDisplayed(latestId)
      }
      save()
    } catch {
      if (request === generation && !signal.aborted) error.value = true
    } finally {
      if (request === generation) loading.value = false
    }
  }
  async function poll() {
    if (filter.value === 'news') return
    if (!mounted || document.visibilityState !== 'visible' || loading.value || !rows.value.length)
      return
    const request = generation
    try {
      const response = await getRecentChanges({ ...params(), limit: 1 })
      if (request !== generation || !mounted) return
      const head = response.data.changes[0]?.event_id
      newActivity.value = !!head && head !== latestId
      if (filter.value === 'all') unread.observe(head)
    } catch {
      /* Retry on the next visible poll. */
    }
  }
  function setVisible(visible: boolean) {
    if (pollTimer) clearInterval(pollTimer)
    pollTimer = undefined
    if (visible && filter.value !== 'news') pollTimer = setInterval(() => void poll(), 60_000)
  }
  function select() {
    controller?.abort()
    generation++
    save()
    currentKey = key()
    if (filter.value === 'news') {
      rows.value = []
      cursor.value = null
      restoredScroll.value = 0
      latestId = ''
      newActivity.value = false
      error.value = false
      loading.value = false
      setVisible(false)
      return
    }
    const snapshot = snapshots.get(currentKey)
    rows.value = snapshot?.rows || []
    cursor.value = snapshot?.cursor || null
    restoredScroll.value = snapshot?.scroll || 0
    latestId = rows.value[0]?.event_id || ''
    newActivity.value = false
    error.value = false
    loading.value = false
    if (snapshot) void poll()
    else void load()
  }
  watch([filter, location, () => auth.state.userId, () => auth.state.isLoading], () => {
    if (mounted && !auth.state.isLoading) select()
  })
  onMounted(() => {
    mounted = true
    if (!auth.state.isLoading) select()
  })
  onBeforeUnmount(() => {
    mounted = false
    controller?.abort()
    generation++
    setVisible(false)
  })
  return {
    rows,
    cursor,
    loading,
    error,
    newActivity,
    restoredScroll,
    load,
    retry: () => load(failedAppend),
    save,
    setVisible,
  }
}
