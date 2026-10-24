import { ref, type Ref } from 'vue'
import { onApiPendingChange, getApiPendingCount } from '@/api'

/**
 * Path-change overlay only (not query/hash): freeze the outgoing view, show a spinner,
 * reveal the next view when in-flight API requests go quiet (no per-page hooks).
 * In-page loads (filters, tabs) use a scoped spinner on the changing region instead.
 */

const QUIET_MS = 120
/** Never keep the freeze up waiting on a slow remote API (chat is not axios). */
const MAX_HOLD_MS = 900

/** Scroll hosts whose `scrollTop` must survive the HTML freeze snapshot. */
const SCROLL_RESTORE_SELECTOR = '.lingo-shell-scroll, .feed-page__body, .main-content'

const freezeHtml: Ref<string | null> = ref(null)
/** Parallel `scrollTop` values for `SCROLL_RESTORE_SELECTOR` matches at capture time. */
const freezeScrollTops: Ref<number[] | null> = ref(null)

let generation = 0
let quietTimer: ReturnType<typeof setTimeout> | null = null
let fallbackTimer: ReturnType<typeof setTimeout> | null = null
let pendingUnsub: (() => void) | null = null
let armed = false

function clearTimers() {
  if (quietTimer) {
    clearTimeout(quietTimer)
    quietTimer = null
  }
  if (fallbackTimer) {
    clearTimeout(fallbackTimer)
    fallbackTimer = null
  }
}

export function clearRouteOverlay() {
  clearTimers()
  pendingUnsub?.()
  pendingUnsub = null
  armed = false
  freezeHtml.value = null
  freezeScrollTops.value = null
}

function trySettle(gen: number) {
  if (gen !== generation || !armed) return
  if (getApiPendingCount() > 0) return
  quietTimer = setTimeout(() => {
    if (gen !== generation || !armed) return
    if (getApiPendingCount() > 0) return
    clearRouteOverlay()
  }, QUIET_MS)
}

function armSettle(gen: number) {
  clearTimers()
  pendingUnsub?.()
  pendingUnsub = onApiPendingChange(() => {
    if (gen !== generation) return
    if (quietTimer) {
      clearTimeout(quietTimer)
      quietTimer = null
    }
    trySettle(gen)
  })
  // Prefer quiet API, but never block navigation on languages/collections/chat.
  fallbackTimer = setTimeout(() => {
    if (gen === generation) clearRouteOverlay()
  }, MAX_HOLD_MS)
  trySettle(gen)
}

/** Snapshot outgoing route HTML (no-op if a freeze is already showing). */
function captureRouteOverlay(liveRoot: HTMLElement | null | undefined) {
  if (freezeHtml.value != null) return
  if (!liveRoot?.innerHTML.trim()) return
  const scrollers = liveRoot.querySelectorAll(SCROLL_RESTORE_SELECTOR)
  freezeScrollTops.value = Array.from(scrollers, (el) => (el as HTMLElement).scrollTop)
  freezeHtml.value = liveRoot.innerHTML
}

/**
 * After the freeze DOM paints, re-apply captured scrollTops so Lingo trees (etc.)
 * do not jump to the top while the route progress bar runs.
 */
export function restoreRouteOverlayScroll(freezeRoot: HTMLElement | null | undefined) {
  const tops = freezeScrollTops.value
  if (!freezeRoot || !tops?.length) return
  const scrollers = freezeRoot.querySelectorAll(SCROLL_RESTORE_SELECTOR)
  scrollers.forEach((el, i) => {
    const top = tops[i]
    if (typeof top === 'number') (el as HTMLElement).scrollTop = top
  })
}

/** Call from router beforeEach on a real navigation away from a painted view. */
export function beginRouteOverlay(liveRoot: HTMLElement | null | undefined) {
  const hadFreeze = freezeHtml.value != null
  captureRouteOverlay(liveRoot)
  if (freezeHtml.value == null) return
  generation += 1
  armed = true
  // Redirects keep the same snapshot; just refresh settle watchers.
  if (hadFreeze) {
    armSettle(generation)
  }
}

/** Call from router afterEach once the new route component has mounted. */
export function scheduleRouteOverlaySettle() {
  if (freezeHtml.value == null || !armed) return
  armSettle(generation)
}

export function useRouteOverlay() {
  return { freezeHtml, freezeScrollTops }
}
