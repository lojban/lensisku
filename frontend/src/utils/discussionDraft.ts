export type DiscussionDraft = { subject: string; markdown: string }
const key = 'lensisku:discussion-draft:v1'
export function readDiscussionDraft(): DiscussionDraft {
  if (typeof window === 'undefined') return { subject: '', markdown: '' }
  try {
    const draft = JSON.parse(sessionStorage.getItem(key) || '{}')
    return {
      subject: typeof draft.subject === 'string' ? draft.subject : '',
      markdown: typeof draft.markdown === 'string' ? draft.markdown : '',
    }
  } catch {
    return { subject: '', markdown: '' }
  }
}
export function saveDiscussionDraft(draft: DiscussionDraft) {
  try {
    sessionStorage.setItem(key, JSON.stringify(draft))
  } catch {
    /* Storage may be unavailable. */
  }
}
export function clearDiscussionDraft() {
  try {
    sessionStorage.removeItem(key)
  } catch {
    /* Storage may be unavailable. */
  }
}
