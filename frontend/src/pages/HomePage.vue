<template>
  <!-- Search and Filter Section -->
  <!-- Skeletons -->
  <SearchFormSkeleton v-if="isInitialLoading" />
  <CombinedFiltersSkeleton
    v-if="isInitialLoading && (searchMode === 'dictionary' || searchMode === 'semantic')"
  />
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

  <div v-if="showTrendingHome" class="min-h-[400px]">
    <div v-if="isLoadingTrending" class="flex justify-center py-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
    </div>
    <!-- Trending Comments -->
    <div v-if="trendingComments.length > 0" class="space-y-4">
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
        <CommentItem :comment="comment" :reply-enabled="true" @reply="handleReply" />
      </div>
    </div>
    <!-- Recent Changes -->
    <div v-if="recentChanges.length > 0" class="space-y-4 mt-8">
      <div
        class="flex flex-col md:flex-row justify-between items-start sm:items-center gap-3 sm:space-x-2 w-full sm:w-auto ml-auto"
      >
        <h2 class="text-xl sm:text-2xl font-bold text-gray-800 select-none">
          {{ $t('home.recentChanges') }}
        </h2>
      </div>

      <div v-for="(group, index) in groupedChanges" :key="index" class="mb-8">
        <h3 class="text-base font-semibold text-gray-700 mb-4 pt-4 border-t">
          {{ formatDate(group.date) }}
        </h3>

        <div class="space-y-3">
          <RecentChangeItem v-for="change in group.changes" :key="change.time" :change="change" />
        </div>
      </div>
    </div>
  </div>

  <div v-else ref="searchResultsRef" class="min-h-[400px]">
    <div class="space-y-4">
      <div
        class="flex flex-wrap justify-between items-center gap-3 sm:space-x-4 w-full sm:w-auto ml-auto"
      >
        <h2 class="text-xl sm:text-2xl font-bold text-gray-800 select-none">
          {{
            searchMode === 'dictionary'
              ? $t('home.searchResultsTitle.dictionary')
              : searchMode === 'semantic'
                ? $t('home.searchResultsTitle.semantic')
                : $t('home.searchResultsTitle.comments')
          }}
        </h2>

        <div
          v-if="auth.state.isLoading"
          class="flex flex-col sm:flex-row items-end sm:items-center gap-3 sm:space-x-4 ml-auto"
        >
          <!-- Skeleton loader shown while auth state loads -->
          <div class="w-[120px] h-6 bg-gray-100 animate-pulse rounded-full" />
        </div>

        <div
          v-else-if="searchMode === 'dictionary' || searchMode === 'semantic'"
          class="flex flex-col sm:flex-row items-end sm:items-center gap-3 sm:space-x-4 ml-auto"
        >
          <Dropdown
            v-if="auth.state.isLoggedIn && decodedRole !== 'Unconfirmed'"
            class="relative inline-block"
          >
            <template #trigger="{ open: menuOpen }">
              <button
                type="button"
                class="ui-btn--create icon-btn-ui-layout"
                :aria-expanded="menuOpen"
                :aria-label="$t('home.addDefinition')"
              >
                <Plus class="h-4 w-4" /> <span>{{ $t('home.addDefinition') }}</span>
                <ChevronDown
                  class="h-4 w-4 opacity-70 transition-transform duration-200"
                  :class="{ 'rotate-180': menuOpen }"
                  :stroke-width="2"
                />
              </button>
            </template>
            <button
              type="button"
              class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
              @click="router.push('/valsi/add')"
            >
              {{ $t('home.createDefinition') }}
            </button>
            <button
              type="button"
              class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
              :disabled="!hasSearchResults"
              :class="{ 'opacity-50 cursor-not-allowed': !hasSearchResults }"
              @click="hasSearchResults && (showAddAllModal = true)"
            >
              {{ $t('home.addAllToCollection') }}
            </button>
          </Dropdown>
        </div>

        <div
          v-else-if="searchMode === 'comments'"
          class="flex flex-col sm:flex-row items-end sm:items-center gap-3 sm:space-x-4 ml-auto"
        >
          <IconButton
            v-if="auth.state.isLoggedIn"
            :label="$t('home.newFreeThread')"
            button-classes="ui-btn--neutral"
            @click="handleNewFreeComment"
          >
            <template #icon> <AudioWaveform class="h-4 w-4 text-purple-600" /> </template>
          </IconButton>
        </div>

        <div
          v-if="searchMode === 'comments'"
          role="group"
          :aria-label="$t('home.searchResultsTitle.comments')"
          class="flex flex-wrap items-center gap-2 sm:gap-3 w-full sm:w-auto ml-auto justify-end sm:justify-start"
        >
          <Dropdown class="relative block w-auto min-w-0 shrink">
            <template #trigger="{ open: waveMenuOpen }">
              <button
                type="button"
                class="ui-btn--empty inline-flex h-8 min-w-0 w-auto max-w-[min(100vw-4rem,18rem)] items-center justify-between gap-1.5 px-3 text-sm"
              >
                <span class="truncate whitespace-nowrap">{{ waveSourceTriggerLabel }}</span>
                <ChevronDown
                  class="h-4 w-4 shrink-0 opacity-60 transition-transform duration-200"
                  :class="{ 'rotate-180': waveMenuOpen }"
                  :stroke-width="2"
                />
              </button>
            </template>
            <button
              type="button"
              class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
              @click="setWaveSource('all')"
            >
              {{ $t('home.waveSourceAll') }}
            </button>
            <button
              type="button"
              class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
              @click="setWaveSource('jbotcan')"
            >
              {{ $t('home.waveSourceJbotcan') }}
            </button>
            <button
              type="button"
              class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
              @click="setWaveSource('comments')"
            >
              {{ $t('home.waveSourceComments') }}
            </button>
            <button
              type="button"
              class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
              @click="setWaveSource('mail')"
            >
              {{ $t('home.waveSourceMail') }}
            </button>
          </Dropdown>
          <Dropdown class="relative block w-auto min-w-0 shrink">
            <template #trigger="{ open: sortMenuOpen }">
              <button
                type="button"
                class="ui-btn--empty inline-flex h-8 min-w-0 w-auto max-w-[min(100vw-4rem,18rem)] items-center justify-between gap-1.5 px-3 text-sm"
              >
                <span class="truncate whitespace-nowrap">{{ sortByTriggerLabel }}</span>
                <ChevronDown
                  class="h-4 w-4 shrink-0 opacity-60 transition-transform duration-200"
                  :class="{ 'rotate-180': sortMenuOpen }"
                  :stroke-width="2"
                />
              </button>
            </template>
            <button
              type="button"
              class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
              @click="setSortByField('time')"
            >
              {{ $t('sort.time') }}
            </button>
            <button
              type="button"
              class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
              @click="setSortByField('reactions')"
            >
              {{ $t('sort.reactions') }}
            </button>
            <button
              type="button"
              class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
              @click="setSortByField('replies')"
            >
              {{ $t('sort.replies') }}
            </button>
          </Dropdown>
          <button
            type="button"
            class="ui-btn--empty inline-flex h-8 min-w-0 w-auto items-center gap-1.5 whitespace-nowrap px-3 text-sm"
            :title="sortOrder === 'asc' ? $t('sort.ascending') : $t('sort.descending')"
            @click="toggleSortOrder"
          >
            <ChevronUp
              v-if="sortOrder === 'asc'"
              class="h-4 w-4 shrink-0 opacity-60"
              :stroke-width="2"
            />
            <ChevronDown v-else class="h-4 w-4 shrink-0 opacity-60" :stroke-width="2" />
            <span class="whitespace-nowrap">{{
              sortOrder === 'asc' ? $t('sort.asc') : $t('sort.desc')
            }}</span>
          </button>
        </div>
      </div>

      <div class="relative" :class="{ 'min-h-[200px]': isLoading }">
        <div
          v-if="isLoading"
          class="absolute inset-0 z-10 flex justify-center pt-8 sm:pt-12 bg-white/70 backdrop-blur-[1px]"
          aria-busy="true"
          aria-live="polite"
        >
          <div class="animate-spin rounded-full h-8 w-8 shrink-0 border-b-2 border-blue-600" />
        </div>

        <div class="relative z-0" :class="{ 'pointer-events-none select-none': isLoading }">
          <DictionaryEntries
            v-if="searchMode === 'dictionary' || searchMode === 'semantic'"
            :definitions="definitions"
            :is-loading="isLoading"
            :error="error"
            :languages="languages"
            :show-scores="auth.state.isLoggedIn"
            :semantic-search="searchMode === 'semantic'"
            :search-query="searchQuery"
            :show-vote-buttons="auth.state.isLoggedIn"
            :collections="collections"
            :decomposition="decomposition || []"
            @collection-updated="collections = $event"
          />
          <div v-else-if="searchMode === 'comments'" class="space-y-4">
            <div v-if="waveItems.length > 0">
              <div
                v-for="item in waveItems"
                :key="
                  item.source === 'comment' ? item.comment.comment_id : 'mail-' + item.message.id
                "
                class="cursor-pointer"
                @click="
                  item.source === 'comment'
                    ? router.push(
                        `/comments?thread_id=${item.comment.thread_id}&comment_id=${item.comment.parent_id}&scroll_to=${item.comment.comment_id}&valsi_id=${item.comment.valsi_id}&definition_id=${item.comment.definition_id || 0}`
                      )
                    : handleViewThreadSummary(
                        item.message.cleaned_subject || item.message.subject || ''
                      )
                "
              >
                <div
                  v-if="item.source === 'comment' && item.import_source === 'jbotcan'"
                  class="mb-1"
                >
                  <SourceTypeBadge type="jbotcan" label="jbotcan" />
                </div>
                <CommentItem
                  v-if="item.source === 'comment'"
                  :comment="item.comment"
                  :reply-enabled="true"
                  :show-context="true"
                  @reply="handleReply"
                />
                <div
                  v-else
                  class="comment-item bg-white border rounded-lg p-3 my-2 hover:border-blue-300 transition-colors min-w-48"
                >
                  <div
                    class="mb-2 text-sm text-gray-600 whitespace-nowrap overflow-hidden flex items-center"
                  >
                    <SourceTypeBadge type="mail" />
                    <span
                      class="text-blue-700 font-medium ml-1.5 truncate inline-block max-w-[calc(100%-120px)]"
                    >
                      {{ item.message.subject || item.message.cleaned_subject || '-' }}
                    </span>
                  </div>

                  <div class="text-xs text-gray-500 mb-2">
                    {{ item.message.from_address }} · {{ item.message.date || '' }}
                  </div>

                  <div
                    v-if="item.message.parts_json && textParts(item.message.parts_json).length"
                    class="text-sm text-gray-700 border-t border-gray-100 pt-2 mt-2 prose prose-sm max-w-none [&_img]:max-h-48 [&_img]:object-contain"
                  >
                    <LazyMathJax
                      v-for="(part, pidx) in textParts(item.message.parts_json)"
                      :key="pidx"
                      :content="part.content || ''"
                      :enable-markdown="part.mime_type === 'text/plain'"
                    />
                  </div>
                </div>
              </div>
            </div>

            <div
              v-else-if="!isLoading"
              class="text-center py-12 bg-blue-50 rounded-lg border border-blue-100"
            >
              <MessageSquare class="mx-auto h-12 w-12 text-blue-400" />
              <p class="mt-4 text-gray-600">{{ $t('home.noCommentsFound') }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
  <!-- Add-all-to-collection modal (triggered from dictionary action menu) -->
  <AddAllToCollectionWidget
    v-model="showAddAllModal"
    :external-collections="collections"
    :load-all-definition-ids="loadAllDefinitionIdsForCurrentSearch"
    @collection-updated="collections = $event"
  />
  <!-- PaginationComponent -->
  <div v-if="!showTrendingHome">
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
import { jwtDecode } from 'jwt-decode'
import { MessageSquare, ChevronDown, ChevronUp, AudioWaveform, Plus } from 'lucide-vue-next'
import { ref, onMounted, watch, computed, onBeforeUnmount, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'

import {
  searchDefinitions,
  fastSearchDefinitions,
  getLanguages,
  getTrendingComments,
  getRecentChanges,
  searchWaves,
  list_wave_threads,
  getCollections,
  getBulkVotes,
} from '@/api'
import AddAllToCollectionWidget from '@/components/AddAllToCollectionWidget.vue'
import CombinedFilters from '@/components/CombinedFilters.vue'
import CommentItem from '@/components/CommentItem.vue'
import SourceTypeBadge from '@/components/SourceTypeBadge.vue'
import DictionaryEntries from '@/components/DictionaryEntries.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import { Dropdown, IconButton } from '@packages/ui'
import PaginationComponent from '@/components/PaginationComponent.vue'
import RecentChangeItem from '@/components/RecentChangeItem.vue'
import SearchForm from '@/components/SearchForm.vue'
import CombinedFiltersSkeleton from '@/components/skeletons/CombinedFiltersSkeleton.vue'
import SearchFormSkeleton from '@/components/skeletons/SearchFormSkeleton.vue'
import { useAuth } from '@/composables/useAuth'
import { useLanguageSelection } from '@/composables/useLanguageSelection'
import { useSeoHead } from '@/composables/useSeoHead'
import { useI18n } from 'vue-i18n'
import { SearchQueue } from '@/utils/searchQueue'
import { queryStr } from '@/utils/routeQuery'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'

interface JwtUserPayload {
  exp?: number
  username?: string
  role?: string
  authorities?: string[]
}

/** Return text parts from mail message parts_json for display. */
function textParts(partsJson: unknown) {
  if (!partsJson) return []
  const parts = Array.isArray(partsJson) ? partsJson : []
  return parts
    .filter((p) => p && (p.mime_type || p.mimeType || '').startsWith('text/'))
    .map((p) => ({
      mime_type: p.mime_type || p.mimeType || 'text/plain',
      content: typeof p.content === 'string' ? p.content : p.content || '',
    }))
}

/** Map `/waves/threads` items to the same shape as `/waves/search` for the list UI. */
function normalizeWaveThreadItems(items: unknown[]) {
  return items.map((raw) => {
    const item = raw as Record<string, unknown>
    const src = item.source as string
    if (src === 'mail') {
      const preview = item.content_preview as string | undefined
      return {
        source: 'mail',
        message: {
          id: (item.cleaned_subject as string) || 'mail-thread',
          subject: item.subject as string | undefined,
          cleaned_subject: item.cleaned_subject as string,
          from_address: item.from_address as string | undefined,
          date:
            typeof item.last_activity_time === 'number'
              ? new Date(item.last_activity_time * 1000).toUTCString()
              : '',
          parts_json: preview ? [{ mime_type: 'text/plain', content: preview }] : null,
        },
      }
    }
    const firstRaw = item.first_comment_content
    let content: Array<{ type: string; data?: string }> = []
    if (Array.isArray(firstRaw)) {
      content = firstRaw.map((p: { type?: string; data?: string }) => ({
        type: p.type || 'text',
        data: p.data ?? '',
      }))
    } else if (firstRaw && typeof firstRaw === 'object') {
      try {
        const arr = JSON.parse(JSON.stringify(firstRaw))
        if (Array.isArray(arr)) {
          content = arr.map((p: { type?: string; data?: string }) => ({
            type: p.type || 'text',
            data: p.data ?? '',
          }))
        }
      } catch {
        /* ignore */
      }
    }
    const simple = item.simple_content as string | undefined
    if (content.length === 0 && simple) {
      content = [{ type: 'text', data: simple }]
    }
    const uname =
      (item.last_comment_username as string | undefined) ||
      (item.username as string | undefined) ||
      ''
    return {
      source: 'comment',
      import_source: item.import_source as string | undefined,
      comment: {
        comment_id: item.comment_id as number,
        thread_id: item.thread_id as number,
        parent_id: (item.last_comment_parent_id as number | null | undefined) ?? null,
        username: uname,
        subject: (item.first_comment_subject as string | undefined) || '',
        time: item.last_activity_time as number,
        content,
        total_replies: item.total_replies as number,
        total_reactions: (item.last_comment_reactions as number | undefined) ?? 0,
        comment_num: (item.comment_num as number | undefined) ?? 0,
        valsi_id: item.valsi_id as number | null | undefined,
        definition_id: item.definition_id as number | null | undefined,
        valsi_word: item.valsi_word as string | null | undefined,
        definition: item.definition as string | null | undefined,
        reactions: [],
      },
    }
  })
}

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
const decodedToken = computed((): JwtUserPayload | null => {
  if (typeof window === 'undefined') return null
  const token = localStorage.getItem('accessToken')
  if (token) {
    try {
      return jwtDecode<JwtUserPayload>(token)
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
  valsiId: {
    type: Number,
    default: 0,
  },
  definitionId: {
    type: Number,
    default: 0,
  },
})

// State
const waveItems = ref([])
const definitions = ref([])
const decomposition = ref([])
const total = ref(0)
const showAddAllModal = ref(false)
const hasSearchResults = computed(
  () => (searchMode.value === 'dictionary' || searchMode.value === 'semantic') && total.value > 0
)
const currentPage = ref(parseInt(queryStr(route.query.page), 10) || 1)
const totalPages = ref(1)
const sortOrder = ref('desc')

// Get search query from localStorage or use default
const getInitialSearchQuery = () => {
  if (typeof window === 'undefined') return
  const storedQuery = localStorage.getItem('searchQuery')
  return normalizeSearchQuery(storedQuery || props.urlSearchQuery || '')
}

const getInitialGroupByThread = () => {
  if (typeof window === 'undefined') return false
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
  if (typeof window === 'undefined') return
  const storedMode = localStorage.getItem('searchMode')
  const mode = storedMode || props.urlSearchMode
  const normalized = mode === 'messages' ? 'comments' : mode
  if (normalized === 'muplis') return 'semantic'
  return normalized
}

const searchMode = ref(getInitialSearchMode())

/** Filter discussion waves: all site + mail, jbotcan imports, site comments only, or mail only. */
const WAVE_SOURCES = ['all', 'jbotcan', 'comments', 'mail'] as const
type WaveSource = (typeof WAVE_SOURCES)[number]

const waveSource = ref<WaveSource>('all')
const trendingComments = ref([])
const isLoading = ref(true) // Loading state for search results
const isInitialLoading = ref(true) // Loading state for initial component setup (languages etc.)
const isLoadingTrending = ref(false)
const error = ref(null)
const searchFormRef = ref(null)
const searchResultsRef = ref(null)

const { t, locale } = useI18n()

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
    return t('home.searchMetaDescription', {
      query: truncatedSearchQuery.value || searchQuery.value,
    })
  }
  return t('home.metaDescription')
})

useSeoHead({ title: pageTitle, description: pageDescription, pathWithoutLocale: '' })

/** When true, show trending + recent changes; when false, show search / waves results. */
const showTrendingHome = computed(() => {
  const q = (searchQuery.value || '').trim()
  const noFilters = !filters.value.selmaho && !filters.value.username && !filters.value.word_type
  if (searchMode.value === 'comments') return false
  return !q && noFilters
})

const waveSourceTriggerLabel = computed(() => {
  const m: Record<WaveSource, string> = {
    all: t('home.waveSourceAll'),
    jbotcan: t('home.waveSourceJbotcan'),
    comments: t('home.waveSourceComments'),
    mail: t('home.waveSourceMail'),
  }
  return m[waveSource.value]
})

// Filter state
const languages = ref([])
const filters = ref({
  selmaho: '',
  username: '',
  word_type: route.query.word_type ? Number(queryStr(route.query.word_type)) : null,
  isExpanded: route.query.isExpanded === 'true',
  selectedLanguages: route.query.langs ? queryStr(route.query.langs).split(',').map(Number) : [],
  source_langid: route.query.source_langid ? Number(queryStr(route.query.source_langid)) : 1,
  isSemantic:
    searchMode.value === 'semantic' || searchMode.value === 'dictionary'
      ? searchMode.value !== 'dictionary'
      : true,
  searchInPhrases: route.query.searchInPhrases !== 'false',
})

// Search queues to prevent race conditions
const definitionsSearchQueue = new SearchQueue()
const wavesSearchQueue = new SearchQueue()

// Fetch corpus entries
const fetchDefinitions = async (page, search = '') => {
  isLoading.value = true
  error.value = null

  const { requestId, signal } = definitionsSearchQueue.createRequest()
  decomposition.value = []

  try {
    const params: {
      page: number
      per_page: number
      search: string
      include_comments: boolean
      username: string | undefined
      group_by_thread: boolean
      languages?: string
      word_type?: number
      source_langid?: number
      selmaho?: string
      search_in_phrases?: boolean
    } = {
      page,
      per_page: 10,
      search: search,
      include_comments: true,
      username: filters.value.username || undefined,
      ...(filters.value.selectedLanguages.length > 0 && {
        languages: filters.value.selectedLanguages.join(','),
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

    if (filters.value.searchInPhrases !== undefined && filters.value.searchInPhrases !== null) {
      params.search_in_phrases = filters.value.searchInPhrases
    }

    let response
    const isSemantic = searchMode.value === 'semantic'

    if (auth.state.isLoggedIn || isSemantic) {
      response = await searchDefinitions(
        {
          ...params,
          semantic: isSemantic,
        },
        signal
      )
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
    decomposition.value = response.data.decomposition || []

    // Get bulk votes for current user only if we have definitions
    if (auth.state.isLoggedIn && definitions.value.length > 0) {
      try {
        const definitionIds = definitions.value.map((d) => d.definitionid)
        // Only fetch votes if we have IDs to check
        if (definitionIds.length > 0) {
          const votesResponse = await getBulkVotes({ definition_ids: definitionIds })
          const votesMap = votesResponse.data.votes

          // Check again before updating votes (in case another request completed)
          if (definitionsSearchQueue.shouldProcess(requestId)) {
            definitions.value = definitions.value.map((def) => ({
              ...def,
              user_vote: votesMap[def.definitionid] || null,
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

/**
 * Fetch every definition id matching the current search filters across **all** pages
 * (not just the page currently rendered), for the "Add all to collection" action.
 *
 * We deliberately re-issue the same dictionary search endpoint with `per_page=500`,
 * incrementing `page` until either the reported `total` is exhausted, the server
 * returns an empty page, or a high page guard trips (to avoid accidental infinite loops).
 * The server applies the same access/privacy rules it uses for the normal search.
 */
const ADD_ALL_PER_PAGE = 500
const ADD_ALL_PAGE_GUARD = 10000

type LoadAllProgress = (current: number, expectedTotal: number) => void

const loadAllDefinitionIdsForCurrentSearch = async (
  onProgress?: LoadAllProgress
): Promise<number[]> => {
  if (searchMode.value !== 'dictionary' && searchMode.value !== 'semantic') {
    return []
  }

  const baseParams: Record<string, unknown> = {
    per_page: ADD_ALL_PER_PAGE,
    search: (searchQuery.value || '').trim(),
    include_comments: false,
    username: filters.value.username || undefined,
  }
  if (filters.value.selectedLanguages.length > 0) {
    baseParams.languages = filters.value.selectedLanguages.join(',')
  }
  if (!filters.value.selmaho) {
    baseParams.word_type = filters.value.word_type || undefined
  } else {
    baseParams.selmaho = filters.value.selmaho
  }
  if (filters.value.source_langid && filters.value.source_langid !== 1) {
    baseParams.source_langid = filters.value.source_langid
  }
  if (filters.value.searchInPhrases !== undefined && filters.value.searchInPhrases !== null) {
    baseParams.search_in_phrases = filters.value.searchInPhrases
  }

  const isSemantic = searchMode.value === 'semantic'
  const collected: number[] = []
  const seen = new Set<number>()
  let page = 1
  let expectedTotal = 0
  let pagesFetched = 0

  while (pagesFetched < ADD_ALL_PAGE_GUARD) {
    const params: Record<string, unknown> = { ...baseParams, page }
    let response
    if (auth.state.isLoggedIn || isSemantic) {
      response = await searchDefinitions({ ...params, semantic: isSemantic })
    } else {
      const fastParams = { ...params } as Record<string, unknown>
      delete fastParams.include_comments
      response = await fastSearchDefinitions(fastParams)
    }
    pagesFetched += 1

    const defs = (response.data?.definitions || []) as Array<{ definitionid: number }>
    const reportedTotal = Number(response.data?.total ?? 0)
    if (page === 1) {
      expectedTotal = reportedTotal
    }

    for (const d of defs) {
      if (typeof d.definitionid === 'number' && !seen.has(d.definitionid)) {
        seen.add(d.definitionid)
        collected.push(d.definitionid)
      }
    }

    onProgress?.(collected.length, expectedTotal)

    // Stop when server reports we're on/past the last page, or when it returned nothing.
    const lastPage = Math.max(1, Math.ceil(reportedTotal / ADD_ALL_PER_PAGE))
    if (defs.length === 0 || page >= lastPage) break
    page += 1
  }

  return collected
}

type RecentChangeRow = { time: number; [key: string]: unknown }
const recentChanges = ref<RecentChangeRow[]>([])
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

const setCachedRecentChanges = (data: RecentChangeRow[]) => {
  if (typeof window === 'undefined') return
  try {
    const cacheData = {
      data,
      timestamp: Date.now(),
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
    recentChanges.value = cachedChanges.slice(0, 10)
    isLoadingChanges.value = false
  }

  try {
    const trendingResponse = await getTrendingComments({
      limit: 10,
      timespan: 'month',
    })
    trendingComments.value = trendingResponse.data

    // Always fetch recent changes to keep them fresh
    const recentResponse = await getRecentChanges({ limit: 10, types: 'comment,definition' })
    const changes = recentResponse.data.changes
    recentChanges.value = changes

    // Cache the fresh data
    setCachedRecentChanges(changes)
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
  const groups = recentChanges.value.reduce<
    Record<string, { date: Date; changes: RecentChangeRow[] }>
  >((acc, change) => {
    const date = new Date(change.time * 1000).toLocaleDateString(locale.value)
    if (!acc[date]) {
      acc[date] = { date: new Date(change.time * 1000), changes: [] }
    }
    acc[date].changes.push(change)
    return acc
  }, {})
  return Object.values(groups).sort((a, b) => b.date.getTime() - a.date.getTime())
})

const formatDate = (date: Date) =>
  new Intl.DateTimeFormat(locale.value, {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  }).format(date)

// Generic data fetching for other modes
const sortBy = ref(searchMode.value === 'messages' ? 'rank' : 'time')

const sortByTriggerLabel = computed(() => {
  if (sortBy.value === 'reactions') return t('sort.reactions')
  if (sortBy.value === 'replies') return t('sort.replies')
  return t('sort.time')
})

const toggleSortOrder = () => {
  sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  if (searchMode.value === 'comments') {
    fetchWaves(currentPage.value, searchQuery.value)
  } else if (searchMode.value === 'messages') {
    fetchData()
  }
}

const handleSortChange = () => {
  currentPage.value = 1
  if (searchMode.value === 'comments') {
    fetchWaves(1, searchQuery.value)
  } else {
    fetchData()
  }
}

const setSortByField = (value: 'time' | 'reactions' | 'replies') => {
  sortBy.value = value
  handleSortChange()
}

const setWaveSource = (value: WaveSource) => {
  waveSource.value = value
  if (typeof window !== 'undefined') {
    localStorage.setItem('waveSource', value)
  }
  currentPage.value = 1
  router.push({
    query: {
      ...route.query,
      wave_source: value === 'all' ? undefined : value,
      page: undefined,
    },
  })
}

const fetchWaves = async (page, search = '') => {
  isLoading.value = true
  error.value = null

  const { requestId, signal } = wavesSearchQueue.createRequest()

  try {
    const q = (search || '').trim()
    const baseParams = {
      page,
      per_page: 10,
      sort_by: sortBy.value,
      sort_order: sortOrder.value,
      source: waveSource.value,
    }

    if (q) {
      const response = await searchWaves({ ...baseParams, search: q }, signal)

      if (!wavesSearchQueue.shouldProcess(requestId)) {
        return
      }

      waveItems.value = response.data.items
      total.value = response.data.total
      currentPage.value = page
      totalPages.value = Math.ceil(response.data.total / 10)
    } else {
      const response = await list_wave_threads(baseParams, signal)

      if (!wavesSearchQueue.shouldProcess(requestId)) {
        return
      }

      waveItems.value = normalizeWaveThreadItems(response.data.items)
      total.value = response.data.total
      currentPage.value = page
      totalPages.value = Math.ceil(response.data.total / 10)
    }
  } catch (e) {
    if (e.name === 'AbortError' || e.code === 'ERR_CANCELED' || e.message?.includes('canceled')) {
      return
    }
    if (wavesSearchQueue.shouldProcess(requestId)) {
      error.value = e.response?.data?.error || 'Failed to load waves'
      console.error('Error fetching waves:', e)
    }
  } finally {
    if (wavesSearchQueue.shouldProcess(requestId)) {
      isLoading.value = false
    }
  }
}
const fetchData = async () => {
  if (searchMode.value === 'comments') {
    await fetchWaves(currentPage.value, searchQuery.value)
    return
  }

  if (
    !searchQuery.value.trim() &&
    !filters.value.selmaho &&
    !filters.value.username &&
    !filters.value.word_type
  ) {
    // Fetch trending/changes but ensure main loading is false
    await fetchTrendingAndChanges()
    isLoading.value = false // Ensure main loading is stopped
    decomposition.value = []
    return
  }

  // Set loading true only if we are actually fetching search results
  isLoading.value = true

  try {
    if (searchMode.value === 'dictionary' || searchMode.value === 'semantic') {
      await fetchDefinitions(currentPage.value, searchQuery.value)
    }
  } catch (error) {
    console.error('Error fetching data:', error)
    // Ensure loading states are reset on error
    isLoading.value = false
    isLoadingTrending.value = false
  } finally {
    // isLoading is handled within specific fetch functions (fetchDefinitions, fetchComments)
    // or set directly in the try block for other modes.
  }
}

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
    word_type: null,
    source_langid: 1,
    searchInPhrases: true,
    isSemantic: true,
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
      searchInPhrases: filters.value.searchInPhrases === false ? 'false' : undefined,
      wave_source: waveSource.value !== 'all' ? waveSource.value : undefined,
    },
  })
}

// Search handling

const performSearch = ({ query, mode }: { query: string; mode: string }) => {
  // Use semantic mode if we're in dictionary mode and semantic search is enabled
  const effectiveMode = mode === 'dictionary' && filters.value.isSemantic ? 'semantic' : mode

  // Reset to first page whenever search query or mode changes
  const updateParams = {
    ...route.query,
    q: query || undefined, // Use undefined if query is empty
    mode: effectiveMode,
    group_by_thread: groupByThread.value ? 'true' : undefined,
    page: undefined, // Always reset to page 1 for a new search
    langs:
      filters.value.selectedLanguages && filters.value.selectedLanguages.length > 0
        ? filters.value.selectedLanguages.join(',')
        : undefined,
    selmaho: filters.value.selmaho || undefined,
    username: filters.value.username || undefined,
    word_type: filters.value.word_type || undefined,
    searchInPhrases: filters.value.searchInPhrases === false ? 'false' : undefined,
    wave_source: waveSource.value !== 'all' ? waveSource.value : undefined,
  }

  // Handle case where we might be on a localized Home-lang route
  const isHomeRoute =
    route.name === 'Home' || (typeof route.name === 'string' && route.name.startsWith('Home-'))

  if (!isHomeRoute) {
    // If we're not on the home page, redirect to home with the search params
    const currentLocale = route.path.split('/')[1] || 'en'
    router.push({ path: `/${currentLocale}`, query: updateParams })
    return
  }

  if (searchMode.value !== effectiveMode) {
    // Reset sortBy to default for the new mode
    sortBy.value = 'time'
  }

  // Update state before pushing to router to avoid duplicate fetches
  const normalizedQuery = normalizeSearchQuery(query) as string
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

const handleReply = (commentId: number) => {
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

const handleViewThreadSummary = (subject: string) => {
  const currentLocale = route.path.split('/')[1] || 'en'
  const routeName = `ThreadView-${currentLocale}`
  router.push({ name: routeName, params: { subject } })
}

// URL sync
const syncFromRoute = () => {
  // Get all params from URL
  const query = route.query

  // Only update values if they exist in URL
  if (query.q !== undefined) {
    const normalized = normalizeSearchQuery(queryStr(query.q)) as string
    searchQuery.value = normalized
    if (typeof window !== 'undefined') localStorage.setItem('searchQuery', normalized)
  }

  if (query.mode !== undefined) {
    let mode = queryStr(query.mode) === 'messages' ? 'comments' : queryStr(query.mode)
    if (mode === 'muplis') mode = 'semantic'
    searchMode.value = mode
    if (typeof window !== 'undefined') localStorage.setItem('searchMode', mode)
  }
  // groupByThread is now handled by its watcher and getInitialGroupByThread
  // if (query.group_by_thread !== undefined) {
  //   groupByThread.value = query.group_by_thread === 'true';
  //   if (typeof window !== 'undefined') localStorage.setItem('mailSearch_groupByThread', groupByThread.value.toString());
  // }

  if (query.page !== undefined) {
    currentPage.value = parseInt(queryStr(query.page), 10) || 1
  }

  // Sync filters from URL
  if (query.langs !== undefined) {
    filters.value.selectedLanguages = queryStr(query.langs).split(',').map(Number)
  }

  if (query.selmaho !== undefined) {
    filters.value.selmaho = queryStr(query.selmaho)
  }

  if (query.username !== undefined) {
    filters.value.username = queryStr(query.username)
  }

  if (query.word_type !== undefined) {
    const wt = queryStr(query.word_type)
    filters.value.word_type = wt ? Number(wt) : null
  }

  if (query.source_langid !== undefined) {
    filters.value.source_langid = parseInt(queryStr(query.source_langid), 10) || 1 // Default to 1 if invalid
  } else {
    filters.value.source_langid = 1 // Default if not present
  }

  // Sync isSemantic from searchMode which was synced from route mode above
  if (searchMode.value === 'semantic' || searchMode.value === 'dictionary') {
    filters.value.isSemantic = searchMode.value === 'semantic'
  }

  if (query.searchInPhrases !== undefined) {
    filters.value.searchInPhrases = query.searchInPhrases !== 'false'
  } else {
    filters.value.searchInPhrases = true
  }

  if (query.wave_source !== undefined) {
    const w = queryStr(query.wave_source)
    if (WAVE_SOURCES.includes(w as WaveSource)) {
      waveSource.value = w as WaveSource
    }
  } else {
    const stored = typeof window !== 'undefined' ? localStorage.getItem('waveSource') : null
    waveSource.value =
      stored && WAVE_SOURCES.includes(stored as WaveSource) ? (stored as WaveSource) : 'all'
  }
}

const handleKeyDown = (event: KeyboardEvent) => {
  // Check if / was pressed and no input/textarea is focused
  if (event.key === '/' && !['INPUT', 'TEXTAREA'].includes(document.activeElement.tagName)) {
    event.preventDefault()
    searchFormRef.value?.$refs.searchInput?.focus()
  }
}

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown)
  try {
    const languagesResponse = await getLanguages()
    languages.value = languagesResponse.data

    // Set initial languages from route or defaults
    const initialLangs = getInitialLanguages(route, languages.value)
    filters.value.selectedLanguages = initialLangs
    // groupByThread is already initialized with getInitialGroupByThread

    // Initial data like languages is loaded, hide skeletons
    // isInitialLoading.value = false; // Moved down

    // Construct the target query parameters based on current state (from localStorage/defaults)
    const queryToPush = { ...route.query } // Start with current URL query
    let pushNeeded = false

    // Sync 'q' from localStorage/default to URL if different
    if (searchQuery.value && route.query.q !== searchQuery.value) {
      queryToPush.q = searchQuery.value
      pushNeeded = true
    } else if (!searchQuery.value && route.query.q === undefined) {
      // Only if URL.q is also undefined
      queryToPush.q = undefined
    }

    // Sync 'mode' from localStorage/default to URL if different
    if (searchMode.value && route.query.mode !== searchMode.value) {
      queryToPush.mode = searchMode.value
      pushNeeded = true
    }
    // Sync 'group_by_thread' from localStorage/default to URL if different
    const targetGroupByThread = groupByThread.value ? 'true' : undefined
    if (route.query.group_by_thread !== targetGroupByThread) {
      queryToPush.group_by_thread = targetGroupByThread
      pushNeeded = true
    }

    // Sync 'langs' from localStorage/default to URL if different
    const targetLangs =
      filters.value.selectedLanguages.length > 0
        ? filters.value.selectedLanguages.join(',')
        : undefined
    if (route.query.langs !== targetLangs) {
      queryToPush.langs = targetLangs
      pushNeeded = true
    }

    // Clean undefined values from queryToPush before pushing
    Object.keys(queryToPush).forEach(
      (key) => queryToPush[key] === undefined && delete queryToPush[key]
    )

    if (pushNeeded) {
      router.push({ query: queryToPush })
      // The route watcher will handle fetching data with the new URL.
    }
    isInitialLoading.value = false // Skeletons can be hidden now.

    // Auth-dependent fetches (like collections) are handled by the auth state watcher.
    // Initial data fetching (search or trending) is handled by the immediate route query watcher.
  } catch (e) {
    console.error('Error loading initial data:', e)
    // Still hide skeletons even if there's an error loading languages,
    // as the components might still render partially or show an error state.
    isInitialLoading.value = false
  } finally {
    // Ensure skeleton is hidden if try block finishes early or has issues not caught by catch
    isInitialLoading.value = false

    // Focus search input if on home page
    if (
      route.name === 'Home' ||
      (typeof route.name === 'string' && route.name.startsWith('Home-'))
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
      localStorage.setItem('mailSearch_groupByThread', newVal.toString())
    }
    updateUrlWithFilters() // This will trigger the route watcher
  }
})
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
      newQuery.source_langid !== oldQuery?.source_langid ||
      newQuery.searchInPhrases !== oldQuery?.searchInPhrases ||
      newQuery.wave_source !== oldQuery?.wave_source

    const groupByThreadChanged = newQuery.group_by_thread !== oldQuery?.group_by_thread
    if (groupByThreadChanged) {
      groupByThread.value = newQuery.group_by_thread === 'true'
    }

    // Update currentPage based on the new query *before* fetching
    currentPage.value = parseInt(queryStr(newQuery.page), 10) || 1

    // Only fetch data if relevant query params changed
    if (relevantParamsChanged || groupByThreadChanged) {
      syncFromRoute() // Sync other state variables
      await fetchData() // Fetch data using the potentially updated currentPage

      // When page changed, scroll search results to top
      if (newQuery.page !== oldQuery?.page && searchResultsRef.value) {
        await nextTick()
        searchResultsRef.value.scrollIntoView({ block: 'start', behavior: 'instant' })
      }

      // Attempt to focus after data fetch if it's the home route and not initial load
      if (
        (route.name === 'Home' || route.name === 'Home-lang') &&
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

watch(
  () => auth.state.isLoading,
  async (isLoadingAuth, wasLoadingAuth) => {
    // Only proceed if loading has completed (was loading and now is not)
    if (wasLoadingAuth && !isLoadingAuth) {
      // Auth state is now determined
      if (auth.state.isLoggedIn) {
        await fetchCollections()
        /**
         * The route watcher runs with `{ immediate: true }` before `checkAuthStatus` finishes, so
         * `auth.state.isLoggedIn` is still false and `fetchDefinitions` uses `fastSearchDefinitions`
         * (`fast: true`). The API then uses fast search and omits `comment_count` (see backend
         * `fast_search_definitions`). After auth resolves, refetch so dictionary search uses full
         * `searchDefinitions` with `include_comments` and discussion links show stable counts.
         */
        await fetchData()
      }
    }
  }
)
</script>
