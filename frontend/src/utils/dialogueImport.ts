/**
 * Dialogue story files (duostories-plan/stories/*.json): one story object, an array of them,
 * or `{ stories: [...] }`. Each story needs `script` (the dialogue markup) and `translation`
 * (card back); `title`/`notes` are optional.
 */
export interface DialogueStoryDraft {
  title?: string
  script: string
  translation: string
  notes?: string
}

export function parseDialogueStoryFile(raw: string): DialogueStoryDraft[] {
  const data: unknown = JSON.parse(raw)
  const list: unknown[] = Array.isArray(data)
    ? data
    : data && typeof data === 'object' && Array.isArray((data as { stories?: unknown }).stories)
      ? (data as { stories: unknown[] }).stories
      : [data]
  return list.map((s, i) => {
    const o = (s ?? {}) as Record<string, unknown>
    const script = typeof o.script === 'string' ? o.script.trim() : ''
    const translation = typeof o.translation === 'string' ? o.translation.trim() : ''
    if (!script || !translation) {
      throw new Error(`story ${i + 1}: "script" and "translation" are required`)
    }
    return {
      title: typeof o.title === 'string' ? o.title : undefined,
      script,
      translation,
      notes: typeof o.notes === 'string' ? o.notes : undefined,
    }
  })
}

/** Payload for POST /collections/{id}/items. */
export function dialogueStoryToItem(s: DialogueStoryDraft) {
  return {
    free_content_front: s.script,
    free_content_back: s.translation,
    notes: s.notes ?? '',
    direction: 'dialogue',
    auto_progress: true,
  }
}
