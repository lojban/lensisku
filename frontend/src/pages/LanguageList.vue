<template>
  <div class="space-y-4">
    <!-- Header with stats -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <div>
        <h2 class="text-2xl font-bold text-gray-800">{{ t('languageList.title') }}</h2>
        <p class="text-gray-600 mt-2">{{ t('languageList.description') }}</p>
      </div>

      <div
        v-if="languages.length > 0"
        class="text-sm text-gray-600 bg-gray-100 px-3 py-1.5 rounded-full font-medium shrink-0"
      >
        {{
          hasActiveSearchOrFilters
            ? t('languageList.showingCount', {
                count: filteredLanguages.length,
                total: languages.length,
              })
            : t('languageList.totalLabel', { count: languages.length })
        }}
      </div>
    </div>

    <!-- Toolbar -->
    <div v-if="!isLoading && languages.length > 0" class="toolbar-panel">
      <div class="toolbar-row">
        <div class="toolbar-search-slot">
          <SearchInput
            v-model="searchQuery"
            :is-loading="isSearching"
            :placeholder="t('languageList.searchPlaceholder')"
            :show-search-icon="true"
            @clear="clearSearch"
          />
        </div>

        <div class="toolbar-field-row">
          <label class="toolbar-control-label">{{ t('languageList.filters.forLojbanLabel') }}</label>
          <div class="toolbar-dropdown-anchor">
            <ToolbarSelectDropdown truncate-label>
              <template #label>{{ forLojbanTriggerLabel }}</template>
              <ToolbarSelectDropdownItem :class="{ 'bg-gray-100': !forLojbanFilter }" @click="setForLojbanFilter('')">
                {{ t('languageList.filters.all') }}
              </ToolbarSelectDropdownItem>
              <ToolbarSelectDropdownItem
                :class="{ 'bg-gray-100': forLojbanFilter === 'yes' }"
                @click="setForLojbanFilter('yes')"
              >
                {{ t('languageList.filters.forLojban.yes') }}
              </ToolbarSelectDropdownItem>
              <ToolbarSelectDropdownItem
                :class="{ 'bg-gray-100': forLojbanFilter === 'no' }"
                @click="setForLojbanFilter('no')"
              >
                {{ t('languageList.filters.forLojban.no') }}
              </ToolbarSelectDropdownItem>
            </ToolbarSelectDropdown>
          </div>
        </div>

        <div class="toolbar-field-row">
          <label class="toolbar-control-label">{{ t('languageList.filters.urlLabel') }}</label>
          <div class="toolbar-dropdown-anchor">
            <ToolbarSelectDropdown truncate-label>
              <template #label>{{ urlTriggerLabel }}</template>
              <ToolbarSelectDropdownItem :class="{ 'bg-gray-100': !urlFilter }" @click="setUrlFilter('')">
                {{ t('languageList.filters.all') }}
              </ToolbarSelectDropdownItem>
              <ToolbarSelectDropdownItem
                :class="{ 'bg-gray-100': urlFilter === 'yes' }"
                @click="setUrlFilter('yes')"
              >
                {{ t('languageList.filters.url.hasUrl') }}
              </ToolbarSelectDropdownItem>
              <ToolbarSelectDropdownItem
                :class="{ 'bg-gray-100': urlFilter === 'no' }"
                @click="setUrlFilter('no')"
              >
                {{ t('languageList.filters.url.noUrl') }}
              </ToolbarSelectDropdownItem>
            </ToolbarSelectDropdown>
          </div>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="flex flex-col items-center justify-center py-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
      <p class="mt-2 text-gray-600">{{ t('languageList.loading') }}</p>
    </div>

    <!-- Main content -->
    <div v-else class="space-y-3">
      <!-- Empty state: no languages -->
      <div
        v-if="languages.length === 0"
        class="text-center py-12 bg-gray-50 rounded-xl border border-gray-200"
      >
        <p class="text-gray-600">{{ t('languageList.noLanguages') }}</p>
      </div>

      <!-- Empty state: no matches -->
      <div
        v-else-if="filteredLanguages.length === 0"
        class="text-center py-12 bg-gray-50 rounded-xl border border-gray-200"
      >
        <p class="text-gray-600">{{ t('languageList.noMatchingLanguages') }}</p>
        <Button variant="read" type="button" class="mt-3" @click="clearSearchAndFilters">
          {{ t('languageList.clearSearch') }}
        </Button>
      </div>

      <div v-else class="space-y-3">
        <ListRowSurface
          v-for="lang in filteredLanguages"
          :key="lang.id"
          :class="['cursor-pointer transition-colors hover:bg-blue-50/40']"
          @click="openLanguage(lang)"
        >
          <div class="flex items-start gap-4">
            <!-- Badge / thumbnail -->
            <div class="shrink-0 mt-1">
              <div
                class="w-10 h-10 rounded-lg flex items-center justify-center text-sm font-semibold border bg-white"
                :class="lang.for_lojban ? 'border-green-200 text-green-700' : 'border-gray-200 text-gray-600'"
              >
                {{ lang.tag }}
              </div>
            </div>

            <div class="min-w-0 flex-1">
              <div class="flex justify-between items-start gap-2 min-w-0">
                <div class="min-w-0 flex-1 pr-1">
                  <h3 class="text-lg font-medium text-blue-600 break-words">
                    {{ lang.english_name }}
                  </h3>
                  <p class="text-gray-600 text-sm mt-0.5 break-words">{{ lang.lojban_name }}</p>
                </div>

                <span
                  class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium shrink-0"
                  :class="
                    lang.for_lojban ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                  "
                >
                  {{ t('languageList.tableHeader.forLojban') }}:
                  {{ lang.for_lojban ? t('languageList.filters.forLojban.yes') : t('languageList.filters.forLojban.no') }}
                </span>
              </div>

              <div class="mt-3 text-sm text-gray-600 space-y-1">
                <div>
                  <span class="font-medium text-gray-500">{{ t('languageList.realNameLabel') }}</span>
                  {{ lang.real_name }}
                </div>
                <div>
                  <span class="font-medium text-gray-500">{{ t('languageList.codeLabel') }}</span>
                  {{ lang.tag }}
                </div>
                <div v-if="lang.url">
                  <span class="font-medium text-gray-500">{{ t('languageList.tableHeader.url') }}</span>
                  <a
                    :href="lang.url"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-blue-600 hover:underline"
                    @click.stop
                  >
                    {{ t('languageList.visit') }}
                  </a>
                </div>
              </div>

              <div class="flex flex-wrap gap-2 mt-3">
                <RouterLink
                  :to="`/?mode=dictionary&langs=${lang.id}`"
                  class="ui-btn--read ui-btn--group-item"
                  @click.stop
                >
                  {{ t('languageList.viewInDictionary') }}
                </RouterLink>
                <a
                  v-if="lang.url"
                  :href="lang.url"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="ui-btn--market ui-btn--group-item"
                  @click.stop
                >
                  {{ t('languageList.visitWebsite') }}
                </a>
              </div>
            </div>
          </div>
        </ListRowSurface>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button, ListRowSurface, ToolbarSelectDropdown, ToolbarSelectDropdownItem } from '@packages/ui'
import { ref, computed, onMounted, watch } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import SearchInput from '@/components/SearchInput.vue'
import { getLanguages } from '@/api'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const { t } = useI18n()
const router = useRouter()

type LanguageRow = {
  id: number
  tag: string
  english_name: string
  lojban_name: string
  real_name: string
  for_lojban?: string | null
  url?: string | null
}

const languages = ref<LanguageRow[]>([])
const searchQuery = ref('')
const isLoading = ref(true)
const isSearching = ref(false)
const { showError, clearError } = useError()

const forLojbanFilter = ref<'' | 'yes' | 'no'>('')
const urlFilter = ref<'' | 'yes' | 'no'>('')

const hasActiveSearchOrFilters = computed(() => {
  const qActive = (searchQuery.value || '').trim().length > 0
  return qActive || !!forLojbanFilter.value || !!urlFilter.value
})

const forLojbanTriggerLabel = computed(() => {
  if (forLojbanFilter.value === 'yes') return t('languageList.filters.forLojban.yes')
  if (forLojbanFilter.value === 'no') return t('languageList.filters.forLojban.no')
  return t('languageList.filters.all')
})

const urlTriggerLabel = computed(() => {
  if (urlFilter.value === 'yes') return t('languageList.filters.url.hasUrl')
  if (urlFilter.value === 'no') return t('languageList.filters.url.noUrl')
  return t('languageList.filters.all')
})

const filteredLanguages = computed(() => {
  const q = (searchQuery.value || '').trim().toLowerCase()

  const matchesSearch = (lang: LanguageRow) => {
    if (!q) return true
    return (
      (lang.tag && lang.tag.toLowerCase().includes(q)) ||
      (lang.english_name && lang.english_name.toLowerCase().includes(q)) ||
      (lang.lojban_name && lang.lojban_name.toLowerCase().includes(q)) ||
      (lang.real_name && lang.real_name.toLowerCase().includes(q))
    )
  }

  const matchesForLojban = (lang: LanguageRow) => {
    if (!forLojbanFilter.value) return true
    const isYes = !!lang.for_lojban
    return forLojbanFilter.value === 'yes' ? isYes : !isYes
  }

  const matchesUrl = (lang: LanguageRow) => {
    if (!urlFilter.value) return true
    const has = !!(lang.url && lang.url.trim())
    return urlFilter.value === 'yes' ? has : !has
  }

  return languages.value.filter((lang) => matchesSearch(lang) && matchesForLojban(lang) && matchesUrl(lang))
})

const setForLojbanFilter = (v: '' | 'yes' | 'no') => {
  forLojbanFilter.value = v
}

const setUrlFilter = (v: '' | 'yes' | 'no') => {
  urlFilter.value = v
}

const clearSearch = () => {
  searchQuery.value = ''
  isSearching.value = false
}

const clearSearchAndFilters = () => {
  searchQuery.value = ''
  forLojbanFilter.value = ''
  urlFilter.value = ''
  isSearching.value = false
}

const openLanguage = (lang: LanguageRow) => {
  router.push(`/language/${lang.id}`)
}

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

useSeoHead({ title: pageTitle, pathWithoutLocale: '/languages' })
</script>
