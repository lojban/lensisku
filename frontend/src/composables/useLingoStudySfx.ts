/**
 * Lingo lesson feedback SFX (same flow as archive/duolingo-clone lesson quiz).
 * Assets in `public/lingo/*.ogg` — see ATTRIBUTION.md.
 */

const SFX = {
  correct: '/lingo/correct.ogg',
  incorrect: '/lingo/incorrect.ogg',
  finish: '/lingo/finish.ogg',
} as const

type SfxKind = keyof typeof SFX

const players = new Map<SfxKind, HTMLAudioElement>()

function getPlayer(kind: SfxKind): HTMLAudioElement {
  let el = players.get(kind)
  if (!el) {
    el = new Audio(SFX[kind])
    el.preload = 'auto'
    players.set(kind, el)
  }
  return el
}

function play(kind: SfxKind) {
  try {
    const el = getPlayer(kind)
    el.currentTime = 0
    void el.play().catch(() => {
      /* autoplay / missing user gesture — ignore */
    })
  } catch {
    /* ignore */
  }
}

export function useLingoStudySfx() {
  return {
    playCorrect: () => play('correct'),
    playIncorrect: () => play('incorrect'),
    playFinish: () => play('finish'),
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
