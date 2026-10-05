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

  if (!searchTerm) return parsedContent
  const escapedTerm = searchTerm.replace(/\W/g, '\\$&')
  return parsedContent.replace(new RegExp(`(${escapedTerm})`, 'gi'), '<mark>$1</mark>')
}
