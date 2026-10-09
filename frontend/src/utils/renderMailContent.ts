import { marked } from 'marked'

/** Shared by mail detail pages, threads and discussion search previews. */
export function renderMailContent(text: string, searchTerm = ''): string {
  if (!text) return ''
  const parsedContent = marked.parse(text.replace(/[\n\r ]+$/, ''), {
    renderer: new marked.Renderer(),
    gfm: true,
    breaks: true,
    async: false,
  }) as string

  // Mail bodies can contain raw HTML. These fragments are rendered with v-html, so CSS and
  // presentation attributes in a message would otherwise apply inside the app or affect layout.
  const safeContent = parsedContent
    .replace(/<style\b[^>]*>[\s\S]*?(?:<\/style\s*>|$)/gi, '')
    .replace(/<link\b[^>]*>/gi, '')
    .replace(/\s(?:style|class|id|width|height)\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)/gi, '')
    .replace(/<\/?font\b[^>]*>/gi, '')

  if (!searchTerm) return safeContent
  const escapedTerm = searchTerm.replace(/\W/g, '\\$&')
  return safeContent.replace(new RegExp(`(${escapedTerm})`, 'gi'), '<mark>$1</mark>')
}
