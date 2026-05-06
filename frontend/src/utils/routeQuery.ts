/** First string from a vue-router query value (may be `string[]`). */
export function queryStr(v: string | string[] | null | undefined): string {
  if (v == null) return ''
  return Array.isArray(v) ? (v[0] ?? '') : v
}

/** First segment from a route param (may be `string[]`). */
export function paramStr(v: string | string[] | null | undefined): string {
  return queryStr(v)
}
