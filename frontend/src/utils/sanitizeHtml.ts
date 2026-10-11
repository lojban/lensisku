import DOMPurify from 'dompurify'

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
