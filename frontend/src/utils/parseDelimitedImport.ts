/** Two columns: front (prompt) and back (answer). */

type BulkImportRow = { free_content_front: string; free_content_back: string }

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

type BulkTableRow = {
  item_id: number
  position: number
  free_content_front: string
  free_content_back: string
}

type BulkDraftRow = {
  id: string
  free_content_front: string
  free_content_back: string
}

type BulkMergeStats = {
  replacedByFront: number
  replacedByBack: number
  inserted: number
  skippedEmpty: number
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
  let newRows = drafts.map((d) => ({ ...d }))
  const stats: BulkMergeStats = {
    replacedByFront: 0,
    replacedByBack: 0,
    inserted: 0,
    skippedEmpty: 0,
  }

  for (const imp of parsed) {
    const f = imp.free_content_front.trim()
    const b = imp.free_content_back.trim()
    if (!f && !b) {
      stats.skippedEmpty += 1
      continue
    }

    const idxSavedFront = rows.findIndex((r) => r.free_content_front.trim() === f)
    if (idxSavedFront !== -1) {
      rows[idxSavedFront].free_content_front = imp.free_content_front
      rows[idxSavedFront].free_content_back = imp.free_content_back
      stats.replacedByFront += 1
      continue
    }

    const idxSavedBack = rows.findIndex((r) => r.free_content_back.trim() === b)
    if (idxSavedBack !== -1) {
      rows[idxSavedBack].free_content_front = imp.free_content_front
      rows[idxSavedBack].free_content_back = imp.free_content_back
      stats.replacedByBack += 1
      continue
    }

    const idxDraftFront = newRows.findIndex((d) => d.free_content_front.trim() === f)
    if (idxDraftFront !== -1) {
      newRows[idxDraftFront].free_content_front = imp.free_content_front
      newRows[idxDraftFront].free_content_back = imp.free_content_back
      stats.replacedByFront += 1
      continue
    }

    const idxDraftBack = newRows.findIndex((d) => d.free_content_back.trim() === b)
    if (idxDraftBack !== -1) {
      newRows[idxDraftBack].free_content_front = imp.free_content_front
      newRows[idxDraftBack].free_content_back = imp.free_content_back
      stats.replacedByBack += 1
      continue
    }

    const draft = createDraftWithId()
    draft.free_content_front = imp.free_content_front
    draft.free_content_back = imp.free_content_back
    newRows.push(draft)
    stats.inserted += 1
  }

  if (newRows.length === 0) {
    newRows = [createDraftWithId()]
  } else {
    const last = newRows[newRows.length - 1]
    const lastEmpty =
      !last.free_content_front.trim() && !last.free_content_back.trim()
    if (!lastEmpty) {
      newRows = [...newRows, createDraftWithId()]
    }
  }

  return { rows, newRows, stats }
}
