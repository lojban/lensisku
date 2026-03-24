/** Client-side check before reading a dropped/selected file (extension or common MIME). */
export function isLikelyCsvOrTsvFile(file: File): boolean {
  if (/\.(csv|tsv)$/i.test(file.name)) return true
  const ty = file.type
  return (
    ty === 'text/csv' ||
    ty === 'text/tab-separated-values' ||
    ty === 'application/vnd.ms-excel' ||
    ty === ''
  )
}
