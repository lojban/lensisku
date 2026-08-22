<template>
  <h1 class="text-2xl font-bold text-gray-800">{{ t('searchExport.title') }}</h1>

  <p class="text-gray-600 my-4">{{ t('searchExport.description') }}</p>

  <div class="p-6 bg-white rounded-lg shadow-sm space-y-6">
    <SearchInput
      v-model="searchQuery"
      class="w-full max-w-3xl"
      :placeholder="t('searchForm.placeholder.dictionary')"
      @search="updateUrlWithFilters"
      @clear="onClearSearch"
    />

    <CombinedFilters
      v-model="filters"
      :languages="languages"
      class="w-full"
      @change="handleFilterChange"
      @reset="handleFiltersReset"
    />

    <p class="text-sm text-gray-500">{{ t('searchExport.emptyFilterHint') }}</p>

    <div class="space-y-2">
      <label class="block text-sm font-medium text-gray-700">{{
        t('dictionaryExport.formatLabel')
      }}</label>
      <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
        <div
          v-for="format in exportFormats"
          :key="format.value"
          class="relative bg-white border rounded-lg cursor-pointer hover:border-blue-500 transition-colors"
          :class="{
            'border-blue-500 ring-2 ring-blue-500': selectedFormat === format.value,
            'border-gray-200': selectedFormat !== format.value,
          }"
          @click="selectedFormat = format.value"
        >
          <div class="p-4">
            <div class="flex items-center justify-between">
              <h3 class="text-sm font-medium text-gray-900">
                {{ t(`dictionaryExport.formats.${format.value}.label`) }}
              </h3>
              <div v-if="selectedFormat === format.value" class="text-blue-500">
                <svg class="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fill-rule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                    clip-rule="evenodd"
                  />
                </svg>
              </div>
            </div>
            <p class="mt-2 text-sm text-gray-500">
              {{ t(`dictionaryExport.formats.${format.value}.description`) }}
            </p>
          </div>
        </div>
      </div>
    </div>

    <div
      class="flex flex-col-reverse items-stretch gap-3 pt-4 sm:flex-row sm:items-center sm:justify-end"
    >
      <p
        class="flex min-h-6 items-center justify-center gap-3 text-sm text-gray-500 sm:justify-end"
        :class="isLoading ? 'visible' : 'invisible'"
        aria-live="polite"
        :aria-busy="isLoading"
      >
        <Loader2 class="h-5 w-5 shrink-0 animate-spin text-blue-500" aria-hidden="true" />
        {{ t('searchExport.generating') }}
      </p>
      <Button
        variant="read"
        :disabled="!canExport || isLoading"
        class="inline-flex w-full items-center justify-center sm:w-auto"
        @click="handleExport"
      >
        {{ t('searchExport.exportButton') }}
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { Loader2 } from '@lucide/vue'
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { exportSearchResults, getLanguages } from '@/api'
import CombinedFilters from '@/components/CombinedFilters.vue'
import SearchInput from '@/components/SearchInput.vue'
import { useError } from '@/composables/useError'
import { useLanguageSelection } from '@/composables/useLanguageSelection'
import { useSeoHead } from '@/composables/useSeoHead'
import {
  combinedFiltersFromQuery,
  combinedFiltersToQuery,
  compactQuery,
  hasActiveSearchFilters,
  queryStr,
} from '@/utils/routeQuery'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'

const { t } = useI18n()
const { showError, clearError } = useError()
const { getInitialLanguages } = useLanguageSelection()
const route = useRoute()
const router = useRouter()

const languages = ref([])
const urlFilters = combinedFiltersFromQuery(route.query)
const searchQuery = ref(normalizeSearchQuery(queryStr(route.query.q)) as string)
const filters = ref({
  ...urlFilters,
  isSemantic: queryStr(route.query.mode) !== 'dictionary',
})
const selectedFormat = ref('pdf')
const isLoading = ref(false)

const exportFormats = [
  { value: 'pdf' },
  { value: 'latex' },
  { value: 'xml' },
  { value: 'json' },
  { value: 'tsv' },
]

useSeoHead({
  title: computed(() => t('searchExport.title')),
  pathWithoutLocale: '/export/search',
})

const canExport = computed(() => {
  return Boolean(searchQuery.value.trim() || hasActiveSearchFilters(filters.value))
})

function updateUrlWithFilters() {
  router.push({
    query: compactQuery({
      q: searchQuery.value || undefined,
      mode: filters.value.isSemantic ? 'semantic' : 'dictionary',
      collection_only: queryStr(route.query.collection_only) === '1' ? '1' : undefined,
      ...combinedFiltersToQuery(filters.value),
    }),
  })
}

function handleFilterChange() {
  updateUrlWithFilters()
}

function handleFiltersReset() {
  filters.value = {
    ...combinedFiltersFromQuery({}),
    selectedLanguages: [],
    searchInPhrases: true,
    isSemantic: true,
  }
  searchQuery.value = ''
  updateUrlWithFilters()
}

function onClearSearch() {
  searchQuery.value = ''
  updateUrlWithFilters()
}

function buildExportParams(): Record<string, string | number | boolean> {
  const params: Record<string, string | number | boolean> = {
    format: selectedFormat.value,
  }
  const search = searchQuery.value.trim()
  if (search) params.search = search
  if (filters.value.selectedLanguages.length > 0) {
    params.languages = filters.value.selectedLanguages.join(',')
  }
  if (filters.value.selmaho) params.selmaho = filters.value.selmaho
  if (!filters.value.selmaho && filters.value.word_type) {
    params.word_type = filters.value.word_type
  }
  if (filters.value.usernames?.length) {
    params.username = filters.value.usernames.join(',')
  }
  if (filters.value.excludeUsernames?.length) {
    params.exclude_usernames = filters.value.excludeUsernames.join(',')
  }
  if (filters.value.source_langid && filters.value.source_langid !== 1) {
    params.source_langid = filters.value.source_langid
  }
  if (filters.value.searchInPhrases !== undefined && filters.value.searchInPhrases !== null) {
    params.search_in_phrases = filters.value.searchInPhrases
  }
  if (filters.value.selectedCollections?.length) {
    params.collection_ids = filters.value.selectedCollections.join(',')
  }
  if (queryStr(route.query.collection_only) === '1') {
    params.collection_only = true
  }
  if (filters.value.isSemantic && search) {
    params.semantic = true
  }
  return params
}

async function messageFromExportError(err: {
  response?: { data?: unknown }
  message?: string
}): Promise<string> {
  const data = err.response?.data
  if (data instanceof Blob) {
    const text = await data.text()
    try {
      const json = JSON.parse(text) as { error?: string; message?: string }
      return json.error || json.message || text
    } catch {
      return text || t('searchExport.failed')
    }
  }
  return err.message || t('searchExport.failed')
}

const handleExport = async () => {
  if (!canExport.value) return

  clearError()
  isLoading.value = true
  updateUrlWithFilters()

  try {
    const response = await exportSearchResults(buildExportParams())
    if (response.status !== 200) {
      showError(t('searchExport.failed'))
      return
    }

    const contentDisposition = response.headers?.['content-disposition']
    const filename = contentDisposition
      ? contentDisposition.split('filename=')[1].replace(/"/g, '')
      : `search-export.${selectedFormat.value === 'tsv' ? 'zip' : selectedFormat.value}`

    const url = window.URL.createObjectURL(response.data)
    const a = document.createElement('a')
    a.href = url
    a.download = filename
    document.body.appendChild(a)
    a.click()
    window.URL.revokeObjectURL(url)
    a.remove()
  } catch (err) {
    showError(
      await messageFromExportError(err as { response?: { data?: unknown }; message?: string })
    )
  } finally {
    isLoading.value = false
  }
}

watch(
  () => route.query,
  (query) => {
    if (query.q !== undefined) {
      searchQuery.value = normalizeSearchQuery(queryStr(query.q)) as string
    }
    const fromQuery = combinedFiltersFromQuery(query)
    if (query.langs !== undefined) {
      filters.value.selectedLanguages = fromQuery.selectedLanguages
    }
    filters.value.selectedCollections = fromQuery.selectedCollections
    filters.value.selmaho = fromQuery.selmaho
    filters.value.usernames = fromQuery.usernames
    filters.value.excludeUsernames = fromQuery.excludeUsernames
    filters.value.word_type = fromQuery.word_type
    filters.value.source_langid = fromQuery.source_langid
    filters.value.searchInPhrases = fromQuery.searchInPhrases
    filters.value.isExpanded = fromQuery.isExpanded
    const mode = queryStr(query.mode)
    if (mode === 'semantic' || mode === 'dictionary') {
      filters.value.isSemantic = mode === 'semantic'
    }
  }
)

onMounted(async () => {
  try {
    const response = await getLanguages()
    languages.value = response.data
    if (!queryStr(route.query.langs)) {
      filters.value.selectedLanguages = getInitialLanguages(route, languages.value)
    }
  } catch {
    showError(t('searchExport.failed'))
  }
})
</script>
