import { readonly, ref } from 'vue'

export const BUTTON_THEME_STORAGE_KEY = 'lensisku.buttonTheme'

export type ButtonThemeId = 'aqua' | 'flat'

export function getStoredButtonTheme(): ButtonThemeId {
  if (typeof window === 'undefined') return 'flat'
  try {
    const t = localStorage.getItem(BUTTON_THEME_STORAGE_KEY)
    if (t === 'aqua' || t === 'flat') return t
  } catch {
    /* ignore */
  }
  return 'flat'
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

const buttonThemeRef = ref<ButtonThemeId>('flat')

export function useButtonTheme() {
  const initButtonTheme = () => {
    buttonThemeRef.value = initButtonThemeFromStorage()
  }

  const setTheme = (theme: ButtonThemeId) => {
    buttonThemeRef.value = theme
    setButtonTheme(theme)
  }

  return {
    buttonTheme: readonly(buttonThemeRef),
    initButtonTheme,
    setButtonTheme: setTheme,
  }
}
