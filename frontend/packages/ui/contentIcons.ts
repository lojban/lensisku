/**
 * Content-type glyphs (brand book §5c).
 *
 * Import these from `@packages/ui`. The same concept uses the same glyph everywhere
 * (page headers, chips, tabs, and create actions). Wiki stays `WikiIcon` (book + W).
 *
 * - **Entry** (valsi / dictionary headword): book with the letter A (`BookA`).
 * - **Definition** (one sense / gloss): quoted text (`TextQuote`).
 *   **Create definition** is `DefinitionCreateIcon` (same quote plus a plus).
 * - **Wave** (discuss / new discussion): audio waveform — not Lucide `Waves`.
 */
export { BookA as EntryIcon, TextQuote as DefinitionIcon } from '@lucide/vue'
export { AudioWaveform as WaveIcon } from '@lucide/vue'
