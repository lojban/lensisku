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

/** Dialogue icons for the study speech-bubble character. */
export const LINGO_STUDY_CHARACTERS = [
  '/lingo/bla5.webp',
  '/lingo/lif7.webp',
  '/lingo/lin5.webp',
] as const

export function pickRandomLingoCharacter(): string {
  const i = Math.floor(Math.random() * LINGO_STUDY_CHARACTERS.length)
  return LINGO_STUDY_CHARACTERS[i]!
}
