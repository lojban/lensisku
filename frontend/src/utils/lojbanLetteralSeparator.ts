/** Lojban consonant letters usable as zoi-style letteral delimiters. */
const CONSONANTS = [...'bcdfgjklmnprstvxz'] as const

/** Build a letteral separator from one or more consonants, e.g. `b` → `by.`, `bc` → `by.cy.`. */
function toLetteral(chars: readonly string[]): string {
  return chars.map((c) => `${c}y.`).join('')
}

/** Lexicographic sequences of `len` consonants (base-|CONSONANTS| odometer). */
function* consonantSequences(len: number): Generator<string[]> {
  const n = CONSONANTS.length
  const idx = Array.from({ length: len }, () => 0)
  while (true) {
    yield idx.map((i) => CONSONANTS[i])
    let p = len - 1
    while (p >= 0) {
      idx[p] += 1
      if (idx[p] < n) break
      idx[p] = 0
      p -= 1
    }
    if (p < 0) return
  }
}

/**
 * Pick a Lojban letteral separator that does not collide with `query`.
 *
 * Prefers single letterals (`by.`, `cy.`, …) whose consonant does not appear
 * in the query; if every consonant appears, escalates to sequences
 * (`by.by.`, `by.cy.`, …) and accepts the first whose full letteral string
 * is not a substring of the query.
 */
export function pickLojbanLetteralSeparator(query: string): string {
  const q = query.toLowerCase()
  const free = CONSONANTS.filter((c) => !q.includes(c))

  if (free.length > 0) {
    return toLetteral([free[0]])
  }

  // Every consonant appears in the query — require the full delimiter token
  // to be absent as a substring (classic zoi delimiter rule).
  for (let len = 2; len <= 8; len++) {
    for (const chars of consonantSequences(len)) {
      const sep = toLetteral(chars)
      if (!q.includes(sep)) {
        return sep
      }
    }
  }

  // Pathological fallback (query contains every short letteral sequence).
  return toLetteral(Array.from({ length: 9 }, () => 'b'))
}
