import { readonly, ref } from 'vue'

export const BUTTON_THEME_STORAGE_KEY = 'lensisku.buttonTheme'

export type ButtonThemeId = 'aqua' | 'flat'
export const DEFAULT_BUTTON_THEME: ButtonThemeId = 'aqua'
const BUTTON_THEMES: ReadonlySet<ButtonThemeId> = new Set(['aqua', 'flat'])

function isButtonThemeId(value: unknown): value is ButtonThemeId {
  return typeof value === 'string' && BUTTON_THEMES.has(value as ButtonThemeId)
}

export function getStoredButtonTheme(): ButtonThemeId {
  if (typeof window === 'undefined') return DEFAULT_BUTTON_THEME
  try {
    const storedTheme = localStorage.getItem(BUTTON_THEME_STORAGE_KEY)
    if (isButtonThemeId(storedTheme)) return storedTheme
  } catch {
    /* ignore */
  }
  return DEFAULT_BUTTON_THEME
}

export function applyButtonThemeToDocument(theme: ButtonThemeId): void {
  if (typeof document === 'undefined') return
  document.documentElement.dataset.buttonTheme = theme
}

export function setButtonTheme(theme: ButtonThemeId): void {
  applyButtonThemeToDocument(theme)
  try {
    localStorage.setItem(BUTTON_THEME_STORAGE_KEY, theme)
  } catch {
    /* ignore */
  }
}

/** Sync document + storage; call once on client after load (inline script may have run already). */
export function initButtonThemeFromStorage(): ButtonThemeId {
  const t = getStoredButtonTheme()
  applyButtonThemeToDocument(t)
  return t
}

const buttonThemeRef = ref<ButtonThemeId>(DEFAULT_BUTTON_THEME)

export function useButtonTheme() {
  const initButtonTheme = () => {
    buttonThemeRef.value = initButtonThemeFromStorage()
  }

  const updateButtonTheme = (theme: ButtonThemeId) => {
    buttonThemeRef.value = theme
    setButtonTheme(theme)
  }

  return {
    buttonTheme: readonly(buttonThemeRef),
    initButtonTheme,
    setButtonTheme: updateButtonTheme,
  }
}
