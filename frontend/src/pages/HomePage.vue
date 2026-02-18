<template>
  <!-- Search and Filter Section -->
  <div class="space-y-4 mt-4 sm:mt-6">
    <!-- Skeletons -->
    <SearchFormSkeleton v-if="isInitialLoading" />
    <CombinedFiltersSkeleton v-if="isInitialLoading && (searchMode === 'dictionary' || searchMode === 'semantic')" />

    <!-- Actual Components (hidden while loading) -->
    <SearchForm
      ref="searchFormRef"
      :initial-query="searchQuery"
      :initial-mode="searchMode"
      :initial-group-by-thread="groupByThread"
      class="w-full transition-opacity duration-300"
      :class="{ 'opacity-0 pointer-events-none h-0 overflow-hidden': isInitialLoading }"
      @search="performSearch"
    />

    <CombinedFilters
      v-if="searchMode === 'dictionary' || searchMode === 'semantic'"
      v-model="filters"
      :languages="languages"
      class="w-full transition-opacity duration-300"
      :class="{ 'opacity-0 pointer-events-none h-0 overflow-hidden': isInitialLoading }"
      @change="handleFilterChange"
      @reset="handleFiltersReset"
    />
  </div>

  <div
    v-if="!searchQuery && !filters.selmaho && !filters.username && !filters.word_type"
    class="min-h-[400px] mt-4 sm:mt-6"
  >
    <div
      v-if="isLoadingTrending"
      class="flex justify-center py-8"
    >
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
    </div>
    <!-- Trending Comments -->
    <div
      v-else-if="trendingComments.length > 0"
      class="space-y-4"
    >
      <h2 class="text-xl sm:text-2xl font-bold text-gray-800 select-none">
        {{ $t('home.trendingComments') }}
      </h2>
      <div
        v-for="comment in trendingComments"
        :key="comment.comment_id"
        class="cursor-pointer"
        @click="
          router.push(
            `/comments?thread_id=${comment.thread_id}&comment_id=${comment.parent_id}&scroll_to=${comment.comment_id}&valsi_id=${comment.valsi_id}&definition_id=${comment.definition_id || 0}`
          )
        "
      >
        <CommentItem
          :comment="comment"
          :reply-enabled="true"
          @reply="handleReply"
        />
      </div>
    </div>

    <!-- Recent Changes -->
    <div
      v-else-if="recentChanges.length > 0"
      class="space-y-4"
    >
      <div
        class="flex flex-col md:flex-row justify-between items-start sm:items-center gap-3 sm:space-x-2 w-full sm:w-auto ml-auto"
      >
        <h2 class="text-xl sm:text-2xl font-bold text-gray-800 select-none">
          {{ $t('home.recentChanges') }}
        </h2>
        <!-- <div class="flex flex-wrap items-start sm:items-center gap-3 sm:space-x-4 ml-auto">
          <div v-if="auth.isLoading" class="w-[120px] h-6 bg-gray-100 animate-pulse rounded-full"></div>
          <div v-else class="flex flex-wrap items-start sm:items-center gap-3 sm:space-x-4 ml-auto">
            <IconButton v-if="auth.state.isLoggedIn && decodedRole !== 'Unconfirmed'" label="Definition"
              buttonClasses="btn-aqua-emerald" @click="router.push('/valsi/add')" />
            <IconButton v-if="auth.state.isLoggedIn" label="New Free Thread" buttonClasses="btn-aqua-rose"
              @click="handleNewFreeComment" />
          </div>
        </div> -->
      </div>
      <div
        v-for="(group, index) in groupedChanges"
        :key="index"
        class="mb-8"
      >
        <h3 class="text-base font-semibold text-gray-700 mb-4 pt-4 border-t">
          {{ formatDate(group.date) }}
        </h3>
        <div class="space-y-3">
          <RecentChangeItem
            v-for="change in group.changes"
            :key="change.time"
            :change="change"
          />
        </div>
      </div>
    </div>
  </div>
  <div
    v-else
    ref="searchResultsRef"
    class="min-h-[400px] mt-4 sm:mt-6"
  >
    <div class="space-y-4">
      <div class="flex flex-wrap justify-between items-center gap-3 sm:space-x-4 w-full sm:w-auto ml-auto">
        <h2 class="text-xl sm:text-2xl font-bold text-gray-800 select-none">
          {{
            searchMode === 'dictionary'
              ? $t('home.searchResultsTitle.dictionary')
              : searchMode === 'semantic'
                ? $t('home.searchResultsTitle.semantic')
                : searchMode === 'muplis'
                  ? $t('home.searchResultsTitle.muplis')
                  : searchMode === 'comments'
                    ? $t('home.searchResultsTitle.comments')
                    : $t('home.searchResultsTitle.messages')
          }}
        </h2>

        <div
          v-if="auth.isLoading"
          class="flex flex-col sm:flex-row items-end sm:items-center gap-3 sm:space-x-4 ml-auto"
        >
          <!-- Skeleton loader shown while auth state loads -->
          <div class="w-[120px] h-6 bg-gray-100 animate-pulse rounded-full" />
        </div>

        <div
          v-else-if="searchMode === 'dictionary' || searchMode === 'semantic'"
          class="flex flex-col sm:flex-row items-end sm:items-center gap-3 sm:space-x-4 ml-auto"
        >
          <IconButton
            v-if="auth.state.isLoggedIn && decodedRole !== 'Unconfirmed'"
            :label="$t('home.addDefinition')"
            button-classes="btn-aqua-emerald"
            @click="router.push('/valsi/add')"
          />
        </div>

        <div
          v-else-if="searchMode === 'comments'"
          class="flex flex-col sm:flex-row items-end sm:items-center gap-3 sm:space-x-4 ml-auto"
        >
          <IconButton
            v-if="auth.state.isLoggedIn"
            :label="$t('home.newFreeThread')"
            button-classes="btn-aqua-emerald"
            @click="handleNewFreeComment"
          >
            <template #icon>
              <AudioWaveform class="h-4 w-4" />
            </template>
          </IconButton>
        </div>

        <div
          v-if="searchMode === 'comments' || searchMode === 'messages'"
          class="flex items-center gap-2"
        >
          <div class="relative">
            <select
              v-model="sortBy"
              class="input-field"
              @change="handleSortChange"
            >
              <option
                v-if="searchMode === 'comments'"
                value="time"
              >
                {{ $t('sort.time') }}
              </option>
              <option
                v-if="searchMode === 'comments'"
                value="reactions"
              >
                {{ $t('sort.reactions') }}
              </option>
              <option
                v-if="searchMode === 'comments'"
                value="replies"
              >
                {{ $t('sort.replies') }}
              </option>
              <option
                v-if="searchMode === 'messages'"
                value="rank"
              >
                {{ $t('sort.relevance') }}
              </option>
              <option
                v-if="searchMode === 'messages'"
                value="date"
              >
                {{ $t('sort.date') }}
              </option>
              <option
                v-if="searchMode === 'messages'"
                value="subject"
              >
                {{ $t('sort.subject') }}
              </option>
              <option
                v-if="searchMode === 'messages'"
                value="from"
              >
                {{ $t('sort.sender') }}
              </option>
            </select>
          </div>
          <button
            class="btn-empty h-8"
            :title="sortOrder === 'asc' ? $t('sort.ascending') : $t('sort.descending')"
            @click="toggleSortOrder"
          >
            <ChevronUp
              v-if="sortOrder === 'asc'"
              class="h-5 w-5"
            />
            <ChevronDown
              v-else
              class="h-5 w-5"
            />
            <span>{{ sortOrder === 'asc' ? $t('sort.asc') : $t('sort.desc') }}</span>
          </button>
          <!-- Group by Thread Toggle -->
          <div v-if="searchMode === 'messages'" class="flex items-center space-x-2">
            <input
              id="groupByThread"
              type="checkbox"
              v-model="groupByThread"
              class="checkbox-toggle">
            <label for="groupByThread" class="text-sm text-gray-700 whitespace-nowrap">{{ $t('searchForm.modes.groupByThread') }}</label>
          </div>
        </div>
      </div>
      <div
        v-if="isLoading"
        class="flex justify-center py-8"
      >
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
      </div>
      <template v-else>
        <DictionaryEntries
          v-if="searchMode === 'dictionary' || searchMode === 'semantic'"
          :definitions="definitions"
          :is-loading="isLoading"
          :error="error"
          :languages="languages"
          :show-scores="searchMode === 'semantic'"
          :collections="collections"
          :decomposition="decomposition || []"
          @collection-updated="collections = $event"
        />
        <MessageList
          v-else-if="searchMode === 'messages'"
          :messages="messages"
          :show-content="includeContent"
          :search-term="searchQuery"
          :is-grouped-by-thread="groupByThread"
          @view-message="viewMessage"
          @view-thread-summary="handleViewThreadSummary"
        />
        <MuplisList
          v-else-if="searchMode === 'muplis'"
          :entries="muplisEntries"
          :search-term="searchQuery"
        />
        <div
          v-else-if="searchMode === 'comments'"
          class="space-y-4"
        >
          <div v-if="comments.length > 0">
            <div
              v-for="comment in comments"
              :key="comment.comment_id"
              class="cursor-pointer"
              @click="
                router.push(
                  `/comments?thread_id=${comment.thread_id}&comment_id=${comment.parent_id}&scroll_to=${comment.comment_id}&valsi_id=${comment.valsi_id}&definition_id=${comment.definition_id || 0}`
                )
              "
            >
              <CommentItem
                :comment="comment"
                :reply-enabled="true"
                :show-context="true"
                @reply="handleReply"
              />
            </div>
          </div>
          <div
            v-else
            class="text-center py-12 bg-blue-50 rounded-lg border border-blue-100"
          >
            <MessageSquare class="mx-auto h-12 w-12 text-blue-400" />
            <p class="mt-4 text-gray-600">
              {{ $t('home.noCommentsFound') }}
            </p>
          </div>
        </div>
      </template>
    </div>
  </div>

  <!-- PaginationComponent -->
  <div
    v-if="!(!searchQuery && !filters.selmaho && !filters.username && !filters.word_type)"
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
import { jwtDecode } from 'jwt-decode';
import { MessageSquare, ChevronDown, ChevronUp, AudioWaveform } from 'lucide-vue-next';
import { ref, onMounted, watch, computed, onBeforeUnmount, nextTick } from 'vue';
import { useRouter, useRoute } from 'vue-router';

import {
  api,
  searchDefinitions,
  fastSearchDefinitions,
  getLanguages,
  getTrendingComments,
  getRecentChanges,
  searchComments,
  getCollections,
  getBulkVotes
} from '@/api'
import CombinedFilters from '@/components/CombinedFilters.vue'
import CommentItem from '@/components/CommentItem.vue'
import DictionaryEntries from '@/components/DictionaryEntries.vue'
import IconButton from '@/components/icons/IconButton.vue'
import MuplisList from '@/components/MuplisList.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import RecentChangeItem from '@/components/RecentChangeItem.vue';
import SearchForm from '@/components/SearchForm.vue';
import CombinedFiltersSkeleton from '@/components/skeletons/CombinedFiltersSkeleton.vue';
import SearchFormSkeleton from '@/components/skeletons/SearchFormSkeleton.vue';
import { useAuth } from '@/composables/useAuth';
import { useLanguageSelection } from '@/composables/useLanguageSelection';
import { useSeoHead } from '@/composables/useSeoHead'
import { useI18n } from 'vue-i18n';
import { SearchQueue } from '@/utils/searchQueue';
import { normalizeSearchQuery } from '@/utils/searchQueryUtils';

import MessageList from './MessageList.vue'



defineEmits(['search', 'view-message', 'view-thread'])

const { getInitialLanguages, saveLanguages } = useLanguageSelection()
const collections = ref([])

const fetchCollections = async () => {
  try {
    const response = await getCollections()
    collections.value = response.data.collections
  } catch (error) {
    console.error('Error fetching collections:', error)
  }
}

const router = useRouter()
const route = useRoute()
const auth = useAuth()
const decodedToken = computed(() => {
  if (typeof window === 'undefined') return;
  const token = localStorage.getItem('accessToken')
  if (token) {
    try {
      return jwtDecode(token)
    } catch (e) {
      console.error('Error decoding token:', e)
      return null
    }
  }
  return null
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeyDown)
})

const decodedRole = computed(() => decodedToken.value?.role || '')

const props = defineProps({
  urlSearchQuery: {
    type: String,
    default: '',
  },
  urlSearchMode: {
    type: String,
    default: 'semantic',
  },
})

// State
const messages = ref([])
const muplisEntries = ref([])
const definitions = ref([])
const decomposition = ref([])
const comments = ref([])
const total = ref(0)
const currentPage = ref(parseInt(route.query.page) || 1)
const totalPages = ref(1)
const sortOrder = ref('desc')
const includeContent = ref(true)
const initialized = ref(false)

// Get search query from localStorage or use default
const getInitialSearchQuery = () => {
  if (typeof window === 'undefined') return;
  const storedQuery = localStorage.getItem('searchQuery')
  return normalizeSearchQuery(storedQuery || props.urlSearchQuery || '')
}

const getInitialGroupByThread = () => {
  if (typeof window === 'undefined') return false;
  const urlParam = route.query.group_by_thread
  if (urlParam !== undefined) {
    return urlParam === 'true'
  }
  return localStorage.getItem('mailSearch_groupByThread') === 'true'
}

const groupByThread = ref(getInitialGroupByThread())

const searchQuery = ref(getInitialSearchQuery())
// Get search mode from localStorage or use default
const getInitialSearchMode = () => {
  if (typeof window === 'undefined') return;
  const storedMode = localStorage.getItem('searchMode')
  return storedMode || props.urlSearchMode
}

const searchMode = ref(getInitialSearchMode())
const trendingComments = ref([])
const isLoading = ref(true); // Loading state for search results
const isInitialLoading = ref(true); // Loading state for initial component setup (languages etc.)
const isLoadingTrending = ref(false);
const error = ref(null);
const searchFormRef = ref(null);
const searchResultsRef = ref(null);

const { t, locale } = useI18n();

// Truncate search query for page title (max 50 characters)
const truncatedSearchQuery = computed(() => {
  if (!searchQuery.value) return null
  const maxLength = 50
  if (searchQuery.value.length <= maxLength) return searchQuery.value
  return searchQuery.value.substring(0, maxLength) + '...'
})

// Page title that reflects the search query
const pageTitle = computed(() => {
  if (truncatedSearchQuery.value) {
    return t('home.searchTitle', { query: truncatedSearchQuery.value })
  }
  return t('home.defaultTitle')
})

// Meta description for search snippets (avoids footer text being used)
const pageDescription = computed(() => {
  if (searchQuery.value?.trim()) {
    return t('home.searchMetaDescription', { query: truncatedSearchQuery.value || searchQuery.value })
  }
  return t('home.metaDescription')
})

useSeoHead({ title: pageTitle, description: pageDescription })

// Filter state
const languages = ref([])
const filters = ref({
  selmaho: '',
  username: '',
  word_type: route.query.word_type ? Number(route.query.word_type) : null,
  isExpanded: route.query.isExpanded === 'true',
  selectedLanguages: route.query.langs ? route.query.langs.split(',').map(Number) : [],
  source_langid: route.query.source_langid ? Number(route.query.source_langid) : 1,
  isSemantic: searchMode.value === 'semantic' || searchMode.value === 'dictionary' ? searchMode.value !== 'dictionary' : true,
})

// Search queues to prevent race conditions
const definitionsSearchQueue = new SearchQueue()
const commentsSearchQueue = new SearchQueue()
const messagesSearchQueue = new SearchQueue()
const muplisSearchQueue = new SearchQueue()

// Fetch corpus entries
const fetchDefinitions = async (page, search = '') => {
  isLoading.value = true
  error.value = null

  const { requestId, signal } = definitionsSearchQueue.createRequest()

  try {
    const params = {
      page,
      per_page: 10,
      search: search,
      include_comments: true,
      username: filters.value.username || undefined,
      ...(filters.value.selectedLanguages.length > 0 && {
        languages: filters.value.selectedLanguages.join(',')
      }),
    group_by_thread: groupByThread.value,
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

    let response
    const isSemantic = searchMode.value === 'semantic'
    
    if (auth.state.isLoggedIn || isSemantic) {
      response = await searchDefinitions({
        ...params,
        semantic: isSemantic,
      }, signal)
    } else {
      // Use fast search for non-logged in users (regular dictionary)
      const fastParams = { ...params }
      delete fastParams.include_comments
      
      response = await fastSearchDefinitions(fastParams, signal)
    }

    // Only process if this is still the latest request
    if (!definitionsSearchQueue.shouldProcess(requestId)) {
      return
    }

    definitions.value = response.data.definitions
    total.value = response.data.total
    currentPage.value = page
    totalPages.value = Math.ceil(response.data.total / 10)
    decomposition.value = response.data.decomposition

    // Get bulk votes for current user only if we have definitions
    if (auth.state.isLoggedIn && definitions.value.length > 0) {
      try {
        const definitionIds = definitions.value.map(d => d.definitionid)
        // Only fetch votes if we have IDs to check
        if (definitionIds.length > 0) {
          const votesResponse = await getBulkVotes({ definition_ids: definitionIds })
          const votesMap = votesResponse.data.votes

          // Check again before updating votes (in case another request completed)
          if (definitionsSearchQueue.shouldProcess(requestId)) {
            definitions.value = definitions.value.map(def => ({
              ...def,
              user_vote: votesMap[def.definitionid] || null
            }))
          }
        }
      } catch (e) {
        console.error('Error fetching votes:', e)
      }
    }
  } catch (e) {
    // Ignore abort errors
    if (e.name === 'AbortError' || e.code === 'ERR_CANCELED' || e.message?.includes('canceled')) {
      return
    }
    
    // Only show errors for the latest request
    if (definitionsSearchQueue.shouldProcess(requestId)) {
      error.value = e.response?.data?.error || 'Failed to load corpus entries'
      console.error('Error fetching valsi:', e)
    }
  } finally {
    // Only update loading state if this is still the latest request
    if (definitionsSearchQueue.shouldProcess(requestId)) {
      isLoading.value = false
    }
  }
}

const recentChanges = ref([])
const isLoadingChanges = ref(false)

// Cache key for recent changes
const RECENT_CHANGES_CACHE_KEY = 'recent_changes_cache'
const RECENT_CHANGES_CACHE_TTL = 5 * 60 * 1000 // 5 minutes in milliseconds

// Helper functions for caching
const getCachedRecentChanges = () => {
  if (typeof window === 'undefined') return null
  try {
    const cached = localStorage.getItem(RECENT_CHANGES_CACHE_KEY)
    if (!cached) return null
    
    const { data, timestamp } = JSON.parse(cached)
    const now = Date.now()
    
    // Check if cache is still valid (within TTL)
    if (now - timestamp < RECENT_CHANGES_CACHE_TTL) {
      return data
    }
    
    // Cache expired, remove it
    localStorage.removeItem(RECENT_CHANGES_CACHE_KEY)
    return null
  } catch (e) {
    console.error('Error reading cached recent changes:', e)
    return null
  }
}

const setCachedRecentChanges = (data) => {
  if (typeof window === 'undefined') return
  try {
    const cacheData = {
      data,
      timestamp: Date.now()
    }
    localStorage.setItem(RECENT_CHANGES_CACHE_KEY, JSON.stringify(cacheData))
  } catch (e) {
    console.error('Error caching recent changes:', e)
  }
}

const fetchTrendingAndChanges = async () => {
  isLoadingTrending.value = true
  
  // Try to load cached recent changes immediately for instant display
  const cachedChanges = getCachedRecentChanges()
  if (cachedChanges) {
    recentChanges.value = cachedChanges.slice(0, 20)
    isLoadingChanges.value = false
  }
  
  try {
    const response = await getTrendingComments({
      limit: 20,
      timespan: 'month',
    })
    trendingComments.value = response.data

    if (trendingComments.value.length === 0) {
      // Only show loading if we don't have cached data
      if (!cachedChanges) {
        isLoadingChanges.value = true
      }
      
      const response = await getRecentChanges({ days: 70 })
      const changes = response.data.changes.slice(0, 20)
      recentChanges.value = changes
      
      // Cache the fresh data
      setCachedRecentChanges(response.data.changes)
    }
  } catch (e) {
    console.error('Error fetching data:', e)
    // If we have cached data and fetch fails, keep using cached data
    if (!cachedChanges) {
      recentChanges.value = []
    }
  } finally {
    isLoadingTrending.value = false
    isLoadingChanges.value = false
  }
}

const groupedChanges = computed(() => {
  const groups = recentChanges.value.reduce((acc, change) => {
    const date = new Date(change.time * 1000).toLocaleDateString(locale.value)
    if (!acc[date]) {
      acc[date] = { date: new Date(change.time * 1000), changes: [] }
    }
    acc[date].changes.push(change)
    return acc
  }, {})
  return Object.values(groups).sort((a, b) => b.date - a.date)
})

const formatDate = (date) =>
  new Intl.DateTimeFormat(locale.value, {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  }).format(date)

// Generic data fetching for other modes
const sortBy = ref(searchMode.value === 'messages' ? 'rank' : 'time')

const toggleSortOrder = () => {
  sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  if (searchMode.value === 'comments') {
    fetchComments(currentPage.value, searchQuery.value)
  } else if (searchMode.value === 'messages') {
    fetchData()
  }
}

const handleSortChange = () => {
  currentPage.value = 1
  if (searchMode.value === 'comments') {
    fetchComments(1, searchQuery.value)
  } else if (searchMode.value === 'messages') {
    fetchData()
  }
}

const fetchComments = async (page, search = '') => {
  isLoading.value = true
  error.value = null

  const { requestId, signal } = commentsSearchQueue.createRequest()

  try {
    const response = await searchComments({
      page,
      per_page: 10,
      search,
      sort_by: sortBy.value,
      sort_order: sortOrder.value,
    }, signal)

    // Only process if this is still the latest request
    if (!commentsSearchQueue.shouldProcess(requestId)) {
      return
    }

    comments.value = response.data.comments
    total.value = response.data.total
    currentPage.value = page
    totalPages.value = Math.ceil(response.data.total / 10)
  } catch (e) {
    // Ignore abort errors
    if (e.name === 'AbortError' || e.code === 'ERR_CANCELED' || e.message?.includes('canceled')) {
      return
    }
    
    // Only show errors for the latest request
    if (commentsSearchQueue.shouldProcess(requestId)) {
      error.value = e.response?.data?.error || 'Failed to load comments'
      console.error('Error fetching comments:', e)
    }
  } finally {
    // Only update loading state if this is still the latest request
    if (commentsSearchQueue.shouldProcess(requestId)) {
      isLoading.value = false
    }
  }
}
const fetchData = async () => {
  if (
    !searchQuery.value.trim() &&
    !filters.value.selmaho &&
    !filters.value.username &&
    !filters.value.word_type
  ) {
    // Fetch trending/changes but ensure main loading is false
    await fetchTrendingAndChanges()
    isLoading.value = false // Ensure main loading is stopped
    return
  }

  // Set loading true only if we are actually fetching search results
  isLoading.value = true

  const params = {
    page: currentPage.value,
    per_page: 10,
    query: searchQuery.value.trim(),
    sort_by: sortBy.value,
    sort_order: sortOrder.value,    
    include_content: includeContent.value,
    group_by_thread: groupByThread.value, // Pass groupByThread here
  }

  try {
    if (searchMode.value === 'dictionary' || searchMode.value === 'semantic') {
      await fetchDefinitions(currentPage.value, searchQuery.value)
    } else if (searchMode.value === 'comments') {
      await fetchComments(currentPage.value, searchQuery.value)
    } else if (searchMode.value === 'messages') {
      const { requestId, signal } = messagesSearchQueue.createRequest()
      
      try {
        const response = await api.get('/mail/search', { params, signal })
        
        // Only process if this is still the latest request
        if (!messagesSearchQueue.shouldProcess(requestId)) {
          return
        }
        
        messages.value = response.data.messages
        totalPages.value = Math.ceil(response.data.total / response.data.per_page)
        isLoading.value = false
      } catch (error) {
        // Ignore abort errors
        if (error.name === 'AbortError' || error.code === 'ERR_CANCELED' || error.message?.includes('canceled')) {
          return
        }
        
        // Only show errors for the latest request
        if (messagesSearchQueue.shouldProcess(requestId)) {
          console.error('Error fetching data:', error)
          isLoading.value = false
        }
      }
    } else if (searchMode.value === 'muplis') {
      const { requestId, signal } = muplisSearchQueue.createRequest()
      
      try {
        const response = await api.get('/muplis/search', {
          params: {
            query: searchQuery.value,
            page: currentPage.value,
            per_page: 10,
          },
          signal,
        })
        
        // Only process if this is still the latest request
        if (!muplisSearchQueue.shouldProcess(requestId)) {
          return
        }
        
        muplisEntries.value = response.data.entries
        totalPages.value = Math.ceil(response.data.total / response.data.per_page)
        isLoading.value = false
      } catch (error) {
        // Ignore abort errors
        if (error.name === 'AbortError' || error.code === 'ERR_CANCELED' || error.message?.includes('canceled')) {
          return
        }
        
        // Only show errors for the latest request
        if (muplisSearchQueue.shouldProcess(requestId)) {
          console.error('Error fetching data:', error)
          isLoading.value = false
        }
      }
    }
  } catch (error) {
    console.error('Error fetching data:', error);
    // Ensure loading states are reset on error
    isLoading.value = false;
    isLoadingTrending.value = false;
  } finally {
    // isLoading is handled within specific fetch functions (fetchDefinitions, fetchComments)
    // or set directly in the try block for other modes.
  }
};

// Filter handling
const handleFilterChange = () => {
  // Update URL which will trigger the central fetchng of data through the route watcher
  updateUrlWithFilters()
}

const handleFiltersReset = async () => {
  filters.value = {
    selmaho: '',
    username: '',
    isExpanded: false,
    selectedLanguages: [],
    word_type: '',
    source_langid: 1,
  }
  currentPage.value = 1
  searchQuery.value = ''
  // if (searchFormRef.value) {
  //   searchFormRef.value.query = ''
  // }
  updateUrlWithFilters()
}

const updateUrlWithFilters = () => {
  router.push({
    query: {
      ...route.query,
      q: searchQuery.value || undefined,
      mode: searchMode.value,
      langs:
        filters.value.selectedLanguages.length > 0
          ? filters.value.selectedLanguages.join(',')
          : undefined,
      selmaho: filters.value.selmaho || undefined,
      username: filters.value.username || undefined,
      word_type: filters.value.word_type || undefined,
      source_langid: filters.value.source_langid !== 1 ? filters.value.source_langid : undefined,
      group_by_thread: groupByThread.value ? 'true' : undefined,
    },
  })
}

// Search handling

const performSearch = ({ query, mode }) => {
  // Use semantic mode if we're in dictionary mode and semantic search is enabled
  const effectiveMode = (mode === 'dictionary' && filters.value.isSemantic) ? 'semantic' : mode;

  // Reset to first page whenever search query or mode changes
  const updateParams = {
    ...route.query,
    q: query || undefined, // Use undefined if query is empty
    mode: effectiveMode,
    group_by_thread: groupByThread.value ? 'true' : undefined,
    page: undefined, // Always reset to page 1 for a new search
    langs: (filters.value.selectedLanguages && filters.value.selectedLanguages.length > 0) ? filters.value.selectedLanguages.join(',') : undefined,
    selmaho: filters.value.selmaho || undefined,
    username: filters.value.username || undefined,
    word_type: filters.value.word_type || undefined,
  }

  // Handle case where we might be on a localized Home-lang route
  const isHomeRoute = route.name === 'Home' || route.name?.startsWith('Home-');
  
  if (!isHomeRoute) {
    // If we're not on the home page, redirect to home with the search params
    const currentLocale = route.path.split('/')[1] || 'en';
    router.push({ path: `/${currentLocale}`, query: updateParams });
    return;
  }

  if (searchMode.value !== effectiveMode) {
    // Reset sortBy to default for the new mode
    sortBy.value = effectiveMode === 'messages' ? 'rank' : 'time'
  }

  // Update state before pushing to router to avoid duplicate fetches
  const normalizedQuery = normalizeSearchQuery(query)
  searchQuery.value = normalizedQuery
  searchMode.value = effectiveMode
  // groupByThread is handled by its own watcher now
  // Store mode and query in localStorage
  if (typeof window !== 'undefined') {
    localStorage.setItem('searchMode', effectiveMode)
    localStorage.setItem('searchQuery', normalizedQuery)
  }

  // Push to router but don't fetch data here - the route watcher will handle it
  router.push({ query: updateParams })
}

// Navigation handlers
const handleNewFreeComment = () => {
  router.push('/comments/new-thread')
}

const handleReply = (commentId) => {
  router.push({
    path: '/comments',
    query: {
      comment_id: commentId,
      valsi_id: props.valsiId || undefined,
      definition_id: props.definitionId || undefined,
    },
  })
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

const viewMessage = (messageId) => {
  // Extract locale from the current route path
  const currentLocale = route.path.split('/')[1] || 'en'; // Default to 'en' if locale is missing
  const routeName = `MessageDetail-${currentLocale}`;
  router.push({ name: routeName, params: { id: messageId } });
}

const handleViewThreadSummary = (subject) => {
  const currentLocale = route.path.split('/')[1] || 'en';
  const routeName = `ThreadView-${currentLocale}`;
  router.push({ name: routeName, params: { subject } });
};

// URL sync
const syncFromRoute = () => {
  // Get all params from URL
  const query = route.query

  // Only update values if they exist in URL
  if (query.q !== undefined) {
    const normalized = normalizeSearchQuery(query.q)
    searchQuery.value = normalized
    if (typeof window !== 'undefined') localStorage.setItem('searchQuery', normalized)
  }

  if (query.mode !== undefined) {
    searchMode.value = query.mode
    if (typeof window !== 'undefined') localStorage.setItem('searchMode', query.mode)
  }
  // groupByThread is now handled by its watcher and getInitialGroupByThread
  // if (query.group_by_thread !== undefined) {
  //   groupByThread.value = query.group_by_thread === 'true';
  //   if (typeof window !== 'undefined') localStorage.setItem('mailSearch_groupByThread', groupByThread.value.toString());
  // }

  if (query.page !== undefined) {
    currentPage.value = parseInt(query.page) || 1
  }

  // Sync filters from URL
  if (query.langs !== undefined) {
    filters.value.selectedLanguages = query.langs.split(',').map(Number)
  }

  if (query.selmaho !== undefined) {
    filters.value.selmaho = query.selmaho
  }

  if (query.username !== undefined) {
    filters.value.username = query.username
  }

  if (query.word_type !== undefined) {
    filters.value.word_type = query.word_type ? Number(query.word_type) : null
  }

  if (query.source_langid !== undefined) {
    filters.value.source_langid = parseInt(query.source_langid) || 1 // Default to 1 if invalid
  } else {
    filters.value.source_langid = 1 // Default if not present
  }

  // Sync isSemantic from searchMode which was synced from route mode above
  if (searchMode.value === 'semantic' || searchMode.value === 'dictionary') {
    filters.value.isSemantic = searchMode.value === 'semantic'
  }
}


const handleKeyDown = (event) => {
  // Check if / was pressed and no input/textarea is focused
  if (event.key === '/' && !['INPUT', 'TEXTAREA'].includes(document.activeElement.tagName)) {
    event.preventDefault()
    searchFormRef.value?.$refs.searchInput?.focus()
  }
}

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  try {
    const languagesResponse = await getLanguages();
    languages.value = languagesResponse.data;

    // Set initial languages from route or defaults
    const initialLangs = getInitialLanguages(route, languages.value);
    filters.value.selectedLanguages = initialLangs;
    // groupByThread is already initialized with getInitialGroupByThread

    // Initial data like languages is loaded, hide skeletons
    // isInitialLoading.value = false; // Moved down

    // Construct the target query parameters based on current state (from localStorage/defaults)
    const queryToPush = { ...route.query }; // Start with current URL query
    let pushNeeded = false;

    // Sync 'q' from localStorage/default to URL if different
    if (searchQuery.value && route.query.q !== searchQuery.value) {
      queryToPush.q = searchQuery.value;
      pushNeeded = true;
    } else if (!searchQuery.value && route.query.q === undefined) { // Only if URL.q is also undefined
      queryToPush.q = undefined;
    }

    // Sync 'mode' from localStorage/default to URL if different
    if (searchMode.value && route.query.mode !== searchMode.value) {
      queryToPush.mode = searchMode.value;
      pushNeeded = true;
    }
    // Sync 'group_by_thread' from localStorage/default to URL if different
    const targetGroupByThread = groupByThread.value ? 'true' : undefined;
    if (route.query.group_by_thread !== targetGroupByThread) {
      queryToPush.group_by_thread = targetGroupByThread;
      pushNeeded = true;
    }

    // Sync 'langs' from localStorage/default to URL if different
    const targetLangs = filters.value.selectedLanguages.length > 0 ? filters.value.selectedLanguages.join(',') : undefined;
    if (route.query.langs !== targetLangs) {
      queryToPush.langs = targetLangs;
      pushNeeded = true;
    }

    // Clean undefined values from queryToPush before pushing
    Object.keys(queryToPush).forEach(key => queryToPush[key] === undefined && delete queryToPush[key]);

    if (pushNeeded) {
      router.push({ query: queryToPush });
      // The route watcher will handle fetching data with the new URL.
    }
    isInitialLoading.value = false; // Skeletons can be hidden now.

    // Auth-dependent fetches (like collections) are handled by the auth state watcher.
    // Initial data fetching (search or trending) is handled by the immediate route query watcher.
  } catch (e) {
    console.error('Error loading initial data:', e);
    // Still hide skeletons even if there's an error loading languages,
    // as the components might still render partially or show an error state.
    isInitialLoading.value = false;
  } finally {
     // Ensure skeleton is hidden if try block finishes early or has issues not caught by catch
    isInitialLoading.value = false;

    // Focus search input if on home page
    if (route.name === 'Home' || route.name.startsWith('Home-')) {
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
      saveLanguages(newLanguages)
    }
  },
  { deep: true }
)

watch(
  () => filters.value.isSemantic,
  (newVal) => {
    if (searchMode.value === 'semantic' || searchMode.value === 'dictionary') {
      const newMode = newVal ? 'semantic' : 'dictionary'
      if (searchMode.value !== newMode) {
        searchMode.value = newMode
        updateUrlWithFilters()
      }
    }
  }
)

watch(groupByThread, (newVal, oldVal) => {
  if (newVal !== oldVal && searchMode.value === 'messages') {
    if (typeof window !== 'undefined') {
      localStorage.setItem('mailSearch_groupByThread', newVal.toString());
    }
    updateUrlWithFilters(); // This will trigger the route watcher
  }
});
watch(
  () => route.query,
  async (newQuery, oldQuery) => {
    const relevantParamsChanged =
      newQuery.q !== oldQuery?.q ||
      newQuery.mode !== oldQuery?.mode ||
      newQuery.page !== oldQuery?.page ||
      newQuery.langs !== oldQuery?.langs ||
      newQuery.selmaho !== oldQuery?.selmaho ||
      newQuery.username !== oldQuery?.username ||
      newQuery.word_type !== oldQuery?.word_type ||
      newQuery.source_langid !== oldQuery?.source_langid

    const groupByThreadChanged = newQuery.group_by_thread !== oldQuery?.group_by_thread;
    if (groupByThreadChanged) {
      groupByThread.value = newQuery.group_by_thread === 'true';
    }

    // Update currentPage based on the new query *before* fetching
    currentPage.value = parseInt(newQuery.page) || 1

    // Only fetch data if relevant query params changed
    if (relevantParamsChanged || groupByThreadChanged) {
      syncFromRoute() // Sync other state variables
      await fetchData() // Fetch data using the potentially updated currentPage

      // When page changed, scroll search results to top
      if (newQuery.page !== oldQuery?.page && searchResultsRef.value) {
        await nextTick();
        searchResultsRef.value.scrollIntoView({ block: 'start', behavior: 'instant' });
      }

      // Attempt to focus after data fetch if it's the home route and not initial load
      if ((route.name === 'Home' || route.name === 'Home-lang') && searchFormRef.value && !isInitialLoading.value) {
        await nextTick();
        searchFormRef.value.focusInput();
      }
    }
  },
  { deep: true, immediate: true }
);

watch(
  () => auth.state.isLoading,
  async (isLoadingAuth, wasLoadingAuth) => {
    // Only proceed if loading has completed (was loading and now is not)
    if (wasLoadingAuth && !isLoadingAuth) {
      // Auth state is now determined
      if (auth.state.isLoggedIn) {
        await fetchCollections();
      }
      // Always fetch data once auth is resolved and component is initialized
      // The route watcher's immediate run handles the very initial state sync.
      // This ensures data loads even if the route doesn't change after auth resolves.
      if (initialized.value) {
         await fetchData();
      }
    }
  }
)
</script>
