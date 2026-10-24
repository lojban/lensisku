/**
 * One shared HTMLAudioElement for all `AudioPlayer` UIs.
 * A new play request fades out whatever is current, then starts the next clip
 * (single-slot pending queue — latest wins while a fade/switch is in flight).
 */

export type SharedAudioPlayRequest = {
  /** Stable id for the requesting AudioPlayer instance. */
  ownerId: string
  /** Blob or http(s) URL for `HTMLAudioElement.src`. */
  src: string
  playbackRate?: number
  /** Fired when this owner starts/stops being the active clip. */
  onActiveChange?: (active: boolean) => void
}

const FADE_MS = 140
const DEFAULT_VOLUME = 1

let audioEl: HTMLAudioElement | null = null
let activeOwnerId: string | null = null
let activeOnChange: ((active: boolean) => void) | null = null
let fadeToken = 0
let switching = false
/** Mini-queue: at most one pending play (newest replaces older). */
let pending: SharedAudioPlayRequest | null = null

const ownerHandlers = new Map<string, (active: boolean) => void>()

function ensureAudio(): HTMLAudioElement {
  if (!audioEl) {
    audioEl = new Audio()
    audioEl.preload = 'auto'
    audioEl.addEventListener('ended', () => {
      notifyActive(false)
      activeOwnerId = null
      activeOnChange = null
    })
  }
  return audioEl
}

function notifyActive(active: boolean) {
  if (activeOnChange) {
    try {
      activeOnChange(active)
    } catch {
      /* ignore UI handler errors */
    }
  }
}

function cancelFade() {
  fadeToken += 1
}

function fadeOut(el: HTMLAudioElement, ms: number): Promise<void> {
  cancelFade()
  const token = fadeToken
  const startVol = el.volume > 0 ? el.volume : DEFAULT_VOLUME
  if (el.paused || el.ended) {
    el.volume = DEFAULT_VOLUME
    return Promise.resolve()
  }
  const start = performance.now()
  return new Promise((resolve) => {
    const step = (now: number) => {
      if (token !== fadeToken) {
        resolve()
        return
      }
      const t = Math.min(1, (now - start) / ms)
      el.volume = startVol * (1 - t)
      if (t < 1) {
        requestAnimationFrame(step)
      } else {
        try {
          el.pause()
        } catch {
          /* ignore */
        }
        el.volume = DEFAULT_VOLUME
        resolve()
      }
    }
    requestAnimationFrame(step)
  })
}

async function startClip(req: SharedAudioPlayRequest): Promise<void> {
  const el = ensureAudio()
  if (activeOwnerId && activeOwnerId !== req.ownerId) {
    notifyActive(false)
  }
  activeOwnerId = req.ownerId
  activeOnChange = req.onActiveChange ?? ownerHandlers.get(req.ownerId) ?? null

  const rate =
    Number.isFinite(req.playbackRate) && (req.playbackRate as number) > 0
      ? (req.playbackRate as number)
      : 1

  cancelFade()
  el.volume = DEFAULT_VOLUME
  if (el.src !== req.src) {
    el.src = req.src
  }
  el.playbackRate = rate
  el.currentTime = 0
  notifyActive(true)
  try {
    await el.play()
  } catch (e) {
    notifyActive(false)
    if (activeOwnerId === req.ownerId) {
      activeOwnerId = null
      activeOnChange = null
    }
    throw e
  }
}

async function drainQueue(): Promise<void> {
  if (switching) return
  switching = true
  try {
    while (pending) {
      const req = pending
      pending = null
      const el = ensureAudio()
      const needsFade = !el.paused && !el.ended
      if (needsFade) {
        await fadeOut(el, FADE_MS)
      }
      // A newer request may have arrived during fade — prefer it.
      if (pending) continue
      await startClip(req)
    }
  } finally {
    switching = false
    if (pending) void drainQueue()
  }
}

/** Register UI active callback for an owner (used when play opts omit onActiveChange). */
export function registerSharedAudioOwner(
  ownerId: string,
  onActiveChange: (active: boolean) => void
): () => void {
  ownerHandlers.set(ownerId, onActiveChange)
  return () => {
    if (ownerHandlers.get(ownerId) === onActiveChange) {
      ownerHandlers.delete(ownerId)
    }
    if (activeOwnerId === ownerId) {
      void stopSharedAudio(ownerId)
    }
  }
}

/** Enqueue a play; fades out the current clip first. Latest pending wins. */
export function playSharedAudio(req: SharedAudioPlayRequest): Promise<void> {
  if (req.onActiveChange) {
    ownerHandlers.set(req.ownerId, req.onActiveChange)
  }
  pending = req
  return drainQueue()
}

/** Pause only if this owner currently owns the shared element. */
export function pauseSharedAudio(ownerId: string): void {
  if (activeOwnerId !== ownerId) return
  const el = audioEl
  if (!el) return
  cancelFade()
  try {
    el.pause()
  } catch {
    /* ignore */
  }
  el.volume = DEFAULT_VOLUME
  notifyActive(false)
}

/** Fade out and clear ownership (e.g. component unmount). */
export async function stopSharedAudio(ownerId: string): Promise<void> {
  if (pending?.ownerId === ownerId) {
    pending = null
  }
  if (activeOwnerId !== ownerId) return
  const el = audioEl
  if (el) {
    await fadeOut(el, FADE_MS / 2)
  }
  notifyActive(false)
  activeOwnerId = null
  activeOnChange = null
}

export function isSharedAudioActive(ownerId: string): boolean {
  return activeOwnerId === ownerId && !!audioEl && !audioEl.paused && !audioEl.ended
}

export function useSharedAudio() {
  return {
    play: playSharedAudio,
    pause: pauseSharedAudio,
    stop: stopSharedAudio,
    register: registerSharedAudioOwner,
    isActive: isSharedAudioActive,
  }
}
