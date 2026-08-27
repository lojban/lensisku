<template>
  <!-- Search and Filter Section -->
  <!-- Skeletons -->
  <SearchFormSkeleton v-if="isInitialLoading" />
  <CombinedFiltersSkeleton v-if="isInitialLoading" />
  <!-- Search Form -->
  <SearchForm
    v-if="!isInitialLoading"
    ref="searchFormRef"
    :initial-query="searchQuery"
    :initial-mode="'semantic'"
    class="w-full"
    @search="performSearch"
  />
  <CombinedFilters
    v-model="filters"
    :languages="languages"
    class="w-full transition-opacity duration-300"
    :class="{ 'opacity-0 pointer-events-none h-0 overflow-hidden': isInitialLoading }"
    @change="handleFilterChange"
    @reset="handleFiltersReset"
  />

  <div
    v-if="searchQuery || hasActiveSearchFilters(filters)"
    class="min-h-[400px]"
  >
    <div class="space-y-4">
      <div
        class="flex flex-wrap justify-between items-center gap-3 sm:space-x-4 w-full sm:w-auto ml-auto"
      >
        <h2 class="text-xl sm:text-2xl font-bold text-gray-800 select-none">
          {{ $t('home.searchResultsTitle.dictionary') }}
        </h2>
      </div>

      <div v-if="isLoading" class="flex justify-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
      </div>
      <template v-else>
        <div v-if="!isLoading && !error" class="grid gap-4 mb-6">
          <!-- Decomposition display -->
          <AlertComponent
            v-if="decomposition?.length"
            type="tip"
            :label="$t('components.dictionaryEntries.decomposition')"
          >
            <div class="inline-flex items-center gap-1">
              <template v-for="(word, index) in decomposition" :key="word">
                <h2
                  class="text-base font-semibold text-blue-700 hover:text-blue-800 hover:underline truncate flex-shrink-0"
                >
                  <RouterLink
                    :to="{
                      path: `/valsi/${word.replace(/ /g, '_')}`,
                      query: { langid: definitions[0]?.langid },
                    }"
                  >
                    {{ word }}
                  </RouterLink>
                </h2>
                <span v-if="index < decomposition.length - 1" class="text-aqua-500">+</span>
              </template>
            </div>
          </AlertComponent>
          <PhraseSplit
            :phrase="searchQuery"
            :selected-languages="filters.selectedLanguages"
            :source-lang-id="filters.source_langid"
            :languages="languages"
          />
          <template
            v-if="filters.selectedCollections?.length || filters.usernames?.length"
          >
            <div
              v-if="expandCollectionItemId"
              class="surface-definition-compact text-sm text-gray-700 flex flex-wrap items-center justify-between gap-2"
            >
              <p class="font-medium text-gray-800">{{ $t('home.collectionItemMatches') }}</p>
              <Button variant="cancel" type="button" @click="clearCollectionItemExpand">
                {{ $t('home.backToSearchResults') }}
              </Button>
            </div>
            <DefinitionCard
              v-for="def in collectionMatches"
              :key="
                def.collection_id != null && def.item_id != null
                  ? `ci-${def.collection_id}-${def.item_id}`
                  : `d-${def.definitionid}`
              "
              :definition="def"
              :languages="languages"
              :disable-toolbar="true"
              :disable-owner-only-lock="true"
              :show-vote-buttons="false"
              :collections="collections"
              :collection-id="def.collection_id"
              :item-id="def.item_id"
              :expand-on-click="!expandCollectionItemId && !!def.item_id"
              @collection-updated="setCollections($event)"
              @card-activate="expandCollectionItem(def)"
            />
            <div class="surface-definition-compact text-sm text-gray-700">
              <p class="font-medium text-gray-800">
                {{ $t('home.globalDictionaryBelow') }}
              </p>
            </div>
          </template>
          <!-- Global dictionary (always after filtered authors∪collections hits) -->
          <DefinitionCardSimple
            v-for="def in dictionaryDefinitions"
            :key="def.definitionid"
            :definition="def"
            :languages="languages"
            :show-word-type="true"
          />
        </div>

        <div
          v-if="!isLoading && dictionaryDefinitions.length === 0 && collectionMatches.length === 0"
          class="text-center py-8 text-gray-600"
        >
          {{ $t('components.dictionaryEntries.noEntries') }}
        </div>

        <div v-if="error" class="text-center py-8 text-red-600">{{ error }}</div>
      </template>
    </div>
  </div>
  <!-- PaginationComponent -->
  <div
    v-if="searchQuery || hasActiveSearchFilters(filters)"
  >
    <PaginationComponent
      :current-page="currentPage"
      :total-pages="totalPages"
      :total="total"
      :per-page="10"
      class="w-full"
      @prev="prevPage"
      @next="nextPage"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { nextTick } from 'vue'

import { Button } from '@packages/ui'
import { fastSearchDefinitions, getLanguages } from '@/api'
import CombinedFilters from '@/components/CombinedFilters.vue'
import DefinitionCard from '@/components/DefinitionCard.vue'
import DefinitionCardSimple from '@/components/DefinitionCardSimple.vue'
import PhraseSplit from '@/components/PhraseSplit.vue'
import AlertComponent from '@/components/AlertComponent.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import SearchForm from '@/components/SearchForm.vue'
import CombinedFiltersSkeleton from '@/components/skeletons/CombinedFiltersSkeleton.vue'
import SearchFormSkeleton from '@/components/skeletons/SearchFormSkeleton.vue'
import { useCollectionsCache } from '@/composables/useCollectionsCache'
import { useLanguageSelection } from '@/composables/useLanguageSelection'
import { useSeoHead } from '@/composables/useSeoHead'
import { SearchQueue } from '@/utils/searchQueue'
import {
  applyCombinedFiltersFromQuery,
  combinedFiltersFromQuery,
  combinedFiltersToQuery,
  commitHomeQuery,
  compactQuery,
  hasActiveSearchFilters,
  resolveHomeQuery,
  queryStr,
} from '@/utils/routeQuery'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'
import {
  mapCollectionItemToDefinition,
  type CollectionDefinitionCard,
  type CollectionSearchItem,
} from '@/utils/mapCollectionItemToDefinition'

const router = useRouter()
const route = useRoute()
const { collections, setCollections } = useCollectionsCache()
const expandCollectionItemId = computed(() => {
  const n = parseInt(queryStr(route.query.expand_ci), 10)
  return Number.isFinite(n) && n > 0 ? n : null
})
const expandCollectionItem = (def: { item_id?: number }) => {
  if (!def.item_id) return
  router.push({
    query: commitHomeQuery({
      ...route.query,
      expand_ci: String(def.item_id),
    }),
  })
}
const clearCollectionItemExpand = () => {
  router.push({
    query: commitHomeQuery({
      ...route.query,
      expand_ci: undefined,
    }),
  })
}
const { getInitialLanguages, saveLanguages } = useLanguageSelection()
const hydratedHomeQuery = resolveHomeQuery(route.query)

type FastSearchDefinitionRow = {
  definitionid: number
  langid?: number
}

// State
const definitions = ref<FastSearchDefinitionRow[]>([])
const collectionMatches = ref<CollectionDefinitionCard[]>([])
const decomposition = ref<string[]>([])
const total = ref(0)
const currentPage = ref(parseInt(queryStr(route.query.page), 10) || 1)
const totalPages = ref(1)
const initialized = ref(false)

/** Global rows with priority (authors∪collections) definition ids removed. */
const dictionaryDefinitions = computed(() => {
  const seen = new Set(
    collectionMatches.value
      .map((d) => d.definitionid)
      .filter((id): id is number => typeof id === 'number' && id > 0)
  )
  if (!seen.size) return definitions.value
  return definitions.value.filter((d) => !d.definitionid || !seen.has(d.definitionid))
})

// Get search query from localStorage or use default
const getInitialSearchQuery = (): string => {
  if (typeof window === 'undefined') return ''
  if (route.query.q !== undefined) {
    return normalizeSearchQuery(queryStr(route.query.q)) as string
  }
  return normalizeSearchQuery(hydratedHomeQuery.q || '') as string
}

const searchQuery = ref(getInitialSearchQuery())
const isLoading = ref(true)
const isInitialLoading = ref(true)
const error = ref(null)
const searchFormRef = ref(null)

const pageTitle = computed(() => searchQuery.value?.trim() || 'Fast Search')
useSeoHead({ title: pageTitle, pathWithoutLocale: '/fast-search' })

// Filter state
const languages = ref([])
const filters = ref(combinedFiltersFromQuery(hydratedHomeQuery))

// Search queue to prevent race conditions
const definitionsSearchQueue = new SearchQueue()

// Fetch definitions using fast search
const fetchDefinitions = async (page: number, search = '') => {
  isLoading.value = true
  error.value = null

  const { requestId, signal } = definitionsSearchQueue.createRequest()

  try {
    const collectionIds = (filters.value.selectedCollections || []).filter(
      (n) => Number.isFinite(n) && n > 0
    )
    const hasAuthors = !!filters.value.usernames?.length
    const usePriorityThenGlobal = collectionIds.length > 0 || hasAuthors

    const params: Record<string, unknown> = {
      page,
      per_page: 10,
      search: String(search ?? '').trim() || undefined,
      username: filters.value.usernames?.length
        ? filters.value.usernames.join(',')
        : undefined,
      exclude_usernames: filters.value.excludeUsernames?.length
        ? filters.value.excludeUsernames.join(',')
        : undefined,
      ...(filters.value.selectedLanguages.length > 0 && {
        languages: filters.value.selectedLanguages.join(','),
      }),
    }

    if (!filters.value.selmaho) {
      params.word_type = filters.value.word_type || undefined
    }

    if (filters.value.source_langid && filters.value.source_langid !== 1) {
      params.source_langid = filters.value.source_langid
    }

    if (filters.value.selmaho) {
      params.selmaho = filters.value.selmaho
    }

    if (filters.value.searchInPhrases !== undefined && filters.value.searchInPhrases !== null) {
      params.search_in_phrases = filters.value.searchInPhrases
    }

    if (usePriorityThenGlobal) {
      params.include_global_group = true
      if (collectionIds.length) {
        params.collection_ids = collectionIds.join(',')
      }
    }
    if (expandCollectionItemId.value) {
      params.expand_collection_item = expandCollectionItemId.value
    }

    const response = await fastSearchDefinitions(params, signal)

    // Only process if this is still the latest request
    if (!definitionsSearchQueue.shouldProcess(requestId)) {
      return
    }

    const collectionHits = (
      (response.data.filtered_collection_items ?? []) as CollectionSearchItem[]
    )
      .map((item) => mapCollectionItemToDefinition(item))
      .filter((d): d is CollectionDefinitionCard => d != null)
    const authorHits = (response.data.filtered_definitions ?? []) as CollectionDefinitionCard[]
    collectionMatches.value = expandCollectionItemId.value
      ? collectionHits
      : [...collectionHits, ...authorHits]
    definitions.value = response.data.definitions
    total.value = response.data.total
    currentPage.value = page
    totalPages.value = Math.ceil(response.data.total / 10)
    decomposition.value = response.data.decomposition || []
  } catch (e) {
    // Ignore abort errors
    if (e.name === 'AbortError' || e.code === 'ERR_CANCELED' || e.message?.includes('canceled')) {
      return
    }

    // Only show errors for the latest request
    if (definitionsSearchQueue.shouldProcess(requestId)) {
      error.value = e.response?.data?.error || 'Failed to load definitions'
      console.error('Error fetching definitions:', e)
    }
  } finally {
    // Only update loading state if this is still the latest request
    if (definitionsSearchQueue.shouldProcess(requestId)) {
      isLoading.value = false
    }
  }
}

const fetchData = async () => {
  if (!searchQuery.value.trim() && !hasActiveSearchFilters(filters.value)) {
    collectionMatches.value = []
    isLoading.value = false
    return
  }

  isLoading.value = true
  await fetchDefinitions(currentPage.value, searchQuery.value)
}

// Filter handling
const handleFilterChange = () => {
  updateUrlWithFilters()
}

const handleFiltersReset = async () => {
  filters.value = {
    ...combinedFiltersFromQuery({}),
    selectedLanguages: [],
    searchInPhrases: true,
  }
  currentPage.value = 1
  searchQuery.value = ''
  updateUrlWithFilters()
}

const updateUrlWithFilters = () => {
  router.push({
    query: commitHomeQuery({
      ...route.query,
      q: searchQuery.value || undefined,
      ...combinedFiltersToQuery(filters.value),
      page: undefined,
      expand_ci: undefined,
    }),
  })
}

// Search handling
const performSearch = ({ query, mode }: { query: string; mode: string }) => {
  const updateParams = commitHomeQuery({
    ...route.query,
    q: query || undefined,
    mode: mode !== 'dictionary' ? mode : undefined, // Keep mode if it's not dictionary
    page: undefined, // Always reset to page 1 for a new search
    expand_ci: undefined,
    ...combinedFiltersToQuery(filters.value),
  })

  if (mode !== 'dictionary') {
    // If we're on FastSearch and mode is semantic, waves, mail, etc., redirect to Home
    const currentLocale = route.path.split('/')[1] || 'en'
    router.push({ path: `/${currentLocale}`, query: updateParams })
    return
  }

  const normalizedQuery = normalizeSearchQuery(query) as string
  searchQuery.value = normalizedQuery
  if (typeof window !== 'undefined') {
    localStorage.setItem('searchQuery', normalizedQuery)
  }

  router.push({ query: updateParams })
}

const prevPage = () => {
  if (currentPage.value > 1) {
    router.push({
      query: {
        ...route.query,
        page: currentPage.value - 1,
      },
    })
  }
}

const nextPage = () => {
  if (currentPage.value < totalPages.value) {
    router.push({
      query: {
        ...route.query,
        page: currentPage.value + 1,
      },
    })
  }
}

// URL sync
const syncFromRoute = () => {
  const query = route.query

  if (query.q !== undefined) {
    const normalized = normalizeSearchQuery(queryStr(query.q)) as string
    searchQuery.value = normalized
    if (typeof window !== 'undefined') localStorage.setItem('searchQuery', normalized)
  }

  if (query.page !== undefined) {
    currentPage.value = parseInt(queryStr(query.page), 10) || 1
  }

  Object.assign(filters.value, applyCombinedFiltersFromQuery(filters.value, query))
}

const handleKeyDown = (event: KeyboardEvent) => {
  if (
    event.key === '/' &&
    document.activeElement &&
    !['INPUT', 'TEXTAREA'].includes(document.activeElement.tagName)
  ) {
    event.preventDefault()
    searchFormRef.value?.$refs.searchInput?.focus()
  }
}

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown)
  try {
    const languagesResponse = await getLanguages()
    const initialLangs = getInitialLanguages(route, languagesResponse.data)
    filters.value.selectedLanguages = initialLangs
    languages.value = languagesResponse.data

    const queryToPush = commitHomeQuery({
      ...route.query,
      ...resolveHomeQuery(route.query),
      q: searchQuery.value || undefined,
      ...combinedFiltersToQuery(filters.value),
    })
    const currentCompact = compactQuery({ ...route.query })
    if (JSON.stringify(currentCompact) !== JSON.stringify(queryToPush)) {
      router.push({ query: queryToPush })
    }
    isInitialLoading.value = false
    initialized.value = true
  } catch (e) {
    console.error('Error loading initial data:', e)
    isInitialLoading.value = false
  } finally {
    isInitialLoading.value = false

    if (
      route.name === 'FastSearch' ||
      (typeof route.name === 'string' && route.name.startsWith('FastSearch-'))
    ) {
      await nextTick()
      if (searchFormRef.value && !isInitialLoading.value) {
        searchFormRef.value.focusInput()
      }
    }
  }
})

watch(
  () => filters.value.selectedLanguages,
  (newLanguages) => {
    if (newLanguages.length > 0) {
      saveLanguages(newLanguages)
    }
  },
  { deep: true }
)

watch(
  () => route.query,
  async (newQuery, oldQuery) => {
    const relevantParamsChanged =
      newQuery.q !== oldQuery?.q ||
      newQuery.page !== oldQuery?.page ||
      newQuery.langs !== oldQuery?.langs ||
      newQuery.collections !== oldQuery?.collections ||
      newQuery.selmaho !== oldQuery?.selmaho ||
      newQuery.username !== oldQuery?.username ||
      newQuery.exclude_usernames !== oldQuery?.exclude_usernames ||
      newQuery.word_type !== oldQuery?.word_type ||
      newQuery.source_langid !== oldQuery?.source_langid ||
      newQuery.searchInPhrases !== oldQuery?.searchInPhrases ||
      newQuery.expand_ci !== oldQuery?.expand_ci

    currentPage.value = parseInt(queryStr(newQuery.page), 10) || 1

    if (newQuery.isExpanded !== oldQuery?.isExpanded) {
      filters.value.isExpanded = queryStr(newQuery.isExpanded) === 'true'
    }

    if (relevantParamsChanged) {
      syncFromRoute()
      await fetchData()

      if (
        (route.name === 'FastSearch' ||
          (typeof route.name === 'string' && route.name.startsWith('FastSearch-'))) &&
        searchFormRef.value &&
        !isInitialLoading.value
      ) {
        await nextTick()
        searchFormRef.value.focusInput()
      }
    }
  },
  { deep: true, immediate: true }
)
</script>
