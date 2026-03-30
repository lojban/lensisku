/** Two columns: front (prompt) and back (answer), with optional item language. */
type BulkImportRow = {
  free_content_front: string
  free_content_back: string
  language_id?: number | null
}

/** Minimal CSV line parser (quoted fields, doubled quotes). */
function parseCsvLine(line: string): string[] {
  const out: string[] = []
  let cur = ''
  let i = 0
  let inQ = false
  while (i < line.length) {
    const c = line[i]
    if (inQ) {
      if (c === '"') {
        if (line[i + 1] === '"') {
          cur += '"'
          i += 2
          continue
        }
        inQ = false
        i++
        continue
      }
      cur += c
      i++
      continue
    }
    if (c === '"') {
      inQ = true
      i++
      continue
    }
    if (c === ',') {
      out.push(cur)
      cur = ''
      i++
      continue
    }
    cur += c
    i++
  }
  out.push(cur)
  return out
}

function isHeaderRow(cols: string[]): boolean {
  const a = (cols[0] || '').trim().toLowerCase()
  const b = (cols[1] || '').trim().toLowerCase()
  return (a === 'front' && b === 'back') || (a === 'prompt' && b === 'answer')
}

type BulkImportStreamProgress = {
  bytesRead: number
  totalBytes: number
}

type BulkImportParseOptions = {
  frontColumnIndex?: number
  backColumnIndex?: number
  skipFirstRow?: boolean
  languageId?: number | null
}

const PREVIEW_MAX_BYTES = 256 * 1024
const DEFAULT_PREVIEW_MAX_LINES = 15

/**
 * Read only the start of a file and return the first non-empty lines split into cells
 * (same delimiter rules as {@link eachBulkImportRowFromFile}).
 */
export async function previewBulkImportFile(
  file: File,
  maxLines: number = DEFAULT_PREVIEW_MAX_LINES
): Promise<{
  format: 'tsv' | 'csv'
  lines: string[][]
  truncated: boolean
}> {
  const sliceEnd = Math.min(file.size, PREVIEW_MAX_BYTES)
  const chunk = await file.slice(0, sliceEnd).arrayBuffer()
  let text = new TextDecoder('utf-8').decode(chunk)
  text = text.replace(/^\uFEFF/, '')
  const complete =
    file.size <= PREVIEW_MAX_BYTES || /\r\n|\r|\n$/.test(text)
  if (!complete) {
    const lastNl = Math.max(
      text.lastIndexOf('\n'),
      text.lastIndexOf('\r')
    )
    if (lastNl >= 0) {
      text = text.slice(0, lastNl)
    } else {
      text = ''
    }
  }

  const rawLines = text.split(/\r\n|\r|\n/)
  const lines: string[][] = []
  let format: 'tsv' | 'csv' | null = null

  for (const line of rawLines) {
    if (line.trim().length === 0) continue
    if (!format) {
      const tabCount = (line.match(/\t/g) || []).length
      const commaCount = (line.match(/,/g) || []).length
      format =
        /\.tsv$/i.test(file.name) || (tabCount > commaCount && tabCount > 0)
          ? 'tsv'
          : 'csv'
    }
    const cells = format === 'tsv' ? line.split('\t') : parseCsvLine(line)
    lines.push(cells)
    if (lines.length >= maxLines) {
      break
    }
  }

  const truncated =
    file.size > PREVIEW_MAX_BYTES || lines.length >= maxLines
  return {
    format: format ?? (/\.tsv$/i.test(file.name) ? 'tsv' : 'csv'),
    lines,
    truncated,
  }
}

function stripLeadingBom(line: string): string {
  return line.replace(/^\uFEFF/, '')
}

/**
 * Reads the file as a stream (no single huge `file.text()` string) and yields one row per
 * non-empty line (no embedded newlines in fields), same delimiter rules as {@link previewBulkImportFile}.
 */
export async function* eachBulkImportRowFromFile(
  file: File,
  onReadProgress?: (p: BulkImportStreamProgress) => void,
  options: BulkImportParseOptions = {}
): AsyncGenerator<BulkImportRow, void, undefined> {
  let bytesRead = 0
  const totalBytes = file.size
  const reader = file.stream().getReader()
  const decoder = new TextDecoder('utf-8')
  let buffer = ''
  let bomStripped = false
  let format: 'tsv' | 'csv' | null = null
  let firstNonEmptyHandled = false
  const frontColumnIndex = Math.max(0, options.frontColumnIndex ?? 0)
  const backColumnIndex = Math.max(0, options.backColumnIndex ?? 1)

  const emitProgress = () => {
    onReadProgress?.({ bytesRead, totalBytes })
  }

  const rowFromDataLine = (line: string): BulkImportRow => {
    const cells = format === 'tsv' ? line.split('\t') : parseCsvLine(line)
    const front = (cells[frontColumnIndex] ?? '').trim()
    const back = (cells[backColumnIndex] ?? '').trim()
    return {
      free_content_front: front,
      free_content_back: back,
      language_id: options.languageId,
    }
  }

  function* handlePhysicalLine(line: string): Generator<BulkImportRow> {
    if (!bomStripped) {
      line = stripLeadingBom(line)
      bomStripped = true
    }
    if (line.trim().length === 0) {
      return
    }
    if (!firstNonEmptyHandled) {
      const tabCount = (line.match(/\t/g) || []).length
      const commaCount = (line.match(/,/g) || []).length
      const useTsv =
        /\.tsv$/i.test(file.name) || (tabCount > commaCount && tabCount > 0)
      format = useTsv ? 'tsv' : 'csv'
      const probeCols = format === 'tsv' ? line.split('\t') : parseCsvLine(line)
      firstNonEmptyHandled = true
      if (options.skipFirstRow) {
        return
      }
      if (probeCols.length >= 2 && isHeaderRow(probeCols)) {
        return
      }
    }
    yield rowFromDataLine(line)
  }

  try {
    while (true) {
      const { done, value } = await reader.read()
      if (value) {
        bytesRead += value.byteLength
        emitProgress()
        buffer += decoder.decode(value, { stream: true })
      }
      if (done) {
        buffer += decoder.decode()
        emitProgress()
      }

      while (true) {
        const match = buffer.match(/\r\n|\r|\n/)
        if (!match || match.index === undefined) {
          break
        }
        const line = buffer.slice(0, match.index)
        buffer = buffer.slice(match.index + match[0].length)
        yield* handlePhysicalLine(line)
      }

      if (done) {
        break
      }
    }

    if (buffer.length > 0) {
      const line = buffer.replace(/\r$/, '')
      yield* handlePhysicalLine(line)
    }
  } finally {
    reader.releaseLock()
  }
}

export type BulkTableRow = {
  item_id: number
  position: number
  free_content_front: string
  free_content_back: string
  language_id: number | null
}

export type BulkDraftRow = {
  id: string
  free_content_front: string
  free_content_back: string
  language_id: number | null
}

export type BulkMergeStats = {
  replacedByFront: number
  replacedByBack: number
  inserted: number
  skippedEmpty: number
}

/** Remove trailing blank drafts so new imports are not pushed after a spacer empty row. */
export function stripTrailingEmptyDrafts(newRows: BulkDraftRow[]): BulkDraftRow[] {
  const out = [...newRows]
  while (out.length > 0) {
    const last = out[out.length - 1]
    const empty =
      !last.free_content_front.trim() && !last.free_content_back.trim()
    if (!empty) break
    out.pop()
  }
  return out
}

export function mergeOneBulkImportRow(
  imp: BulkImportRow,
  rows: BulkTableRow[],
  newRows: BulkDraftRow[],
  createDraftWithId: () => BulkDraftRow,
  stats: BulkMergeStats
): void {
  const f = imp.free_content_front.trim()
  const b = imp.free_content_back.trim()
  if (!f && !b) {
    stats.skippedEmpty += 1
    return
  }

  const idxSavedFront = rows.findIndex((r) => r.free_content_front.trim() === f)
  if (idxSavedFront !== -1) {
    rows[idxSavedFront].free_content_front = imp.free_content_front
    rows[idxSavedFront].free_content_back = imp.free_content_back
    if (imp.language_id !== undefined) {
      rows[idxSavedFront].language_id = imp.language_id
    }
    stats.replacedByFront += 1
    return
  }

  const idxSavedBack = rows.findIndex((r) => r.free_content_back.trim() === b)
  if (idxSavedBack !== -1) {
    rows[idxSavedBack].free_content_front = imp.free_content_front
    rows[idxSavedBack].free_content_back = imp.free_content_back
    if (imp.language_id !== undefined) {
      rows[idxSavedBack].language_id = imp.language_id
    }
    stats.replacedByBack += 1
    return
  }

  const idxDraftFront = newRows.findIndex((d) => d.free_content_front.trim() === f)
  if (idxDraftFront !== -1) {
    newRows[idxDraftFront].free_content_front = imp.free_content_front
    newRows[idxDraftFront].free_content_back = imp.free_content_back
    if (imp.language_id !== undefined) {
      newRows[idxDraftFront].language_id = imp.language_id
    }
    stats.replacedByFront += 1
    return
  }

  const idxDraftBack = newRows.findIndex((d) => d.free_content_back.trim() === b)
  if (idxDraftBack !== -1) {
    newRows[idxDraftBack].free_content_front = imp.free_content_front
    newRows[idxDraftBack].free_content_back = imp.free_content_back
    if (imp.language_id !== undefined) {
      newRows[idxDraftBack].language_id = imp.language_id
    }
    stats.replacedByBack += 1
    return
  }

  const draft = createDraftWithId()
  draft.free_content_front = imp.free_content_front
  draft.free_content_back = imp.free_content_back
  draft.language_id = imp.language_id ?? null
  newRows.push(draft)
  stats.inserted += 1
}

export function finalizeBulkImportDraftRows(
  newRows: BulkDraftRow[],
  createDraftWithId: () => BulkDraftRow
): BulkDraftRow[] {
  if (newRows.length === 0) {
    return [createDraftWithId()]
  }
  const last = newRows[newRows.length - 1]
  const lastEmpty =
    !last.free_content_front.trim() && !last.free_content_back.trim()
  if (!lastEmpty) {
    return [...newRows, createDraftWithId()]
  }
  return newRows
}
