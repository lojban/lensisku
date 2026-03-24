import { createI18n } from 'vue-i18n'
import { type SupportedLocale, defaultLocale } from './config/locales'

// Import locale messages
// Using vite-plugin-vue-i18n or manual imports
// For simplicity here, we'll manually import
import enMessages from './locales/en.json'
import jboMessages from './locales/jbo.json'
import ruMessages from './locales/ru.json'
type MessageSchema = typeof enMessages

const i18n = createI18n<[MessageSchema], SupportedLocale>({
  legacy: false, // Use Composition API mode
  locale: defaultLocale, // Set default locale
  fallbackLocale: defaultLocale, // Fallback locale if translation is missing
  messages: {
    en: enMessages,
    jbo: { ...enMessages, ...jboMessages } as unknown as typeof enMessages,
    ru: { ...enMessages, ...ruMessages } as unknown as typeof enMessages,
  },
  // Silent fallback warnings if needed
  // silentTranslationWarn: true,
  // silentFallbackWarn: true,
})

export default i18n
