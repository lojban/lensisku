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
            <div v-if="article.is_native" class="flex items-center gap-2">
              <Button
                v-if="article.can_edit"
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
                @click="router.push(`/definition/${article.definition_id}/history`)"
              >
                <History class="h-4 w-4" />
                {{ t('wiki.history') }}
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
          {{ t('wiki.redirectNotice') }}
        </div>

        <div class="prose max-w-none text-gray-700 message-content">
          <LazyMathJax :content="article.markdown" :enable-markdown="true" />
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
import { ArrowLeft, Loader2, Pencil, History } from 'lucide-vue-next'
import { getWikiArticle, getNativeWikiArticle } from '@/api'
import LazyMathJax from '@/components/LazyMathJax.vue'
import SourceTypeBadge from '@/components/SourceTypeBadge.vue'

interface WikiArticleDetail {
  page_id: number
  namespace: number
  title: string
  markdown: string
  last_edited: string | null
  is_redirect: boolean
  source_url: string
  is_native?: boolean
  definition_id?: number | null
  can_edit?: boolean
}

const props = defineProps<{ title: string }>()
const router = useRouter()
const { t } = useI18n()

const loading = ref(true)
const error = ref<string | null>(null)
const article = ref<WikiArticleDetail | null>(null)

const encodedTitle = computed(() => encodeURIComponent(props.title.replace(/ /g, '_')))

async function load(title: string) {
  loading.value = true
  error.value = null
  article.value = null

  // Try native wiki first; if none exists, fall back to the mw.lojban.org mirror.
  try {
    const nativeResp = await getNativeWikiArticle(title)
    const def = nativeResp.data
    if (def) {
      article.value = {
        page_id: def.definitionid,
        namespace: 0,
        title: def.valsiword,
        markdown: def.definition,
        last_edited: def.created_at,
        is_redirect: false,
        source_url: '',
        is_native: true,
        definition_id: def.definitionid,
        can_edit: def.can_edit,
      }
      loading.value = false
      return
    }
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } })?.response?.status
    if (status !== 404) {
      console.error('Error loading native wiki:', e)
    }
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

onMounted(() => load(props.title))
watch(
  () => props.title,
  (newTitle) => {
    if (newTitle) load(newTitle)
  }
)
</script>
