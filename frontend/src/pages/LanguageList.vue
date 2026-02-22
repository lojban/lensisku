<template>
  <!-- Header with stats -->
  <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6">
    <div>
      <h2 class="text-2xl font-bold text-gray-800">
        {{ t('languageList.title') }}
      </h2>
      <p class="text-gray-600 mt-2">
        {{ t('languageList.description') }}
      </p>
    </div>
    <div v-if="languages.length > 0" class="text-sm text-gray-600 bg-gray-100 px-3 py-1.5 rounded-full font-medium shrink-0">
      {{ hasActiveSearch ? t('languageList.showingCount', { count: filteredLanguages.length, total: languages.length }) : t('languageList.totalLabel', { count: languages.length }) }}
    </div>
  </div>

  <!-- Search bar (client-side) -->
  <div v-if="!isLoading && languages.length > 0" class="mb-6">
    <div class="bg-white border border-gray-200 rounded-xl shadow-sm p-4">
      <SearchInput
        v-model="searchQuery"
        :placeholder="t('languageList.searchPlaceholder')"
        :show-search-icon="true"
        @clear="searchQuery = ''"
      />
    </div>
  </div>

  <!-- Loading State -->
  <div v-if="isLoading" class="flex flex-col items-center justify-center py-8">
    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
    <p class="mt-2 text-gray-600">{{ t('languageList.loading') }}</p>
  </div>

  <!-- Language Grid/Cards -->
  <div v-if="!isLoading" class="space-y-4">
    <!-- Table Header - Hidden on Mobile -->
    <div class="hidden sm:grid sm:grid-cols-6 gap-4 py-3 px-4 bg-gray-50 border border-gray-200 rounded-xl font-medium text-gray-700">
      <div class="text-sm font-medium">
        {{ t('languageList.tableHeader.tag') }}
      </div>
      <div class="text-sm font-medium">
        {{ t('languageList.tableHeader.englishName') }}
      </div>
      <div class="text-sm font-medium">
        {{ t('languageList.tableHeader.lojbanName') }}
      </div>
      <div class="text-sm font-medium">
        {{ t('languageList.tableHeader.realName') }}
      </div>
      <div class="text-sm font-medium">
        {{ t('languageList.tableHeader.forLojban') }}
      </div>
      <div class="text-sm font-medium">
        {{ t('languageList.tableHeader.url') }}
      </div>
    </div>

    <!-- Mobile Cards / Desktop Rows -->
    <div class="space-y-4 sm:space-y-0 sm:border sm:border-gray-200 sm:rounded-xl sm:bg-white sm:divide-y sm:divide-gray-200 overflow-hidden">
      <div v-for="lang in filteredLanguages" :key="lang.tag"
        class="sm:[&:not(:last-child)]:border-b-0 sm:[&:last-child]:border-b sm:border-gray-200 block sm:grid sm:grid-cols-6 gap-4 p-4 bg-white rounded-xl sm:rounded-none border border-gray-200 sm:border-0 hover:bg-gray-50/80 transition-colors duration-200">
        <!-- Mobile Layout -->
        <div class="sm:hidden space-y-3">
          <div class="flex flex-wrap gap-2 md:gap-0" role="group">
            <RouterLink :to="`/?mode=dictionary&langs=${lang.id}`" class="btn-get btn-group-item">
              {{ lang.english_name }}
            </RouterLink>
            <a v-if="lang.url" :href="lang.url" target="_blank" rel="noopener noreferrer"
              class="btn-market btn-group-item">
              {{ t('languageList.visitWebsite') }}
            </a>
          </div>
          <div class="text-sm text-secondary-600">
            <div><span class="font-medium">{{ t('languageList.realNameLabel') }}</span> {{ lang.real_name }}</div>
            <div><span class="font-medium">{{ t('languageList.codeLabel') }}</span> {{ lang.tag }}</div>
            <div><span class="font-medium">{{ t('languageList.lojbanLabel') }}</span> {{ lang.lojban_name }}</div>
            <div v-if="lang.for_lojban">
              <span class="font-medium">{{ t('languageList.forLojbanLabel') }}</span>
            </div>
          </div>
        </div>

        <!-- Desktop Layout -->
        <div class="hidden sm:block text-sm font-medium text-primary-900">
          {{ lang.tag }}
        </div>
        <div class="hidden sm:block text-sm text-primary-800">
          <RouterLink :to="`/?mode=dictionary&langs=${lang.id}`" class="btn-get">
            {{ lang.english_name }}
          </RouterLink>
        </div>
        <div class="hidden sm:block text-sm text-primary-800">
          {{ lang.lojban_name }}
        </div>
        <div class="hidden sm:block text-sm text-primary-800">
          {{ lang.real_name }}
        </div>
        <div class="hidden sm:block text-sm text-primary-800">
          <span v-if="lang.for_lojban"
            class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-success-100 text-success-800">
            {{ t('languageList.yes') }}
          </span>
        </div>
        <div class="hidden sm:block text-sm">
          <a v-if="lang.url" :href="lang.url" target="_blank" rel="noopener noreferrer" class="btn-market">
            {{ t('languageList.visit') }}
          </a>
        </div>
      </div>
    </div>
  </div>

  <!-- Empty State (no languages at all) -->
  <div v-if="!isLoading && languages.length === 0"
    class="text-center py-12 bg-gray-50 rounded-xl border border-gray-200">
    <p class="text-gray-600">
      {{ t('languageList.noLanguages') }}
    </p>
  </div>

  <!-- Empty State (search returned no results) -->
  <div v-if="!isLoading && languages.length > 0 && filteredLanguages.length === 0"
    class="text-center py-12 bg-gray-50 rounded-xl border border-gray-200">
    <p class="text-gray-600">
      {{ t('languageList.noMatchingLanguages') }}
    </p>
    <button type="button" class="btn-get mt-3" @click="searchQuery = ''">
      {{ t('languageList.clearSearch') }}
    </button>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'

import SearchInput from '@/components/SearchInput.vue'
import { getLanguages } from '@/api'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const { t, locale } = useI18n()
const languages = ref([])
const searchQuery = ref('')
const isLoading = ref(true)
const { showError, clearError } = useError()

const hasActiveSearch = computed(() => (searchQuery.value || '').trim().length > 0)

const filteredLanguages = computed(() => {
  const q = (searchQuery.value || '').trim().toLowerCase()
  if (!q) return languages.value
  return languages.value.filter((lang) => {
    return (
      (lang.tag && lang.tag.toLowerCase().includes(q)) ||
      (lang.english_name && lang.english_name.toLowerCase().includes(q)) ||
      (lang.lojban_name && lang.lojban_name.toLowerCase().includes(q)) ||
      (lang.real_name && lang.real_name.toLowerCase().includes(q))
    )
  })
})

const fetchLanguages = async () => {
  isLoading.value = true
  clearError()

  try {
    const response = await getLanguages()
    languages.value = response.data
  } catch (e) {
    showError(e.response?.data?.error || t('languageList.loadError'))
    console.error('Error fetching languages:', e)
  } finally {
    isLoading.value = false
  }
}

const pageTitle = ref(t('languageList.title'))

onMounted(() => {
  fetchLanguages()
})

watch(languages, (newLanguages) => {
  if (newLanguages.length > 0) {
    pageTitle.value = `${t('languageList.title')} (${newLanguages.length})`
  } else {
    pageTitle.value = t('languageList.title')
  }
})

useSeoHead({ title: pageTitle }, locale.value)
</script>
