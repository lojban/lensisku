/**
 * Normalize search query text by replacing curly/smart apostrophes (e.g. from iPad)
 * with ASCII single quote (U+0027) so URLs and API params work consistently.
 * @param {string} str - Search query string
 * @returns {string} String with apostrophes normalized to '
 */
export function normalizeSearchQuery(str) {
  if (typeof str !== 'string') return str
  return str
    .replace(/\u2019/g, "'") // RIGHT SINGLE QUOTATION MARK '
    .replace(/\u2018/g, "'") // LEFT SINGLE QUOTATION MARK '
    .replace(/\u02BC/g, "'") // MODIFIER LETTER APOSTROPHE ʼ
}
