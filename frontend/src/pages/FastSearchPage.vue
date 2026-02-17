<template>
  <!-- Search and Filter Section -->
  <div class="space-y-4 mt-4 sm:mt-6">
    <!-- Skeletons -->
    <SearchFormSkeleton v-if="isInitialLoading" />
    <CombinedFiltersSkeleton v-if="isInitialLoading" />

    <!-- Actual Components (hidden while loading) -->
    <SearchForm
      ref="searchFormRef"
      :initial-query="searchQuery"
      :initial-mode="'dictionary'"
      class="w-full transition-opacity duration-300"
      :class="{ 'opacity-0 pointer-events-none h-0 overflow-hidden': isInitialLoading }"
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
  </div>

  <div
    v-if="searchQuery || filters.selmaho || filters.username || filters.word_type"
    class="min-h-[400px] mt-4 sm:mt-6"
  >
    <div class="space-y-4">
      <div class="flex flex-wrap justify-between items-center gap-3 sm:space-x-4 w-full sm:w-auto ml-auto">
        <h2 class="text-xl sm:text-2xl font-bold text-gray-800 select-none">
          {{ $t('home.searchResultsTitle.dictionary') }}
        </h2>
      </div>

      <div
        v-if="isLoading"
        class="flex justify-center py-8"
      >
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
      </div>

      <template v-else>
        <div
          v-if="!isLoading && !error"
          class="grid gap-4 mb-6"
        >
          <!-- Decomposition display -->
          <AlertComponent
            v-if="decomposition?.length"
            type="tip"
            :label="$t('components.dictionaryEntries.decomposition')"
          >
            <div class="inline-flex items-center gap-1">
              <template
                v-for="(word, index) in decomposition"
                :key="word"
              >
                <h2
                  class="text-base font-semibold text-blue-700 hover:text-blue-800 hover:underline truncate flex-shrink-0"
                >
                  <RouterLink :to="{ path: `/valsi/${word.replace(/ /g, '_')}`, query: { langid: definitions[0]?.langid } }">
                    {{ word }}
                  </RouterLink>
                </h2>
                <span
                  v-if="index < decomposition.length - 1"
                  class="text-aqua-500"
                >+</span>
              </template>
            </div>
          </AlertComponent>

          <!-- Definition Cards -->
          <DefinitionCardSimple
            v-for="def in definitions"
            :key="def.definitionid"
            :definition="def"
            :languages="languages"
            :show-word-type="true"
          />
        </div>

        <div
          v-if="!isLoading && definitions.length === 0"
          class="text-center py-8 text-gray-600"
        >
          {{ $t('components.dictionaryEntries.noEntries') }}
        </div>

        <div
          v-if="error"
          class="text-center py-8 text-red-600"
        >
          {{ error }}
        </div>
      </template>
    </div>
  </div>

  <!-- PaginationComponent -->
  <div
    v-if="searchQuery || filters.selmaho || filters.username || filters.word_type"
    class="mt-6"
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

<script setup>
import { ref, onMounted, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { nextTick } from 'vue';

import {
  fastSearchDefinitions,
  getLanguages,
} from '@/api';
import CombinedFilters from '@/components/CombinedFilters.vue';
import DefinitionCardSimple from '@/components/DefinitionCardSimple.vue';
import AlertComponent from '@/components/AlertComponent.vue';
import PaginationComponent from '@/components/PaginationComponent.vue';
import SearchForm from '@/components/SearchForm.vue';
import CombinedFiltersSkeleton from '@/components/skeletons/CombinedFiltersSkeleton.vue';
import SearchFormSkeleton from '@/components/skeletons/SearchFormSkeleton.vue';
import { useLanguageSelection } from '@/composables/useLanguageSelection';
import { useSeoHead } from '@/composables/useSeoHead';
import { SearchQueue } from '@/utils/searchQueue';
import { normalizeSearchQuery } from '@/utils/searchQueryUtils';

const router = useRouter();
const route = useRoute();
const { getInitialLanguages, saveLanguages } = useLanguageSelection();

// State
const definitions = ref([]);
const decomposition = ref([]);
const total = ref(0);
const currentPage = ref(parseInt(route.query.page) || 1);
const totalPages = ref(1);
const initialized = ref(false);

// Get search query from localStorage or use default
const getInitialSearchQuery = () => {
  if (typeof window === 'undefined') return '';
  return normalizeSearchQuery(route.query.q || '');
};

const searchQuery = ref(getInitialSearchQuery());
const isLoading = ref(true);
const isInitialLoading = ref(true);
const error = ref(null);
const searchFormRef = ref(null);

const { t, locale } = useI18n();
useSeoHead({ title: searchQuery.value || 'Fast Search', locale: locale.value });

// Filter state
const languages = ref([]);
const filters = ref({
  selmaho: '',
  username: '',
  word_type: null,
  isExpanded: false,
  selectedLanguages: [],
  source_langid: 1,
});

// Search queue to prevent race conditions
const definitionsSearchQueue = new SearchQueue();

// Fetch definitions using fast search
const fetchDefinitions = async (page, search = '') => {
  isLoading.value = true;
  error.value = null;

  const { requestId, signal } = definitionsSearchQueue.createRequest();

  try {
    const params = {
      page,
      per_page: 10,
      search: search,
      username: filters.value.username || undefined,
      ...(filters.value.selectedLanguages.length > 0 && {
        languages: filters.value.selectedLanguages.join(','),
      }),
    };

    if (!filters.value.selmaho) {
      params.word_type = filters.value.word_type || undefined;
    }

    if (filters.value.source_langid && filters.value.source_langid !== 1) {
      params.source_langid = filters.value.source_langid;
    }

    if (filters.value.selmaho) {
      params.selmaho = filters.value.selmaho;
    }

    const response = await fastSearchDefinitions(params, signal);

    // Only process if this is still the latest request
    if (!definitionsSearchQueue.shouldProcess(requestId)) {
      return;
    }

    definitions.value = response.data.definitions;
    total.value = response.data.total;
    currentPage.value = page;
    totalPages.value = Math.ceil(response.data.total / 10);
    decomposition.value = response.data.decomposition || [];
  } catch (e) {
    // Ignore abort errors
    if (e.name === 'AbortError' || e.code === 'ERR_CANCELED' || e.message?.includes('canceled')) {
      return;
    }

    // Only show errors for the latest request
    if (definitionsSearchQueue.shouldProcess(requestId)) {
      error.value = e.response?.data?.error || 'Failed to load definitions';
      console.error('Error fetching definitions:', e);
    }
  } finally {
    // Only update loading state if this is still the latest request
    if (definitionsSearchQueue.shouldProcess(requestId)) {
      isLoading.value = false;
    }
  }
};

const fetchData = async () => {
  if (
    !searchQuery.value.trim() &&
    !filters.value.selmaho &&
    !filters.value.username &&
    !filters.value.word_type
  ) {
    isLoading.value = false;
    return;
  }

  isLoading.value = true;
  await fetchDefinitions(currentPage.value, searchQuery.value);
};

// Filter handling
const handleFilterChange = () => {
  updateUrlWithFilters();
};

const handleFiltersReset = async () => {
  filters.value = {
    selmaho: '',
    username: '',
    isExpanded: false,
    selectedLanguages: [],
    word_type: '',
    source_langid: 1,
  };
  currentPage.value = 1;
  searchQuery.value = '';
  updateUrlWithFilters();
};

const updateUrlWithFilters = () => {
  router.push({
    query: {
      ...route.query,
      q: searchQuery.value || undefined,
      langs:
        filters.value.selectedLanguages.length > 0
          ? filters.value.selectedLanguages.join(',')
          : undefined,
      selmaho: filters.value.selmaho || undefined,
      username: filters.value.username || undefined,
      word_type: filters.value.word_type || undefined,
      source_langid: filters.value.source_langid !== 1 ? filters.value.source_langid : undefined,
    },
  });
};

// Search handling
const performSearch = ({ query }) => {
  const updateParams = {
    ...route.query,
    q: query || undefined,
    page: undefined, // Always reset to page 1 for a new search
    langs:
      filters.value.selectedLanguages && filters.value.selectedLanguages.length > 0
        ? filters.value.selectedLanguages.join(',')
        : undefined,
    selmaho: filters.value.selmaho || undefined,
    username: filters.value.username || undefined,
    word_type: filters.value.word_type || undefined,
  };

  const normalizedQuery = normalizeSearchQuery(query);
  searchQuery.value = normalizedQuery;
  if (typeof window !== 'undefined') {
    localStorage.setItem('searchQuery', normalizedQuery);
  }

  router.push({ query: updateParams });
};

const prevPage = () => {
  if (currentPage.value > 1) {
    router.push({
      query: {
        ...route.query,
        page: currentPage.value - 1,
      },
    });
  }
};

const nextPage = () => {
  if (currentPage.value < totalPages.value) {
    router.push({
      query: {
        ...route.query,
        page: currentPage.value + 1,
      },
    });
  }
};

// URL sync
const syncFromRoute = () => {
  const query = route.query;

  if (query.q !== undefined) {
    const normalized = normalizeSearchQuery(query.q);
    searchQuery.value = normalized;
    if (typeof window !== 'undefined') localStorage.setItem('searchQuery', normalized);
  }

  if (query.page !== undefined) {
    currentPage.value = parseInt(query.page) || 1;
  }

  // Sync filters from URL
  if (query.langs !== undefined) {
    filters.value.selectedLanguages = query.langs.split(',').map(Number);
  }

  if (query.selmaho !== undefined) {
    filters.value.selmaho = query.selmaho;
  }

  if (query.username !== undefined) {
    filters.value.username = query.username;
  }

  if (query.word_type !== undefined) {
    filters.value.word_type = query.word_type ? Number(query.word_type) : null;
  }

  if (query.source_langid !== undefined) {
    filters.value.source_langid = parseInt(query.source_langid) || 1;
  } else {
    filters.value.source_langid = 1;
  }
};

const handleKeyDown = (event) => {
  if (event.key === '/' && !['INPUT', 'TEXTAREA'].includes(document.activeElement.tagName)) {
    event.preventDefault();
    searchFormRef.value?.$refs.searchInput?.focus();
  }
};

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  try {
    const languagesResponse = await getLanguages();
    languages.value = languagesResponse.data;

    const initialLangs = getInitialLanguages(route, languages.value);
    filters.value.selectedLanguages = initialLangs;

    const queryToPush = { ...route.query };
    let pushNeeded = false;

    if (searchQuery.value && route.query.q !== searchQuery.value) {
      queryToPush.q = searchQuery.value;
      pushNeeded = true;
    } else if (!searchQuery.value && route.query.q === undefined) {
      queryToPush.q = undefined;
    }

    const targetLangs =
      filters.value.selectedLanguages.length > 0
        ? filters.value.selectedLanguages.join(',')
        : undefined;
    if (route.query.langs !== targetLangs) {
      queryToPush.langs = targetLangs;
      pushNeeded = true;
    }

    Object.keys(queryToPush).forEach(
      (key) => queryToPush[key] === undefined && delete queryToPush[key],
    );

    if (pushNeeded) {
      router.push({ query: queryToPush });
    }
    isInitialLoading.value = false;
    initialized.value = true;
  } catch (e) {
    console.error('Error loading initial data:', e);
    isInitialLoading.value = false;
  } finally {
    isInitialLoading.value = false;

    if (route.name === 'FastSearch' || route.name?.startsWith('FastSearch-')) {
      await nextTick();
      if (searchFormRef.value && !isInitialLoading.value) {
        searchFormRef.value.focusInput();
      }
    }
  }
});

watch(
  () => filters.value.selectedLanguages,
  (newLanguages) => {
    if (newLanguages.length > 0) {
      saveLanguages(newLanguages);
    }
  },
  { deep: true },
);

watch(
  () => route.query,
  async (newQuery, oldQuery) => {
    const relevantParamsChanged =
      newQuery.q !== oldQuery?.q ||
      newQuery.page !== oldQuery?.page ||
      newQuery.langs !== oldQuery?.langs ||
      newQuery.selmaho !== oldQuery?.selmaho ||
      newQuery.username !== oldQuery?.username ||
      newQuery.word_type !== oldQuery?.word_type ||
      newQuery.source_langid !== oldQuery?.source_langid;

    currentPage.value = parseInt(newQuery.page) || 1;

    if (relevantParamsChanged) {
      syncFromRoute();
      await fetchData();

      if (
        (route.name === 'FastSearch' || route.name?.startsWith('FastSearch-')) &&
        searchFormRef.value &&
        !isInitialLoading.value
      ) {
        await nextTick();
        searchFormRef.value.focusInput();
      }
    }
  },
  { deep: true, immediate: true },
);
</script>
