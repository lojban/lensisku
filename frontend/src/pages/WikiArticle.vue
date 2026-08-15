<template>
  <div class="container mx-auto px-3 sm:px-6 py-6 max-w-4xl">
    <div class="mb-6 flex items-center gap-3">
      <Button variant="back" type="button" @click="router.back()">
        <ArrowLeft class="h-5 w-5" />
      </Button>
      <SourceTypeBadge type="wiki" />
    </div>

    <div v-if="loading" class="bg-white border border-blue-200 rounded-lg p-6 flex justify-center">
      <Loader2 class="h-8 w-8 animate-spin text-gray-400" />
    </div>

    <div v-else-if="error" class="p-4 bg-red-50 border border-red-100 text-red-700 rounded">
      {{ error }}
    </div>

    <div v-else-if="article">
      <div class="p-4 bg-white rounded-lg shadow-sm border border-gray-100">
        <div class="space-y-6">
          <div
            class="flex flex-wrap items-center justify-between gap-3 mb-4 pb-4 border-b border-gray-100"
          >
            <h1 class="text-2xl font-bold text-gray-800">
              {{ article.title }}
            </h1>
            <div v-if="article.is_native" class="flex items-center gap-2 flex-wrap">
              <Button
                v-if="article.can_edit && !article.is_redirect"
                variant="edit"
                class="inline-flex items-center gap-2 text-sm"
                @click="router.push(`/wiki/${encodedTitle}/edit`)"
              >
                <Pencil class="h-4 w-4" />
                {{ t('wiki.edit') }}
              </Button>
              <Button
                v-if="article.definition_id"
                variant="empty"
                class="inline-flex items-center gap-2 text-sm"
                @click="
                  router.push(
                    `/definition/${article.definition_id}/history?wiki=1&title=${encodedTitle}`
                  )
                "
              >
                <History class="h-4 w-4" />
                {{ t('wiki.history') }}
              </Button>
              <Button
                v-if="article.definition_id && article.valsiid"
                variant="empty"
                class="inline-flex items-center gap-2 text-sm"
                @click="
                  router.push(
                    `/comments?valsi_id=${article.valsiid}&definition_id=${article.definition_id}`
                  )
                "
              >
                <MessageSquare class="h-4 w-4" />
                {{ t('wiki.discussions') }}
              </Button>
            </div>
          </div>

          <div class="flex flex-col md:flex-row gap-4 md:gap-6">
            <div class="space-y-4 md:space-y-6 md:flex-1 min-w-[280px]">
              <div v-if="article.last_edited" class="space-y-1">
                <div class="text-xs font-medium text-gray-500 uppercase tracking-wider">
                  {{ t('wiki.lastEdited') }}
                </div>
                <div class="text-gray-700">
                  {{ new Date(article.last_edited).toLocaleString() }}
                </div>
              </div>
            </div>

            <div v-if="!article.is_native" class="space-y-4 md:space-y-6 md:flex-1 min-w-[280px]">
              <div class="space-y-1">
                <div class="text-xs font-medium text-gray-500 uppercase tracking-wider">
                  {{ t('wiki.viewOnMediawiki') }}
                </div>
                <a
                  :href="article.source_url"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="text-gray-700 text-sm break-words hover:text-blue-600 hover:underline"
                >
                  {{ article.source_url }}
                </a>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="mt-6 p-4 bg-white rounded-lg shadow-sm border border-gray-100 space-y-6">
        <div
          v-if="article.is_redirect"
          class="p-3 bg-yellow-50 border border-yellow-100 rounded text-sm text-yellow-800"
        >
          <template v-if="article.redirect_to">
            {{ t('wiki.redirectTo', { title: article.redirect_to }) }}
          </template>
          <template v-else>
            {{ t('wiki.redirectNotice') }}
          </template>
        </div>

        <div
          class="prose max-w-none text-gray-700 message-content wiki-body"
          @click="onBodyClick"
        >
          <LazyMathJax :content="renderedMarkdown" :enable-markdown="true" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ArrowLeft, Loader2, Pencil, History, MessageSquare } from 'lucide-vue-next'
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
