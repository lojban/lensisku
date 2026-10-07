type MailPart = { mime_type?: string; content?: string }

/** A short, plain-text preview for search and social snippets. */
export function mailSeoPreview(parts: MailPart[] | null | undefined): string {
  if (!Array.isArray(parts)) return ''
  const part =
    parts.find((item) => item.mime_type === 'text/plain' && item.content?.trim()) ||
    parts.find((item) => item.mime_type === 'text/html' && item.content?.trim())
  if (!part?.content) return ''

  const text = part.mime_type === 'text/html' ? part.content.replace(/<[^>]*>/g, ' ') : part.content

  return text
    .replace(/^\s*>.*$/gm, ' ')
    .replace(/&(?:nbsp|amp|lt|gt|quot|#39);/gi, (entity) => {
      const decoded: Record<string, string> = {
        '&nbsp;': ' ',
        '&amp;': '&',
        '&lt;': '<',
        '&gt;': '>',
        '&quot;': '"',
        '&#39;': "'",
      }
      return decoded[entity.toLowerCase()] || entity
    })
    .replace(/\s+/g, ' ')
    .trim()
    .slice(0, 220)
}
