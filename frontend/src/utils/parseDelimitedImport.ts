/** Two columns: front (prompt) and back (answer). */

type BulkImportRow = { free_content_front: string; free_content_back: string }

/** Minimal CSV line parser (quoted fields, doubled quotes). */
export function parseCsvLine(line: string): string[] {
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

export function isHeaderRow(cols: string[]): boolean {
  const a = (cols[0] || '').trim().toLowerCase()
  const b = (cols[1] || '').trim().toLowerCase()
  return (a === 'front' && b === 'back') || (a === 'prompt' && b === 'answer')
}

export function parseCsvOrTsvFile(
  text: string,
  filename?: string
): { rows: BulkImportRow[]; warnings: string[] } {
  const warnings: string[] = []
  const raw = text.replace(/^\uFEFF/, '')
  const lines = raw.split(/\r?\n/).map((l) => l.replace(/\r$/, ''))
  const nonEmpty = lines.filter((l) => l.trim().length > 0)
  if (nonEmpty.length === 0) {
    warnings.push('empty')
    return { rows: [], warnings }
  }

  const first = nonEmpty[0]
  const tabCount = (first.match(/\t/g) || []).length
  const commaCount = (first.match(/,/g) || []).length
  const useTsv = /\.tsv$/i.test(filename || '') || (tabCount > commaCount && tabCount > 0)

  const rows: BulkImportRow[] = []
  let start = 0
  const probeCols = useTsv ? first.split('\t') : parseCsvLine(first)
  if (probeCols.length >= 2 && isHeaderRow(probeCols)) {
    start = 1
  }

  for (let li = start; li < nonEmpty.length; li++) {
    const line = nonEmpty[li]
    const cells = useTsv ? line.split('\t') : parseCsvLine(line)
    const front = (cells[0] ?? '').trim()
    const back = (cells[1] ?? '').trim()
    rows.push({ free_content_front: front, free_content_back: back })
  }

  return { rows, warnings }
}

export type BulkImportStreamProgress = {
  bytesRead: number
  totalBytes: number
}

function stripLeadingBom(line: string): string {
  return line.replace(/^\uFEFF/, '')
}

/**
 * Reads the file as a stream (no single huge `file.text()` string) and yields one row per
 * non-empty line, matching {@link parseCsvOrTsvFile} semantics (no embedded newlines in fields).
 */
export async function* eachBulkImportRowFromFile(
  file: File,
  onReadProgress?: (p: BulkImportStreamProgress) => void
): AsyncGenerator<BulkImportRow, void, undefined> {
  let bytesRead = 0
  const totalBytes = file.size
  const reader = file.stream().getReader()
  const decoder = new TextDecoder('utf-8')
  let buffer = ''
  let bomStripped = false
  let format: 'tsv' | 'csv' | null = null
  let firstNonEmptyHandled = false

  const emitProgress = () => {
    onReadProgress?.({ bytesRead, totalBytes })
  }

  const rowFromDataLine = (line: string): BulkImportRow => {
    const cells = format === 'tsv' ? line.split('\t') : parseCsvLine(line)
    const front = (cells[0] ?? '').trim()
    const back = (cells[1] ?? '').trim()
    return { free_content_front: front, free_content_back: back }
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
}

export type BulkDraftRow = {
  id: string
  free_content_front: string
  free_content_back: string
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
    stats.replacedByFront += 1
    return
  }

  const idxSavedBack = rows.findIndex((r) => r.free_content_back.trim() === b)
  if (idxSavedBack !== -1) {
    rows[idxSavedBack].free_content_front = imp.free_content_front
    rows[idxSavedBack].free_content_back = imp.free_content_back
    stats.replacedByBack += 1
    return
  }

  const idxDraftFront = newRows.findIndex((d) => d.free_content_front.trim() === f)
  if (idxDraftFront !== -1) {
    newRows[idxDraftFront].free_content_front = imp.free_content_front
    newRows[idxDraftFront].free_content_back = imp.free_content_back
    stats.replacedByFront += 1
    return
  }

  const idxDraftBack = newRows.findIndex((d) => d.free_content_back.trim() === b)
  if (idxDraftBack !== -1) {
    newRows[idxDraftBack].free_content_front = imp.free_content_front
    newRows[idxDraftBack].free_content_back = imp.free_content_back
    stats.replacedByBack += 1
    return
  }

  const draft = createDraftWithId()
  draft.free_content_front = imp.free_content_front
  draft.free_content_back = imp.free_content_back
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

/**
 * Merge file rows into existing table + drafts (all client-side).
 * Order per row: match saved row by trimmed front → replace; else by trimmed back → replace;
 * else match draft by front; else by back; else append draft. Trailing empty draft is preserved via caller watch.
 */
export function mergeBulkImportRows(
  parsed: BulkImportRow[],
  existingRows: BulkTableRow[],
  drafts: BulkDraftRow[],
  createDraftWithId: () => BulkDraftRow
): { rows: BulkTableRow[]; newRows: BulkDraftRow[]; stats: BulkMergeStats } {
  const rows = existingRows.map((r) => ({ ...r }))
  let newRows = stripTrailingEmptyDrafts(drafts.map((d) => ({ ...d })))
  const stats: BulkMergeStats = {
    replacedByFront: 0,
    replacedByBack: 0,
    inserted: 0,
    skippedEmpty: 0,
  }

  for (const imp of parsed) {
    mergeOneBulkImportRow(imp, rows, newRows, createDraftWithId, stats)
  }

  newRows = finalizeBulkImportDraftRows(newRows, createDraftWithId)

  return { rows, newRows, stats }
}
