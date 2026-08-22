<template>
  <div class="wiki-article">
    <div class="wiki-article-toolbar">
      <Button variant="back" type="button" @click="router.back()">
        <ArrowLeft class="h-5 w-5" />
      </Button>
      <SourceTypeBadge type="wiki" />
      <div v-if="article?.is_native" class="wiki-article-actions ml-auto">
        <Button
          v-if="article.can_edit && !article.is_redirect"
          variant="edit"
          class="inline-flex items-center"
          @click="router.push(`/wiki/${encodedTitle}/edit`)"
        >
          <Pencil class="h-4 w-4" />
          <span class="hidden md:inline">{{ t('wiki.edit') }}</span>
          <span class="sr-only md:hidden">{{ t('wiki.edit') }}</span>
        </Button>
        <Button
          v-if="article.definition_id"
          variant="empty"
          class="inline-flex items-center"
          @click="
            router.push(
              `/definition/${article.definition_id}/history?wiki=1&title=${encodedTitle}`
            )
          "
        >
          <History class="h-4 w-4" />
          <span class="hidden md:inline">{{ t('wiki.history') }}</span>
          <span class="sr-only md:hidden">{{ t('wiki.history') }}</span>
        </Button>
        <Button
          v-if="article.definition_id && article.valsiid"
          variant="empty"
          class="inline-flex items-center"
          @click="
            router.push(
              `/comments?valsi_id=${article.valsiid}&definition_id=${article.definition_id}`
            )
          "
        >
          <MessageSquare class="h-4 w-4" />
          <span class="hidden md:inline">{{ t('wiki.discussions') }}</span>
          <span class="sr-only md:hidden">{{ t('wiki.discussions') }}</span>
        </Button>
      </div>
    </div>

    <div v-if="loading" class="wiki-article-card flex justify-center py-6">
      <Loader2 class="h-8 w-8 animate-spin text-gray-400" />
    </div>

    <div v-else-if="error" class="rounded border border-red-100 bg-red-50 p-3 text-red-700">
      {{ error }}
    </div>

    <div v-else-if="article" class="wiki-article-card">
      <header class="wiki-article-header">
        <h1 class="wiki-article-title">
          {{ article.title }}
        </h1>
        <p v-if="article.last_edited" class="wiki-article-meta">
          {{ t('wiki.lastEdited') }}
          {{ new Date(article.last_edited).toLocaleString() }}
        </p>
        <a
          v-if="!article.is_native && article.source_url"
          :href="article.source_url"
          target="_blank"
          rel="noopener noreferrer"
          class="wiki-article-meta mt-0.5 block break-words hover:text-blue-600 hover:underline"
        >
          {{ t('wiki.viewOnMediawiki') }}
        </a>
      </header>

      <div
        v-if="article.is_redirect"
        class="mb-2 rounded border border-yellow-100 bg-yellow-50 p-2 text-sm text-yellow-800"
      >
        <template v-if="article.redirect_to">
          {{ t('wiki.redirectTo', { title: article.redirect_to }) }}
        </template>
        <template v-else>
          {{ t('wiki.redirectNotice') }}
        </template>
      </div>

      <div class="wiki-article-body" @click="onBodyClick">
        <LazyMathJax :content="renderedMarkdown" :enable-markdown="true" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ArrowLeft, Loader2, Pencil, History, MessageSquare } from '@lucide/vue'
import { getWikiArticle, getNativeWikiArticle, getWikiByDefinitionId } from '@/api'
import LazyMathJax from '@/components/LazyMathJax.vue'
import SourceTypeBadge from '@/components/SourceTypeBadge.vue'

interface WikiArticleDetail {
  page_id: number
  namespace: number
  title: string
  markdown: string
  last_edited: string | null
  is_redirect: boolean
  redirect_to?: string | null
  source_url: string
  is_native?: boolean
  definition_id?: number | null
  valsiid?: number | null
  can_edit?: boolean
}

const props = defineProps<{
  title?: string
  definitionId?: number | string
}>()
const router = useRouter()
const { t } = useI18n()

const loading = ref(true)
const error = ref<string | null>(null)
const article = ref<WikiArticleDetail | null>(null)
const followedOnce = ref(false)

const encodedTitle = computed(() =>
  encodeURIComponent((article.value?.title || props.title || '').replace(/ /g, '_'))
)

function wikiLinkify(markdown: string): string {
  // Convert bare [[Title]] to markdown links when not already linked.
  return markdown.replace(/\[\[([^\]]+)\]\](?!\()/g, (_match, page: string) => {
    const title = String(page).trim()
    const slug = title.replace(/ /g, '_')
    return `[${title}](/wiki/${encodeURIComponent(slug)})`
  })
}

const renderedMarkdown = computed(() => wikiLinkify(article.value?.markdown || ''))

function onBodyClick(event: MouseEvent) {
  const target = event.target as HTMLElement | null
  const anchor = target?.closest('a') as HTMLAnchorElement | null
  if (!anchor) return
  const href = anchor.getAttribute('href')
  if (!href?.startsWith('/wiki/')) return
  event.preventDefault()
  router.push(href)
}

function metadataRedirect(def: {
  metadata?: { is_redirect?: boolean; redirect_to?: string } | null
}): { is_redirect: boolean; redirect_to: string | null } {
  const meta = def.metadata
  if (!meta || typeof meta !== 'object') {
    return { is_redirect: false, redirect_to: null }
  }
  return {
    is_redirect: Boolean(meta.is_redirect),
    redirect_to: typeof meta.redirect_to === 'string' ? meta.redirect_to : null,
  }
}

async function loadNativeByTitle(title: string): Promise<boolean> {
  try {
    const nativeResp = await getNativeWikiArticle(title)
    const def = nativeResp.data
    if (!def) return false

    const { is_redirect, redirect_to } = metadataRedirect(def)
    article.value = {
      page_id: def.definitionid,
      namespace: 0,
      title: def.valsiword,
      markdown: def.definition,
      last_edited: def.created_at,
      is_redirect,
      redirect_to,
      source_url: '',
      is_native: true,
      definition_id: def.definitionid,
      valsiid: def.valsiid,
      can_edit: def.can_edit,
    }

    if (is_redirect && redirect_to && !followedOnce.value) {
      followedOnce.value = true
      await router.replace(`/wiki/${encodeURIComponent(redirect_to.replace(/ /g, '_'))}`)
      return true
    }
    return true
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } })?.response?.status
    if (status !== 404) {
      console.error('Error loading native wiki:', e)
    }
    return false
  }
}

async function loadByDefinitionId(id: number) {
  loading.value = true
  error.value = null
  article.value = null
  try {
    const resp = await getWikiByDefinitionId(id)
    const page = resp.data
    if (page.is_redirect && page.redirect_to && !followedOnce.value) {
      followedOnce.value = true
      await router.replace(`/wiki/${encodeURIComponent(page.redirect_to.replace(/ /g, '_'))}`)
      return
    }
    await router.replace(`/wiki/${encodeURIComponent(page.word.replace(/ /g, '_'))}`)
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } })?.response?.status
    error.value = status === 404 ? t('wiki.notFound') : t('wiki.loadFailed')
    loading.value = false
  }
}

async function load(title: string) {
  loading.value = true
  error.value = null
  article.value = null

  const foundNative = await loadNativeByTitle(title)
  if (foundNative) {
    loading.value = false
    return
  }

  try {
    const resp = await getWikiArticle(title)
    article.value = resp.data as WikiArticleDetail
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } })?.response?.status
    error.value = status === 404 ? t('wiki.notFound') : t('wiki.loadFailed')
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  if (props.definitionId != null && props.definitionId !== '') {
    loadByDefinitionId(Number(props.definitionId))
  } else if (props.title) {
    load(props.title)
  }
})

watch(
  () => props.title,
  (newTitle) => {
    if (newTitle) {
      followedOnce.value = false
      load(newTitle)
    }
  }
)

watch(
  () => props.definitionId,
  (id) => {
    if (id != null && id !== '') {
      followedOnce.value = false
      loadByDefinitionId(Number(id))
    }
  }
)
</script>
