<script setup>
import { File } from 'lucide-vue-next'
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import SearchInput from '@/components/SearchInput.vue'
import { listCachedExports, getApiBaseUrl } from '@/api'
import { useSeoHead } from '@/composables/useSeoHead'
import { useError } from '@/composables/useError'

const { t, locale } = useI18n()

useSeoHead({ title: t('cachedExports.title') }, locale.value)

const exports = ref([])
const searchQuery = ref('')
const isLoading = ref(true)
const { showError } = useError()

const hasActiveSearch = computed(() => (searchQuery.value || '').trim().length > 0)

const filteredExports = computed(() => {
  const q = (searchQuery.value || '').trim().toLowerCase()
  if (!q) return exports.value
  return exports.value.filter((item) => {
    return (
      (item.language_tag && item.language_tag.toLowerCase().includes(q)) ||
      (item.language_realname && item.language_realname.toLowerCase().includes(q)) ||
      (item.format && item.format.toLowerCase().includes(q))
    )
  })
})

const formatDate = (dateString) => {
  const date = new Date(dateString)
  return date.toLocaleDateString(locale.value, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

const downloadHref = (exportItem) => {
  const base = getApiBaseUrl()
  const tag = encodeURIComponent(exportItem.language_tag)
  const format = encodeURIComponent(exportItem.format)
  return `${base}/export/cached/${tag}/${format}`
}

onMounted(async () => {
  try {
    const response = await listCachedExports()
    exports.value = response.data
  } catch (err) {
    showError(err.response?.data?.error || t('cachedExports.loadError'))
  } finally {
    isLoading.value = false
  }
})
</script>

<template>
  <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6">
    <div>
      <h2 class="text-2xl font-bold text-gray-800">
        {{ t('cachedExports.title') }}
      </h2>
      <p class="text-gray-600 mt-2">
        {{ t('cachedExports.description') }}
      </p>
    </div>
    <div
      v-if="exports.length > 0"
      class="text-sm text-gray-600 bg-gray-100 px-3 py-1.5 rounded-full font-medium shrink-0"
    >
      {{
        hasActiveSearch
          ? t('cachedExports.showingCount', {
              count: filteredExports.length,
              total: exports.length,
            })
          : t('cachedExports.totalLabel', { count: exports.length })
      }}
    </div>
  </div>

  <!-- Search bar -->
  <div v-if="!isLoading && exports.length > 0" class="mb-6">
    <div class="bg-white border border-gray-200 rounded-xl shadow-sm p-4">
      <SearchInput
        v-model="searchQuery"
        :placeholder="t('cachedExports.searchPlaceholder')"
        :show-search-icon="true"
        @clear="searchQuery = ''"
      />
    </div>
  </div>

  <div v-if="isLoading" class="flex justify-center py-8">
    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
  </div>

  <div v-else>
    <div
      v-if="exports.length === 0"
      class="text-center py-12 bg-blue-50 rounded-lg border border-blue-100"
    >
      <File class="mx-auto h-12 w-12 text-blue-400" />
      <p class="mt-4 text-gray-600">
        {{ t('cachedExports.noExports') }}
      </p>
    </div>

    <div v-else class="bg-white shadow-sm rounded-lg overflow-hidden">
      <div class="divide-y divide-gray-200">
        <div
          v-for="exportItem in filteredExports"
          :key="`${exportItem.language_tag}-${exportItem.format}`"
          class="p-4 hover:bg-gray-50 flex items-center justify-between"
        >
          <div>
            <div class="font-medium text-gray-900">
              {{ exportItem.language_realname }} - {{ exportItem.format.toUpperCase() }}
            </div>
            <div class="text-sm text-gray-500">
              {{ formatDate(exportItem.created_at) }}
            </div>
          </div>
          <a :href="downloadHref(exportItem)" :download="exportItem.filename" class="btn-get">
            {{ t('cachedExports.download') }}
          </a>
        </div>
      </div>
    </div>

    <!-- Empty search state -->
    <div
      v-if="!isLoading && exports.length > 0 && filteredExports.length === 0"
      class="text-center py-12 bg-gray-50 rounded-xl border border-gray-200"
    >
      <p class="text-gray-600">
        {{ t('cachedExports.noMatchingExports') }}
      </p>
      <button type="button" class="btn-get mt-3" @click="searchQuery = ''">
        {{ t('cachedExports.clearSearch') }}
      </button>
    </div>
  </div>
</template>
