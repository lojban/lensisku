/**
 * Content-type glyphs (brand book §5c).
 *
 * Each concept has three variants. Import them from `@packages/ui`.
 *
 * - **One**: the single-item glyph (waveform, quoted gloss, letter W, star).
 * - **Several**: the list glyph (Lucide waves, Lucide list, the wiki book, a star with its left half).
 * - **Create**: that concept’s one-glyph plus a plus (`StarPlus` corner).
 *
 * Custom SVG components (W, wiki book, star contour, create glyphs) live in this folder
 * and are re-exported from `packages/ui/index.ts`.
 */
export { BookA as EntryIcon } from '@lucide/vue'
export { List as DefinitionsIcon, TextQuote as DefinitionIcon } from '@lucide/vue'
export { AudioWaveform as WaveIcon, WavesHorizontal as WavesIcon } from '@lucide/vue'
export { Star as StarIcon, StarPlus as StarCreateIcon } from '@lucide/vue'
