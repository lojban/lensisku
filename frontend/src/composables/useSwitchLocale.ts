import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

/** Switch the UI locale and keep the current route under the new prefix. */
export function useSwitchLocale() {
  const { locale, availableLocales } = useI18n()
  const router = useRouter()
  const route = useRoute()

  function switchLocale(next: string) {
    if (typeof window === 'undefined' || !availableLocales.includes(next)) return
    locale.value = next
    localStorage.setItem('selectedLocale', next)
    const segments = route.path.split('/').filter((segment) => segment !== '')
    if (segments.length > 0 && availableLocales.includes(segments[0])) {
      segments[0] = next
    } else {
      segments.unshift(next)
    }
    void router.push({ path: `/${segments.join('/')}`, query: route.query })
  }

  function onSelectChange(event: Event) {
    const value = (event.target as HTMLSelectElement | null)?.value
    if (value) switchLocale(value)
  }

  return { locale, availableLocales, switchLocale, onSelectChange }
}
