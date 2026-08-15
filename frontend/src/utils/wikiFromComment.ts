/** Session key for wiki create prefill from a comment. */
export const WIKI_FROM_COMMENT_STORAGE_PREFIX = 'lensisku_wiki_from_comment_'

export type WikiFromCommentPrefill = {
  commentId: number
  word: string
  definition: string
  commitMessage?: string
}

export type CommentContentPart = {
  type: string
  data?: string
}

/** Flatten comment JSONB parts into title + markdown body for a wiki page. */
export function commentToWikiPrefill(
  commentId: number,
  content: CommentContentPart[]
): WikiFromCommentPrefill {
  const subject = content.find((p) => p.type === 'header')?.data?.trim() || ''
  const textParts = content
    .filter((p) => p.type === 'text' && p.data?.trim())
    .map((p) => String(p.data).trim())

  const body = textParts.join('\n\n')
  let word = subject
  if (!word) {
    const firstLine = body.split(/\r?\n/).find((l) => l.trim()) || ''
    // Strip lightweight markdown markers for a title suggestion.
    word = firstLine
      .replace(/^#+\s*/, '')
      .replace(/^\*+\s*/, '')
      .replace(/^[-*]\s+/, '')
      .trim()
      .slice(0, 80)
  }

  return {
    commentId,
    word,
    definition: body,
    commitMessage: `Created from comment #${commentId}`,
  }
}

export function storeWikiFromCommentPrefill(prefill: WikiFromCommentPrefill): void {
  sessionStorage.setItem(
    `${WIKI_FROM_COMMENT_STORAGE_PREFIX}${prefill.commentId}`,
    JSON.stringify(prefill)
  )
}

export function loadWikiFromCommentPrefill(commentId: number | string): WikiFromCommentPrefill | null {
  const key = `${WIKI_FROM_COMMENT_STORAGE_PREFIX}${commentId}`
  const raw = sessionStorage.getItem(key)
  if (!raw) return null
  try {
    const parsed = JSON.parse(raw) as WikiFromCommentPrefill
    sessionStorage.removeItem(key)
    if (!parsed || typeof parsed.definition !== 'string') return null
    return parsed
  } catch {
    sessionStorage.removeItem(key)
    return null
  }
}
