/** Chip used in Duolingo-style word-bank assemble challenges (text from server). */
export interface WordbankChip {
  id: string
  text: string
}

function normalizeDirection(dir: string): string {
  return dir.toLowerCase().replace(/_/g, '')
}

/**
 * Expected typed/assembled answer for fill-in, wordbank, and listen-wordbank.
 * Wordbank/fill-in: direct → back; reverse → front.
 * Listen-tap: direct → front (hear & tap); reverse → back (hear & translate).
 *
 * Chip banks arrive on the flashcard (`wordbank_chips` / `wordbank_expected_normalized`).
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
