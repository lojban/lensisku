import { createI18n } from 'vue-i18n'
import { type SupportedLocale, defaultLocale } from './config/locales'

import enMessages from './locales/en.json'
import jboMessages from './locales/jbo.json'
import ruMessages from './locales/ru.json'
import jaMessages from './locales/ja.json'
import zhMessages from './locales/zh.json'
type MessageSchema = typeof enMessages

const i18n = createI18n<[MessageSchema], SupportedLocale>({
  legacy: false,
  locale: defaultLocale,
  fallbackLocale: defaultLocale,
  messages: {
    en: enMessages,
    jbo: { ...enMessages, ...jboMessages } as unknown as typeof enMessages,
    ru: { ...enMessages, ...ruMessages } as unknown as typeof enMessages,
    ja: { ...enMessages, ...jaMessages } as unknown as typeof enMessages,
    zh: { ...enMessages, ...zhMessages } as unknown as typeof enMessages,
  },
})

export default i18n
