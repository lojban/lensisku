import { ref, watch } from 'vue'

import { LANGUAGES } from './languages'

export function useLocalization() {
  if (typeof window === 'undefined')
    return { currentLanguage: ref(LANGUAGES[0]), setLanguage: () => {} }
  const getInitialLanguage = () => {
    const savedLanguageId = localStorage.getItem('lang')
    return LANGUAGES.find((lang) => lang.id === savedLanguageId) || LANGUAGES[0]
  }

  const currentLanguage = ref(getInitialLanguage())

  watch(
    currentLanguage,
    (newLang) => {
      localStorage.setItem('lang', newLang.id)
    },
    { deep: true }
  )

  const setLanguage = (languageId: string) => {
    const newLanguage = LANGUAGES.find((lang) => lang.id === languageId)
    if (newLanguage) {
      currentLanguage.value = newLanguage
      window.location.reload()
    }
  }

  return {
    currentLanguage,
    setLanguage,
  }
}
