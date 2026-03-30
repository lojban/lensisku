/** Rows match bulk custom-text import: columns front, back (see parseDelimitedImport). */

type BulkCustomTextExportRow = {
  free_content_front: string
  free_content_back: string
}

function escapeCsvField(s: string): string {
  if (/[",\r\n]/.test(s)) {
    return `"${s.replace(/"/g, '""')}"`
  }
  return s
}

/** Sanitize cell for TSV: tabs/newlines break one-row-per-line layout used by import. */
function sanitizeTsvCell(s: string): string {
  return s.replace(/\r\n|\r|\n/g, ' ').replace(/\t/g, ' ')
}

export function buildBulkCustomTextCsv(rows: BulkCustomTextExportRow[]): string {
  const header = 'front,back'
  const lines = rows.map(
    (r) =>
      `${escapeCsvField(r.free_content_front)},${escapeCsvField(r.free_content_back)}`
  )
  return [header, ...lines].join('\r\n')
}

export function buildBulkCustomTextTsv(rows: BulkCustomTextExportRow[]): string {
  const header = 'front\tback'
  const lines = rows.map(
    (r) =>
      `${sanitizeTsvCell(r.free_content_front)}\t${sanitizeTsvCell(r.free_content_back)}`
  )
  return [header, ...lines].join('\r\n')
}

export function downloadTextFile(filename: string, content: string, mime: string): void {
  const blob = new Blob([`\uFEFF${content}`], { type: `${mime};charset=utf-8` })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.rel = 'noopener'
  a.click()
  URL.revokeObjectURL(url)
}
