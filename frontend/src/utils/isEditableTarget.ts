/**
 * True when a keyboard event (or element) belongs to a place where the user types text:
 * <input> (text-like), <textarea>, <select>, or any contenteditable host such as the
 * Milkdown / ProseMirror comment editor. Global shortcut handlers (Space / Enter / digits /
 * "/") must bail out on these so the keys reach the editor.
 */
export function isEditableTarget(target: EventTarget | Element | null | undefined): boolean {
  const el = target instanceof Element ? target : null
  if (!el) return false
  const tag = el.tagName
  if (tag === 'TEXTAREA' || tag === 'SELECT') return true
  if (tag === 'INPUT') {
    const type = ((el as HTMLInputElement).type || 'text').toLowerCase()
    return ![
      'button',
      'checkbox',
      'radio',
      'submit',
      'reset',
      'range',
      'color',
      'file',
      'image',
    ].includes(type)
  }
  if ((el as HTMLElement).isContentEditable) return true
  return !!el.closest('[contenteditable=""], [contenteditable="true"], .ProseMirror')
}

/** `isEditableTarget` for the event target or, failing that, the focused element. */
export function isTypingInEditable(event?: Event): boolean {
  return (
    isEditableTarget(event?.target as EventTarget | null) ||
    (typeof document !== 'undefined' && isEditableTarget(document.activeElement))
  )
}
