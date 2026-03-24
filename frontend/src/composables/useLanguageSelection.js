import { defaultFilterLanguageTags } from '../config/locales'

export const useLanguageSelection = () => {
  // Get languages from localStorage or use defaults
  const getStoredLanguages = () => {
    if (typeof window === 'undefined') return

    try {
      const stored = localStorage.getItem('selectedLanguages')
      return stored ? JSON.parse(stored) : null
    } catch (e) {
      console.error('Error reading from localStorage:', e)
      return null
    }
  }

  // Save languages to localStorage
  const saveLanguages = (languageIds) => {
    if (typeof window === 'undefined') return

    try {
      localStorage.setItem('selectedLanguages', JSON.stringify(languageIds))
    } catch (e) {
      console.error('Error saving to localStorage:', e)
    }
  }

  // Get initial languages based on priority:
  // 1. Router query params
  // 2. localStorage
  // 3. Default languages (en, jbo)
  const getInitialLanguages = (route, availableLanguages) => {
    // Check router query params first
    if (route.query.langs) {
      const routeLanguages = route.query.langs.split(',').map(Number)
      saveLanguages(routeLanguages)
      return routeLanguages
    }

    // Check localStorage next
    const storedLanguages = getStoredLanguages()
    if (storedLanguages) {
      return storedLanguages
    }

    // Fall back to default languages
    // Use defaultFilterLanguageTags from the centralized config
    const defaultLanguages = availableLanguages
      .filter((lang) => defaultFilterLanguageTags.includes(lang.tag))
      .map((lang) => lang.id)
    saveLanguages(defaultLanguages)
    return defaultLanguages
  }

  return {
    getInitialLanguages,
    saveLanguages,
  }
}
