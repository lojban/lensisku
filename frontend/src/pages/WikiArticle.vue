<template>
  <div class="container mx-auto px-3 sm:px-6 py-6 max-w-4xl">
    <div class="mb-4 flex items-center gap-2">
      <button
        type="button"
        class="ui-btn--empty inline-flex items-center gap-1 px-3 py-1.5 text-sm"
        @click="router.back()"
      >
        <ArrowLeft class="h-4 w-4" />
        <span>{{ t('wiki.back') }}</span>
      </button>
      <SourceTypeBadge type="wiki" />
    </div>

    <div v-if="loading" class="flex justify-center py-12">
      <Loader2 class="h-8 w-8 animate-spin text-gray-400" />
    </div>

    <div v-else-if="error" class="p-4 bg-red-50 border border-red-100 text-red-700 rounded">
      {{ error }}
    </div>

    <article v-else-if="article" class="prose prose-sm sm:prose-base max-w-none">
      <header class="mb-4 not-prose">
        <h1 class="text-2xl sm:text-3xl font-bold text-gray-900">
          {{ article.title }}
        </h1>
        <div class="mt-2 text-xs text-gray-500 flex flex-wrap gap-x-4">
          <span v-if="article.last_edited">
            {{ t('wiki.lastEdited') }}:
            {{ new Date(article.last_edited).toLocaleString() }}
          </span>
          <a
            :href="article.source_url"
            target="_blank"
            rel="noopener noreferrer"
            class="text-blue-600 hover:underline"
          >
            {{ t('wiki.viewOnMediawiki') }}
          </a>
        </div>
      </header>

      <div
        v-if="article.is_redirect"
        class="mb-4 p-3 bg-yellow-50 border border-yellow-100 rounded text-sm text-yellow-800"
      >
        {{ t('wiki.redirectNotice') }}
      </div>

      <LazyMathJax :content="article.markdown" :enable-markdown="true" />
    </article>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ArrowLeft, Loader2 } from 'lucide-vue-next'
import { getWikiArticle } from '@/api'
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
}

const props = defineProps<{ title: string }>()
const router = useRouter()
const { t } = useI18n()

const loading = ref(true)
const error = ref<string | null>(null)
const article = ref<WikiArticleDetail | null>(null)

async function load(title: string) {
  loading.value = true
  error.value = null
  article.value = null
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
