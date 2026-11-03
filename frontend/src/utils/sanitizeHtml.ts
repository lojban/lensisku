import DOMPurify from 'dompurify'

/**
 * Markdown source may contain `<T | U>` and comparisons. Escape every `<` so
 * marked and DOMPurify keep it as text. `>` stays so blockquotes still work.
 * Callers that already store sanitized HTML (wiki) pass `allowRawHtml` and
 * rely on `sanitizeHtml` for the tag policy.
 */
export function escapeMarkdownAngles(source: string, allowRawHtml = false): string {
  if (!source || allowRawHtml) return source
  return source.replace(/</g, '&lt;')
}

/** Escape a text fragment that will be inserted into HTML (restored MathJax). */
export function escapeHtmlText(source: string): string {
  return source.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

/** Sanitize the final rendered HTML, including Markdown and highlighting output. */
export function sanitizeHtml(html: string): string {
  if (typeof window === 'undefined') {
    return html.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  }
  return DOMPurify.sanitize(html, {
    USE_PROFILES: { html: true },
    FORBID_TAGS: ['style', 'link', 'iframe', 'object', 'embed'],
    FORBID_ATTR: ['style'],
  })
}
