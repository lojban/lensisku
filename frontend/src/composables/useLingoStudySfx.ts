/**
 * Lingo lesson feedback SFX (same flow as archive/duolingo-clone lesson quiz).
 * Assets in `public/lingo/*.ogg` — see ATTRIBUTION.md.
 *
 * Prefetches + decodes into AudioBuffers so Forgot / Good / Easy feel immediate.
 */

const SFX = {
  correct: '/lingo/correct.ogg',
  incorrect: '/lingo/incorrect.ogg',
  finish: '/lingo/finish.ogg',
} as const

type SfxKind = keyof typeof SFX

let audioCtx: AudioContext | null = null
const buffers = new Map<SfxKind, AudioBuffer>()
/** HTMLAudio fallback while decode is in flight or Web Audio unavailable. */
const htmlPlayers = new Map<SfxKind, HTMLAudioElement>()
let preloadPromise: Promise<void> | null = null

function getAudioContext(): AudioContext | null {
  if (typeof window === 'undefined') return null
  if (!audioCtx) {
    const AC =
      window.AudioContext ||
      (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext
    if (!AC) return null
    audioCtx = new AC()
  }
  return audioCtx
}

function getHtmlPlayer(kind: SfxKind): HTMLAudioElement {
  let el = htmlPlayers.get(kind)
  if (!el) {
    el = new Audio(SFX[kind])
    el.preload = 'auto'
    el.load()
    htmlPlayers.set(kind, el)
  }
  return el
}

function playHtml(kind: SfxKind) {
  try {
    const el = getHtmlPlayer(kind)
    el.currentTime = 0
    void el.play().catch(() => {
      /* autoplay / missing user gesture — ignore */
    })
  } catch {
    /* ignore */
  }
}

async function resumeCtx(ctx: AudioContext) {
  if (ctx.state === 'suspended') {
    try {
      await ctx.resume()
    } catch {
      /* ignore */
    }
  }
}

/**
 * Fetch + decode all lesson SFX (safe to call multiple times).
 * Starts network work immediately; AudioContext may stay suspended until a gesture.
 */
export function preloadLingoStudySfx(): Promise<void> {
  if (preloadPromise) return preloadPromise
  preloadPromise = (async () => {
    // Warm HTMLAudio cache in parallel (fallback path).
    for (const kind of Object.keys(SFX) as SfxKind[]) {
      getHtmlPlayer(kind)
    }

    const ctx = getAudioContext()
    if (!ctx) return

    await Promise.all(
      (Object.keys(SFX) as SfxKind[]).map(async (kind) => {
        if (buffers.has(kind)) return
        try {
          const res = await fetch(SFX[kind])
          if (!res.ok) return
          const raw = await res.arrayBuffer()
          // decodeAudioData may detach the buffer; copy for safety across browsers.
          const decoded = await ctx.decodeAudioData(raw.slice(0))
          buffers.set(kind, decoded)
        } catch {
          /* keep HTMLAudio fallback for this kind */
        }
      })
    )
  })()
  return preloadPromise
}

function playBuffer(ctx: AudioContext, buf: AudioBuffer) {
  const src = ctx.createBufferSource()
  src.buffer = buf
  src.connect(ctx.destination)
  src.start(0)
}

function play(kind: SfxKind) {
  const ctx = getAudioContext()
  const buf = buffers.get(kind)
  if (ctx && buf) {
    // Prefer sync start when already unlocked (rating taps after Check).
    if (ctx.state === 'running') {
      try {
        playBuffer(ctx, buf)
        return
      } catch {
        playHtml(kind)
        return
      }
    }
    void resumeCtx(ctx).then(() => {
      try {
        playBuffer(ctx, buf)
      } catch {
        playHtml(kind)
      }
    })
    return
  }
  playHtml(kind)
}

/** Resume AudioContext on a user gesture so the next rating SFX can start synchronously. */
export async function unlockLingoStudySfx(): Promise<void> {
  await preloadLingoStudySfx()
  const ctx = getAudioContext()
  if (ctx) await resumeCtx(ctx)
}

export function useLingoStudySfx() {
  void preloadLingoStudySfx()
  return {
    playCorrect: () => play('correct'),
    playIncorrect: () => play('incorrect'),
    playFinish: () => play('finish'),
    preload: preloadLingoStudySfx,
    unlock: unlockLingoStudySfx,
  }
}

/** Question-row sprites — only the `*1` poses. */
export const LINGO_STUDY_QUESTION_CHARACTERS = [
  '/lingo/lin1.webp',
  '/lingo/lif1.webp',
  '/lingo/bla1.webp',
] as const

/** Reply-row sprites — the three poses used before the full set was copied. */
export const LINGO_STUDY_ANSWER_CHARACTERS = [
  '/lingo/bla5.webp',
  '/lingo/lif7.webp',
  '/lingo/lin5.webp',
] as const

/** @deprecated Prefer question/answer pools; kept for any single-sprite callers. */
export const LINGO_STUDY_CHARACTERS = LINGO_STUDY_ANSWER_CHARACTERS

function pickFrom(pool: readonly string[]): string {
  return pool[Math.floor(Math.random() * pool.length)]!
}

export function pickRandomLingoCharacter(): string {
  return pickFrom(LINGO_STUDY_ANSWER_CHARACTERS)
}

/** Same character family, pose `1` (e.g. `/lingo/lin5.webp` → `/lingo/lin1.webp`). */
export function lingoStudyPose1Src(src: string): string {
  return src.replace(
    /(\/[a-z]+)\d+(\.webp)$/i,
    (_m, prefix: string, ext: string) => `${prefix}1${ext}`
  )
}

/**
 * Question uses a `*1` sprite; reply uses one of the three answer sprites.
 * Prefixes differ when possible so the two speakers are not the same family.
 */
export function pickLingoStudyCharacterPair(): { question: string; answer: string } {
  const question = pickFrom(LINGO_STUDY_QUESTION_CHARACTERS)
  const qPrefix = question.match(/\/([a-z]+)\d+\.webp$/)?.[1]
  const answerPool = LINGO_STUDY_ANSWER_CHARACTERS.filter((src) => {
    const aPrefix = src.match(/\/([a-z]+)\d+\.webp$/)?.[1]
    return aPrefix !== qPrefix
  })
  const answer = pickFrom(answerPool.length ? answerPool : LINGO_STUDY_ANSWER_CHARACTERS)
  return { question, answer }
}

/** FNV-1a 32-bit → base36 fragment id for a study question (stable deep-link hash). */
export function hashLingoQuestionKey(text: string): string {
  let h = 2166136261
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i)
    h = Math.imul(h, 16777619)
  }
  return (h >>> 0).toString(36)
}
