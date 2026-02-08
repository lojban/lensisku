import { unref, computed } from 'vue'
import { useHead } from '@vueuse/head'
import { useI18n } from 'vue-i18n'

type MetaTag = {
  name?: string
  property?: string
  content: string
  key?: string
}

type SeoConfig = {
  title: string | ReturnType<typeof computed<string>>
  /** Meta description for search snippets and og:description. Keep under ~155 chars. */
  description?: string | ReturnType<typeof computed<string>>
  /** Canonical URL for the page (absolute or path). */
  canonical?: string | ReturnType<typeof computed<string>>
  meta?: MetaTag[]
}

export function useSeoHead(config: SeoConfig) {
  const i18n = useI18n()
  const baseTitle = computed(() => i18n.t('seo.baseTitle'))

  const resolvedTitle = computed(() => {
    const title = unref(config.title)
    return title ? `${title} | ${baseTitle.value}` : baseTitle.value
  })

  const resolvedDescription = config.description
    ? computed(() => String(unref(config.description) ?? '').slice(0, 160))
    : null

  const ogTags = computed<MetaTag[]>(() => {
    const tags: MetaTag[] = [
      { property: 'og:title', content: resolvedTitle.value },
      { property: 'og:site_name', content: baseTitle.value },
      { property: 'og:type', content: 'website' },
      { property: 'og:locale', content: i18n.locale.value.replace('-', '_') },
    ]
    if (resolvedDescription?.value) {
      tags.push({ property: 'og:description', content: resolvedDescription.value })
    }
    return tags
  })

  const descriptionMeta = computed<MetaTag[]>(() =>
    resolvedDescription?.value
      ? [{ name: 'description', content: resolvedDescription.value, key: 'description' }]
      : []
  )

  const metaTags = computed(() => [
    ...ogTags.value,
    ...descriptionMeta.value,
    ...(config.meta || []),
  ])

  const linkTags = computed(() => {
    const canonical = config.canonical ? unref(config.canonical) : null
    if (!canonical) return []
    const href = canonical.startsWith('http')
      ? canonical
      : `${typeof window !== 'undefined' ? window.location.origin : ''}${canonical.startsWith('/') ? '' : '/'}${canonical}`
    return [{ rel: 'canonical', href }]
  })

  useHead({
    title: resolvedTitle,
    meta: metaTags,
    link: linkTags,
    htmlAttrs: {
      lang: i18n.locale.value,
    },
  })

  if (typeof document !== 'undefined') {
    document.title = resolvedTitle.value
  }
}