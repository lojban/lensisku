import { ref, type Ref } from 'vue'
import { onApiPendingChange, getApiPendingCount } from '@/api'

/**
 * Path-change overlay only (not query/hash): freeze the outgoing view, show a spinner,
 * reveal the next view when in-flight API requests go quiet (no per-page hooks).
 * In-page loads (filters, tabs) use a scoped spinner on the changing region instead.
 */

/** API must stay idle this long; bridges gaps between chained page fetches (await A; then B). */
const QUIET_MS = 200
/**
 * Hard cap on the freeze (slow or failing API). Was 900 ms, which revealed the next page
 * mid-load on anything slower than a warm cache.
 */
const MAX_HOLD_MS = 8000

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

const nextFrames = () =>
  new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r())))

/**
 * Reveal the next route once it is actually ready: its first fetch burst went quiet, page-level
 * reveal holds (holdRouteReveal) are released, and any fetches those triggered went quiet too.
 * Capped by MAX_HOLD_MS.
 */
function armSettle(gen: number) {
  clearTimers()
  pendingUnsub?.()
  pendingUnsub = null
  fallbackTimer = setTimeout(() => {
    if (gen === generation) clearRouteOverlay()
  }, MAX_HOLD_MS)
  void (async () => {
    // Let onMounted handlers start their requests before measuring "quiet".
    await nextFrames()
    await waitForApiQuiet(MAX_HOLD_MS)
    await waitForRevealHolds(MAX_HOLD_MS)
    await waitForApiQuiet(MAX_HOLD_MS)
    await nextFrames()
    if (gen === generation && armed) clearRouteOverlay()
  })()
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

/** Resolve once in-flight API requests have been quiet for QUIET_MS (or after `maxMs`). */
export function waitForApiQuiet(maxMs = 1500): Promise<void> {
  return new Promise((resolve) => {
    let done = false
    let timer: ReturnType<typeof setTimeout> | null = null
    const finish = () => {
      if (done) return
      done = true
      if (timer) clearTimeout(timer)
      unsub()
      clearTimeout(hard)
      resolve()
    }
    const check = () => {
      if (timer) clearTimeout(timer)
      timer = null
      if (getApiPendingCount() === 0) timer = setTimeout(finish, QUIET_MS)
    }
    const unsub = onApiPendingChange(check)
    const hard = setTimeout(finish, maxMs)
    check()
  })
}

/**
 * Page-level reveal holds: a page whose first paint depends on more than the first API burst
 * (e.g. Lingo study builds a session in several steps) holds the full-viewport layout/boot
 * overlay until its content is ready. Always call the returned release (idempotent).
 */
const revealHolds = ref(0)

export function holdRouteReveal(): () => void {
  revealHolds.value += 1
  let released = false
  return () => {
    if (released) return
    released = true
    revealHolds.value -= 1
  }
}

export function waitForRevealHolds(maxMs = 8000): Promise<void> {
  if (revealHolds.value <= 0) return Promise.resolve()
  return new Promise((resolve) => {
    const started = Date.now()
    const tick = () => {
      if (revealHolds.value <= 0 || Date.now() - started > maxMs) resolve()
      else setTimeout(tick, 50)
    }
    tick()
  })
}

/** Resolve when images currently in `root` have loaded (or failed), capped at `maxMs`. */
export function waitForImages(root: ParentNode = document, maxMs = 1500): Promise<void> {
  const imgs = Array.from(root.querySelectorAll('img')).filter(
    (img) => !img.complete && img.loading !== 'lazy'
  )
  if (!imgs.length) return Promise.resolve()
  return Promise.race([
    Promise.all(
      imgs.map(
        (img) =>
          new Promise<void>((r) => {
            img.addEventListener('load', () => r(), { once: true })
            img.addEventListener('error', () => r(), { once: true })
          })
      )
    ).then(() => undefined),
    new Promise<void>((r) => setTimeout(r, maxMs)),
  ])
}
