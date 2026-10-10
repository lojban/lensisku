import { useRoute } from 'vue-router'
import { defaultLocale, supportedLocales } from '@/config/locales'

export function useLocalePath() {
  const route = useRoute()
  return (path = '') => {
    const segment = route.path.split('/')[1]
    const locale = supportedLocales.find((locale) => locale === segment) || defaultLocale
    return `/${locale}${path === '/' ? '' : path.startsWith('/') || !path ? path : `/${path}`}`
  }
}
