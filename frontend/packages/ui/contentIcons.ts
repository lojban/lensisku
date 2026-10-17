/**
 * Content-type glyphs (brand book §5c).
 *
 * Import these from `@packages/ui`. The same concept uses the same glyph everywhere
 * (page headers, chips, tabs, and create actions). Wiki stays `WikiIcon` (book + W).
 *
 * - **Entry** (valsi / dictionary headword): open book.
 * - **Definition** (one sense): closed book. **Create definition** adds a plus.
 * - **Wave** (discuss / new discussion): audio waveform — not Lucide `Waves`.
 */
export {
  BookOpen as EntryIcon,
  Book as DefinitionIcon,
  BookPlus as DefinitionCreateIcon,
} from '@lucide/vue'
export { AudioWaveform as WaveIcon } from '@lucide/vue'
