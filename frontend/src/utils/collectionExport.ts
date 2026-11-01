/**
 * Full collection export/import file (`lensisku.collection`, format_version 2).
 * Produced by the JSON "full collection" export and accepted by `POST /collections/import/full`.
 * Version 1 files (no `format_version`) are still accepted; all v2 fields are optional.
 */
export const COLLECTION_EXPORT_FORMAT = 'lensisku.collection'
export const COLLECTION_EXPORT_FORMAT_VERSION = 2

export interface DialogueLineSoundExport {
  line_index: number
  text_hash: string
  voice: string
  /** `data:audio/...;base64,...` */
  sound_url: string
}

/** Dialogue extras; the script itself (lines, `~` glosses, every scored `?` question) is `free_content_front`. */
export interface DialogueExport {
  voices?: Record<string, string> | null
  line_sounds?: DialogueLineSoundExport[]
  /** Parsed, read-only view of the script; ignored on import. */
  structure?: unknown
}

export interface CollectionExportItem {
  item_id?: number
  position?: number
  collection_note?: string | null
  definition_id?: number | null
  language_id?: number | null
  owner_user_id?: number | null
  license?: string | null
  word?: string | null
  word_type?: string | null
  definition?: string | null
  free_content_front?: string | null
  free_content_back?: string | null
  front_image_url?: string | null
  back_image_url?: string | null
  sound_url?: string | null
  direction?: string | null
  level_index?: number | null
  position_in_level?: number | null
  auto_progress?: boolean | null
  script_code?: string | null
  is_original?: boolean | null
  canonical_form?: string | null
  use_canonical_comparison?: boolean | null
  quiz_correct_answer?: string | null
  dialogue?: DialogueExport | null
  [key: string]: unknown
}

export interface LevelExport {
  name: string
  description?: string | null
  min_cards: number
  min_success_rate: number
  position: number
  prerequisite_positions?: number[]
  item_positions?: number[]
}

export interface CollectionFullExportFile {
  format?: string
  format_version?: number
  collection: {
    name: string
    description?: string | null
    is_public?: boolean | null
    challenge_mode?: 'off' | 'fixed' | 'auto' | null
    cover_image_url?: string | null
  }
  items: CollectionExportItem[]
  levels?: LevelExport[]
}

/** Validates a parsed file and returns the import payload, passing every field through. */
export function toImportFullPayload(data: unknown): CollectionFullExportFile | null {
  if (!data || typeof data !== 'object') return null
  const d = data as Partial<CollectionFullExportFile>
  if (!d.collection || typeof d.collection !== 'object' || !Array.isArray(d.items)) return null
  if (d.format && d.format !== COLLECTION_EXPORT_FORMAT) return null
  return {
    ...(d.format ? { format: d.format } : {}),
    ...(d.format_version != null ? { format_version: d.format_version } : {}),
    collection: {
      ...d.collection,
      description: d.collection.description ?? null,
      is_public: d.collection.is_public ?? true,
    },
    items: d.items,
    levels: Array.isArray(d.levels) ? d.levels : [],
  }
}
