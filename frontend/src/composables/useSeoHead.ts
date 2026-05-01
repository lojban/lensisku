import { computed, toValue } from 'vue'
import type { MaybeRefOrGetter } from 'vue'
import { useHead } from '@vueuse/head'
import { useI18n } from 'vue-i18n'
import { supportedLocales, defaultLocale } from '@/config/locales'

type MetaTag = {
  name?: string
  property?: string
  content: string
  key?: string
}

type SeoConfig = {
  title: MaybeRefOrGetter<string | null | undefined>
  /** Meta description for search snippets and og:description. Keep under ~155 chars. */
  description?: MaybeRefOrGetter<string | null | undefined>
  /** Canonical URL for the page (absolute or path). */
  canonical?: MaybeRefOrGetter<string | null | undefined>
  meta?: MetaTag[]
  /** Path without locale prefix for hreflang generation (e.g., '/collections' for '/en/collections') */
  pathWithoutLocale?: MaybeRefOrGetter<string>
  /** Open Graph image URL (absolute or path). Defaults to site logo. */
  ogImage?: MaybeRefOrGetter<string | null | undefined>
  /** Twitter card type: 'summary', 'summary_large_image', 'app', 'player'. Defaults to 'summary_large_image'. */
  twitterCard?: MaybeRefOrGetter<'summary' | 'summary_large_image' | 'app' | 'player'>
  /** JSON-LD structured data object. */
  jsonLd?: MaybeRefOrGetter<object | null | undefined>
  /** Robots meta tag: 'index', 'noindex', 'nofollow', etc. */
  robots?: MaybeRefOrGetter<string | null | undefined>
}

export function useSeoHead(config: SeoConfig) {
  const i18n = useI18n()
  const baseTitle = computed(() => i18n.t('seo.baseTitle'))

  const resolvedTitle = computed(() => {
    const title = toValue(config.title)
    return title ? `${title} | ${baseTitle.value}` : baseTitle.value
  })

  const resolvedDescription = config.description
    ? computed(() => String(toValue(config.description) ?? '').slice(0, 160))
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
    const ogImage = config.ogImage ? toValue(config.ogImage) : null
    if (ogImage) {
      const imageUrl = ogImage.startsWith('http') ? ogImage : `${typeof window !== 'undefined' ? window.location.origin : ''}${ogImage}`
      tags.push({ property: 'og:image', content: imageUrl })
    }
    return tags
  })

  const descriptionMeta = computed<MetaTag[]>(() =>
    resolvedDescription?.value
      ? [{ name: 'description', content: resolvedDescription.value, key: 'description' }]
      : []
  )

  const metaTags = computed(() => {
    const tags = [
      ...ogTags.value,
      ...descriptionMeta.value,
      ...(config.meta || []),
    ]

    // Twitter Card meta tags
    const twitterCard = config.twitterCard ? toValue(config.twitterCard) : 'summary_large_image'
    tags.push({ name: 'twitter:card', content: twitterCard })
    tags.push({ name: 'twitter:title', content: resolvedTitle.value })
    if (resolvedDescription?.value) {
      tags.push({ name: 'twitter:description', content: resolvedDescription.value })
    }
    const ogImage = config.ogImage ? toValue(config.ogImage) : null
    if (ogImage) {
      const imageUrl = ogImage.startsWith('http') ? ogImage : `${typeof window !== 'undefined' ? window.location.origin : ''}${ogImage}`
      tags.push({ name: 'twitter:image', content: imageUrl })
    }

    // Robots meta tag
    const robots = config.robots ? toValue(config.robots) : null
    if (robots) {
      tags.push({ name: 'robots', content: robots, key: 'robots' })
    }

    return tags
  })

  const linkTags = computed(() => {
    const links: Array<{ rel: string; href?: string; hreflang?: string }> = []
    const canonical = config.canonical ? toValue(config.canonical) : null
    const pathWithoutLocale = config.pathWithoutLocale ? toValue(config.pathWithoutLocale) : null

    // Canonical link
    if (canonical) {
      const href = canonical.startsWith('http')
        ? canonical
        : `${typeof window !== 'undefined' ? window.location.origin : ''}${canonical.startsWith('/') ? '' : '/'}${canonical}`
      links.push({ rel: 'canonical', href })
    }

    // hreflang links for all supported locales
    if (pathWithoutLocale) {
      const origin = typeof window !== 'undefined' ? window.location.origin : ''
      for (const locale of supportedLocales) {
        links.push({
          rel: 'alternate',
          hreflang: locale,
          href: `${origin}/${locale}${pathWithoutLocale}`,
        })
      }
      // x-default points to English
      links.push({
        rel: 'alternate',
        hreflang: 'x-default',
        href: `${origin}/en${pathWithoutLocale}`,
      })
    }

    return links
  })

  useHead({
    title: resolvedTitle,
    meta: metaTags,
    link: linkTags,
    htmlAttrs: {
      lang: i18n.locale.value,
    },
    script: computed(() => {
      const jsonLd = config.jsonLd ? toValue(config.jsonLd) : null
      if (!jsonLd) return []
      return [
        {
          type: 'application/ld+json',
          children: JSON.stringify(jsonLd),
        },
      ]
    }),
  })

  if (typeof document !== 'undefined') {
    document.title = resolvedTitle.value
  }
}
