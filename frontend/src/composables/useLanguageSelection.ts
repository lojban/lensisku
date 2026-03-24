import type { RouteLocationNormalizedLoaded } from 'vue-router'

import { defaultFilterLanguageTags } from '../config/locales'

export interface LanguageOption {
  id: number
  tag: string
}

export const useLanguageSelection = () => {
  const getStoredLanguages = (): number[] | null | undefined => {
    if (typeof window === 'undefined') return

    try {
      const stored = localStorage.getItem('selectedLanguages')
      return stored ? (JSON.parse(stored) as number[]) : null
    } catch (e) {
      console.error('Error reading from localStorage:', e)
      return null
    }
  }

  const saveLanguages = (languageIds: number[]): void => {
    if (typeof window === 'undefined') return

    try {
      localStorage.setItem('selectedLanguages', JSON.stringify(languageIds))
    } catch (e) {
      console.error('Error saving to localStorage:', e)
    }
  }

  const getInitialLanguages = (
    route: RouteLocationNormalizedLoaded,
    availableLanguages: LanguageOption[]
  ): number[] => {
    const langsQuery = route.query.langs
    if (typeof langsQuery === 'string' && langsQuery) {
      const routeLanguages = langsQuery.split(',').map(Number)
      saveLanguages(routeLanguages)
      return routeLanguages
    }

    const storedLanguages = getStoredLanguages()
    if (storedLanguages) {
      return storedLanguages
    }

    const defaultLanguages = availableLanguages
      .filter((lang) =>
        (defaultFilterLanguageTags as readonly string[]).includes(lang.tag)
      )
      .map((lang) => lang.id)
    saveLanguages(defaultLanguages)
    return defaultLanguages
  }

  return {
    getInitialLanguages,
    saveLanguages,
  }
}
