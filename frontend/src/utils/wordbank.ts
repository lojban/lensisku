/** Chip used in Duolingo-style word-bank assemble challenges. */
export interface WordbankChip {
  id: string
  text: string
}

/** Prefer the first `;`-separated alternative for chip banks. */
export function primaryExpectedAnswer(expected: string): string {
  if (!expected.includes(';')) return expected.trim()
  const first = expected
    .split(';')
    .map((s) => s.trim())
    .find(Boolean)
  return first ?? expected.trim()
}

/**
 * Strip sentence punctuation from a chip token and lowercase it.
 * Keeps letters/numbers, apostrophe (Lojban denpa bu / EN contractions), and hyphen.
 * Hides periods/commas/caps that would otherwise reconstruct the sentence.
 */
export function normalizeWordbankToken(token: string): string {
  return token
    .replace(/[^\p{L}\p{N}'\u2019-]+/gu, '')
    .normalize('NFC')
    .toLocaleLowerCase()
}

/** Full phrase → normalized chip tokens (punct gone, no sentence-initial caps). */
export function tokenizeWordbankAnswer(text: string): string[] {
  return primaryExpectedAnswer(text)
    .split(/\s+/)
    .map((t) => normalizeWordbankToken(t.trim()))
    .filter(Boolean)
}

/** Same rules as chips, joined — for grading wordbank / listen-wordbank answers. */
export function normalizeWordbankPhrase(text: string): string {
  return tokenizeWordbankAnswer(text).join(' ')
}

function normalizeDirection(dir: string): string {
  return dir.toLowerCase().replace(/_/g, '')
}

/**
 * Expected typed/assembled answer for fill-in, wordbank, and listen-wordbank.
 * Wordbank/fill-in: direct → back; reverse → front.
 * Listen-tap: direct → front (hear & tap); reverse → back (hear & translate).
 */
export function expectedFillinLikeAnswer(
  fc: {
    definition?: string | null
    free_content_back?: string | null
    word?: string | null
    free_content_front?: string | null
    direction?: string | null
  },
  cardSide: string
): string {
  const d = normalizeDirection(fc.direction ?? '')
  const isListen = d.startsWith('listenwordbank')
  if (isListen) {
    if (cardSide === 'direct' && (d === 'listenwordbank' || d === 'listenwordbankboth')) {
      return (fc.word ?? fc.free_content_front ?? '').trim()
    }
    if (cardSide === 'reverse' && (d === 'listenwordbankreverse' || d === 'listenwordbankboth')) {
      return (fc.definition ?? fc.free_content_back ?? '').trim()
    }
    return ''
  }
  const isDirectFamily =
    d === 'fillin' || d === 'fillinboth' || d === 'wordbank' || d === 'wordbankboth'
  const isReverseFamily =
    d === 'fillinreverse' || d === 'fillinboth' || d === 'wordbankreverse' || d === 'wordbankboth'
  if (cardSide === 'direct' && isDirectFamily) {
    return (fc.definition ?? fc.free_content_back ?? '').trim()
  }
  if (cardSide === 'reverse' && isReverseFamily) {
    return (fc.word ?? fc.free_content_front ?? '').trim()
  }
  return ''
}

function shuffleInPlace<T>(arr: T[]): T[] {
  for (let i = arr.length - 1; i > 0; i -= 1) {
    const j = Math.floor(Math.random() * (i + 1))
    const tmp = arr[i]
    arr[i] = arr[j]
    arr[j] = tmp
  }
  return arr
}

/**
 * Build a shuffled chip bank: correct tokens plus distractors from other card texts.
 */
export function buildWordbankChips(
  expected: string,
  distractorSources: string[],
  opts?: { maxExtra?: number }
): WordbankChip[] {
  const correctTokens = tokenizeWordbankAnswer(expected)
  if (correctTokens.length === 0) return []

  const correctSet = new Set(correctTokens.map((t) => t.toLowerCase()))
  const extras: string[] = []
  for (const src of distractorSources) {
    for (const tok of tokenizeWordbankAnswer(src)) {
      const key = tok.toLowerCase()
      if (correctSet.has(key)) continue
      if (extras.some((e) => e.toLowerCase() === key)) continue
      extras.push(tok)
    }
  }
  shuffleInPlace(extras)
  const maxExtra = opts?.maxExtra ?? Math.max(3, Math.min(6, correctTokens.length + 2))
  const picked = extras.slice(0, maxExtra)

  const chips: WordbankChip[] = []
  let i = 0
  for (const text of correctTokens) {
    chips.push({ id: `wb-c-${i}-${text}`, text })
    i += 1
  }
  for (const text of picked) {
    chips.push({ id: `wb-d-${i}-${text}`, text })
    i += 1
  }
  return shuffleInPlace(chips)
}

/** Collect plain text from a flashcard for distractor mining. */
export function flashcardDistractorTexts(fc: {
  word?: string | null
  definition?: string | null
  free_content_front?: string | null
  free_content_back?: string | null
}): string[] {
  return [fc.word, fc.definition, fc.free_content_front, fc.free_content_back].filter(
    (s): s is string => typeof s === 'string' && s.trim().length > 0
  )
}
