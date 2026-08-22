<template>
  <!-- Search and Filter Section -->
  <!-- Skeletons -->
  <SearchFormSkeleton v-if="isInitialLoading" />
  <CombinedFiltersSkeleton
    v-if="isInitialLoading && (searchMode === 'dictionary' || searchMode === 'semantic')"
  />
  <!-- Search Form -->
  <SearchForm
    v-if="!isInitialLoading"
    ref="searchFormRef"
    :initial-query="searchQuery"
    :initial-mode="searchMode"
    :initial-group-by-thread="groupByThread"
    class="w-full"
    @search="performSearch"
  />
  <CombinedFilters
    v-if="searchMode === 'dictionary' || searchMode === 'semantic'"
    v-model="filters"
    :languages="languages"
    :force-show-reset="!!similarDefinitionId"
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
            similarDefinitionId
              ? $t('home.searchResultsTitle.similar')
              : searchMode === 'dictionary'
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
          <ToolbarSelectDropdown
            v-if="auth.state.isLoggedIn && decodedRole !== 'Unconfirmed'"
            :aria-label="$t('home.addDefinition')"
            trigger-icon="ellipsis"
          >
            <template #label>
              <Plus class="h-4 w-4" />
              <span>{{ $t('home.addDefinition') }}</span>
            </template>
            <ToolbarSelectDropdownItem @click="router.push('/valsi/add')">
              {{ $t('home.createDefinition') }}
            </ToolbarSelectDropdownItem>
            <ToolbarSelectDropdownItem
              :disabled="!hasSearchResults"
              :class="{ 'opacity-50 cursor-not-allowed': !hasSearchResults }"
              @click="hasSearchResults && (showAddAllModal = true)"
            >
              {{ $t('home.addAllToCollection') }}
            </ToolbarSelectDropdownItem>
            <ToolbarSelectDropdownItem @click="goToSearchExport">
              <ExportIcon class="h-4 w-4 shrink-0" aria-hidden="true" />
              {{ $t('home.exportResults') }}
            </ToolbarSelectDropdownItem>
          </ToolbarSelectDropdown>
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
          <ToolbarSelectDropdown
            trigger-class="!w-full max-w-[min(100vw-4rem,18rem)]"
            truncate-label
          >
            <template #label>{{ waveSourceTriggerLabel }}</template>
            <ToolbarSelectDropdownItem
              :class="{ 'bg-gray-100': waveSource === 'all' }"
              @click="setWaveSource('all')"
            >
              {{ $t('home.waveSourceAll') }}
            </ToolbarSelectDropdownItem>
            <ToolbarSelectDropdownItem
              :class="{ 'bg-gray-100': waveSource === 'jbotcan' }"
              @click="setWaveSource('jbotcan')"
            >
              {{ $t('home.waveSourceJbotcan') }}
            </ToolbarSelectDropdownItem>
            <ToolbarSelectDropdownItem
              :class="{ 'bg-gray-100': waveSource === 'comments' }"
              @click="setWaveSource('comments')"
            >
              {{ $t('home.waveSourceComments') }}
            </ToolbarSelectDropdownItem>
            <ToolbarSelectDropdownItem
              :class="{ 'bg-gray-100': waveSource === 'mail' }"
              @click="setWaveSource('mail')"
            >
              {{ $t('home.waveSourceMail') }}
            </ToolbarSelectDropdownItem>
            <ToolbarSelectDropdownItem
              :class="{ 'bg-gray-100': waveSource === 'wiki' }"
              @click="setWaveSource('wiki')"
            >
              {{ $t('home.waveSourceWiki') }}
            </ToolbarSelectDropdownItem>
          </ToolbarSelectDropdown>
          <ToolbarSelectDropdown
            trigger-class="!w-full max-w-[min(100vw-4rem,18rem)]"
            truncate-label
          >
            <template #label>{{ sortByTriggerLabel }}</template>
            <ToolbarSelectDropdownItem
              :class="{ 'bg-gray-100': sortBy === 'time' }"
              @click="setSortByField('time')"
            >
              {{ $t('sort.time') }}
            </ToolbarSelectDropdownItem>
            <ToolbarSelectDropdownItem
              :class="{ 'bg-gray-100': sortBy === 'reactions' }"
              @click="setSortByField('reactions')"
            >
              {{ $t('sort.reactions') }}
            </ToolbarSelectDropdownItem>
            <ToolbarSelectDropdownItem
              :class="{ 'bg-gray-100': sortBy === 'replies' }"
              @click="setSortByField('replies')"
            >
              {{ $t('sort.replies') }}
            </ToolbarSelectDropdownItem>
          </ToolbarSelectDropdown>
          <Button
            variant="empty"
            type="button"
            class="inline-flex h-8 min-w-0 w-auto items-center gap-1.5 whitespace-nowrap px-3 text-sm"
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
          </Button>
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
          <PhraseSplit
            v-if="
              (searchMode === 'dictionary' || searchMode === 'semantic') && !similarDefinitionId
            "
            :phrase="searchQuery"
            :selected-languages="filters.selectedLanguages"
            :source-lang-id="filters.source_langid"
            :languages="languages"
          />
          <DictionaryEntries
            v-if="searchMode === 'dictionary' || searchMode === 'semantic'"
            :definitions="definitions"
            :is-loading="isLoading"
            :error="error"
            :languages="languages"
            :show-scores="auth.state.isLoggedIn"
            :semantic-search="searchMode === 'semantic' && !!(searchQuery || '').trim()"
            :search-query="searchQuery"
            :show-vote-buttons="false"
            :collections="collections"
            :collection-matches="collectionMatches"
            :show-global-dictionary-banner="
              !similarDefinitionId &&
              (filters.selectedCollections?.length > 0 || filters.usernames?.length > 0)
            "
            :decomposition="decomposition || []"
            @collection-updated="setCollections($event)"
          >
            <template v-if="similarDefinitionId" #before>
              <div v-if="isLoadingSimilarAnchor" class="flex justify-center py-6">
                <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600" />
              </div>
              <DefinitionCard
                v-else-if="similarAnchorDefinition"
                :definition="similarAnchorDefinition"
                :languages="languages"
                :show-vote-buttons="false"
                :disable-toolbar="true"
                :disable-owner-only-lock="true"
                :hide-find-similar="true"
                :collections="collections"
                @collection-updated="setCollections($event)"
              />
              <div class="relative">
                <div class="surface-definition-compact pr-10">
                  <div class="text-xs font-medium text-gray-500">
                    {{ $t('home.similarSearchLabel') }}
                  </div>
                  <p class="mt-2 text-sm text-gray-700">
                    {{ $t('home.similarSearchHint') }}
                  </p>
                </div>
                <button
                  type="button"
                  class="absolute top-4 right-4 z-10 inline-flex items-center justify-center rounded p-1 text-gray-500 hover:bg-gray-100 hover:text-gray-800"
                  :title="$t('filters.resetAllFilters')"
                  :aria-label="$t('filters.resetAllFilters')"
                  @click="handleFiltersReset"
                >
                  <X class="h-4 w-4 shrink-0" />
                </button>
              </div>
            </template>
          </DictionaryEntries>
          <div v-else-if="searchMode === 'comments'" class="space-y-4">
            <div v-if="waveItems.length > 0">
              <div
                v-for="item in waveItems"
                :key="
                  item.source === 'comment'
                    ? item.comment.comment_id
                    : item.source === 'wiki'
                      ? 'wiki-' + item.article.page_id
                      : 'mail-' + item.message.id
                "
                class="cursor-pointer"
                @click="
                  item.source === 'comment'
                    ? router.push(
                        `/comments?thread_id=${item.comment.thread_id}&comment_id=${item.comment.parent_id}&scroll_to=${item.comment.comment_id}&valsi_id=${item.comment.valsi_id}&definition_id=${item.comment.definition_id || 0}`
                      )
                    : item.source === 'wiki'
                      ? router.push(`/wiki/${encodeURIComponent(item.article.title)}`)
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
                  v-else-if="item.source === 'wiki'"
                  class="comment-item bg-white border rounded-lg p-3 my-2 hover:border-blue-300 transition-colors min-w-48"
                >
                  <div
                    class="mb-2 text-sm text-gray-600 whitespace-nowrap overflow-hidden flex items-center"
                  >
                    <SourceTypeBadge type="wiki" />
                    <span
                      class="text-blue-700 font-medium ml-1.5 truncate inline-block max-w-[calc(100%-120px)]"
                    >
                      {{ item.article.title }}
                    </span>
                  </div>
                  <div v-if="item.article.last_edited" class="text-xs text-gray-500 mb-2">
                    {{ new Date(item.article.last_edited).toLocaleString() }}
                  </div>
                  <div
                    v-if="item.article.content_preview"
                    class="text-sm text-gray-700 border-t border-gray-100 pt-2 mt-2"
                  >
                    {{ item.article.content_preview }}
                  </div>
                </div>
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
    @collection-updated="setCollections($event)"
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
import { MessageSquare, ChevronDown, ChevronUp, AudioWaveform, Plus, X } from '@lucide/vue'
import { ref, onMounted, watch, computed, onBeforeUnmount, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'

import {
  searchDefinitions,
  fastSearchDefinitions,
  getLanguages,
  getTopComments,
  getRecentChanges,
  searchWaves,
  list_wave_threads,
  getDefinition,
  searchItemsInCollections,
} from '@/api'
import AddAllToCollectionWidget from '@/components/AddAllToCollectionWidget.vue'
import CombinedFilters from '@/components/CombinedFilters.vue'
import CommentItem from '@/components/CommentItem.vue'
import SourceTypeBadge from '@/components/SourceTypeBadge.vue'
import DefinitionCard from '@/components/DefinitionCard.vue'
import DictionaryEntries from '@/components/DictionaryEntries.vue'
import PhraseSplit from '@/components/PhraseSplit.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import { Button, IconButton, ToolbarSelectDropdown, ToolbarSelectDropdownItem, ExportIcon } from '@packages/ui'
import PaginationComponent from '@/components/PaginationComponent.vue'
import RecentChangeItem from '@/components/RecentChangeItem.vue'
import SearchForm from '@/components/SearchForm.vue'
import CombinedFiltersSkeleton from '@/components/skeletons/CombinedFiltersSkeleton.vue'
import SearchFormSkeleton from '@/components/skeletons/SearchFormSkeleton.vue'
import { useAuth } from '@/composables/useAuth'
import { useCollectionsCache } from '@/composables/useCollectionsCache'
import { useLanguageSelection } from '@/composables/useLanguageSelection'
import { useSeoHead } from '@/composables/useSeoHead'
import { useDateFormat } from '@/composables/useDateFormat'
import { useI18n } from 'vue-i18n'
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
import { pickLojbanLetteralSeparator } from '@/utils/lojbanLetteralSeparator'

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
    if (src === 'wiki') {
      const summary = (item.summary as Record<string, unknown>) || {}
      return {
        source: 'wiki',
        article: {
          page_id: summary.page_id as number,
          namespace: summary.namespace as number,
          title: summary.title as string,
          last_edited: summary.last_edited as string | null,
          content_preview: summary.content_preview as string | null,
          article_url: summary.article_url as string,
        },
      }
    }
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
const { collections, preload: preloadCollections, setCollections } = useCollectionsCache()
const collectionMatches = ref<CollectionDefinitionCard[]>([])

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
  window.removeEventListener('lensisku:clear-search', handleLogoClear)
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

const hydratedHomeQuery = resolveHomeQuery(route.query)

// URL wins; otherwise last-home / localStorage snapshot
const getInitialSearchQuery = () => {
  if (typeof window === 'undefined') return
  if (route.query.definition_id) return ''
  if (route.query.q !== undefined) {
    return normalizeSearchQuery(queryStr(route.query.q))
  }
  return normalizeSearchQuery(hydratedHomeQuery.q || props.urlSearchQuery || '')
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
  if (route.query.definition_id) return 'semantic'
  const mode =
    (route.query.mode !== undefined ? queryStr(route.query.mode) : '') ||
    hydratedHomeQuery.mode ||
    props.urlSearchMode
  const normalized = mode === 'messages' ? 'comments' : mode
  if (normalized === 'muplis') return 'semantic'
  return normalized
}

const searchMode = ref(getInitialSearchMode())

const getInitialSimilarDefinitionId = (): number | null => {
  const raw = route.query.definition_id
  if (raw === undefined || raw === null || raw === '') return null
  const id = parseInt(queryStr(raw), 10)
  return Number.isFinite(id) && id > 0 ? id : null
}

/** When set, semantic search uses this definition's stored embedding (find-similar mode). */
const similarDefinitionId = ref<number | null>(getInitialSimilarDefinitionId())
const similarAnchorDefinition = ref<Record<string, unknown> | null>(null)
const isLoadingSimilarAnchor = ref(false)

const fetchSimilarAnchorDefinition = async (definitionId: number | null) => {
  if (!definitionId) {
    similarAnchorDefinition.value = null
    isLoadingSimilarAnchor.value = false
    return
  }

  isLoadingSimilarAnchor.value = true
  try {
    const response = await getDefinition(definitionId)
    similarAnchorDefinition.value = response.data as Record<string, unknown>
  } catch (e) {
    console.error('Error fetching similar anchor definition:', e)
    similarAnchorDefinition.value = null
  } finally {
    isLoadingSimilarAnchor.value = false
  }
}

watch(
  similarDefinitionId,
  (id) => {
    fetchSimilarAnchorDefinition(id)
  },
  { immediate: true }
)

/** Filter discussion waves: all site + mail, jbotcan imports, site comments only, or mail only. */
const WAVE_SOURCES = ['all', 'jbotcan', 'comments', 'mail', 'wiki'] as const
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
const { formatDate } = useDateFormat()

// Truncate search query for page title (max 50 characters)
const truncatedSearchQuery = computed(() => {
  if (!searchQuery.value) return null
  const maxLength = 50
  if (searchQuery.value.length <= maxLength) return searchQuery.value
  return searchQuery.value.substring(0, maxLength) + '...'
})

// Page title that reflects the search query
const pageTitle = computed(() => {
  if (similarDefinitionId.value) {
    return t('home.searchResultsTitle.similar')
  }
  if (truncatedSearchQuery.value) {
    const query = truncatedSearchQuery.value
    // Lojban: wrap query in letterals that do not collide with query letters
    if (locale.value === 'jbo') {
      return t('home.searchTitle', {
        query,
        separator: pickLojbanLetteralSeparator(query),
      })
    }
    return t('home.searchTitle', { query })
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
  if (searchMode.value === 'comments') return false
  if (similarDefinitionId.value) return false
  const q = (searchQuery.value || '').trim()
  return !q && !hasActiveSearchFilters(filters.value)
})

const waveSourceTriggerLabel = computed(() => {
  const m: Record<WaveSource, string> = {
    all: t('home.waveSourceAll'),
    jbotcan: t('home.waveSourceJbotcan'),
    comments: t('home.waveSourceComments'),
    mail: t('home.waveSourceMail'),
    wiki: t('home.waveSourceWiki'),
  }
  return m[waveSource.value]
})

// Filter state
const languages = ref([])
const urlFilters = combinedFiltersFromQuery(hydratedHomeQuery)
const filters = ref({
  ...urlFilters,
  isSemantic:
    searchMode.value === 'semantic' || searchMode.value === 'dictionary'
      ? searchMode.value !== 'dictionary'
      : true,
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
    const similarId = similarDefinitionId.value
    const trimmedSearch = similarId ? '' : String(search ?? '').trim()
    // Semantic search needs text (or a definition_id). Empty box + filters → lexical browse.
    const useSemantic = !!similarId || (searchMode.value === 'semantic' && trimmedSearch.length > 0)
    const collectionIds = !similarId
      ? (filters.value.selectedCollections || []).filter(
          (n: number) => Number.isFinite(n) && n > 0
        )
      : []
    const hasAuthors = !similarId && !!filters.value.usernames?.length
    const usePriorityThenGlobal = collectionIds.length > 0 || hasAuthors

    const params: Record<string, unknown> = {
      page,
      per_page: 10,
      search: similarId ? undefined : trimmedSearch || undefined,
      definition_id: similarId || undefined,
      include_comments: true,
      username: filters.value.usernames?.length ? filters.value.usernames.join(',') : undefined,
      exclude_usernames: filters.value.excludeUsernames?.length
        ? filters.value.excludeUsernames.join(',')
        : undefined,
      group_by_thread: groupByThread.value,
      semantic: useSemantic,
    }

    if (filters.value.selectedLanguages.length > 0) {
      params.languages = filters.value.selectedLanguages.join(',')
    }
    if (!filters.value.selmaho) {
      params.word_type = filters.value.word_type || undefined
    } else {
      params.selmaho = filters.value.selmaho
    }
    if (filters.value.source_langid && filters.value.source_langid !== 1) {
      params.source_langid = filters.value.source_langid
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

    let response
    if (auth.state.isLoggedIn || useSemantic) {
      response = await searchDefinitions(params, signal)
    } else {
      const fastParams = { ...params }
      delete fastParams.include_comments
      delete fastParams.definition_id
      response = await fastSearchDefinitions(fastParams, signal)
    }

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
    collectionMatches.value = [...collectionHits, ...authorHits]

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
 * Authors∪collections is a union: definition ids from selected collections plus
 * definition ids by selected authors. The unscoped global dictionary shown below
 * the banner in the UI is not included.
 */
const ADD_ALL_PER_PAGE = 500
/** Matches server `MAX_MULTI_COLLECTION_PER_PAGE` on `GET /collections/items/search`. */
const ADD_ALL_COLLECTION_PER_PAGE = 100
const ADD_ALL_PAGE_GUARD = 10000

type LoadAllProgress = (current: number, expectedTotal: number) => void

const buildSharedSearchFilterParams = (opts?: {
  includeAuthors?: boolean
}): Record<string, unknown> => {
  const includeAuthors = opts?.includeAuthors !== false
  const params: Record<string, unknown> = {
    username:
      includeAuthors && filters.value.usernames?.length
        ? filters.value.usernames.join(',')
        : undefined,
    exclude_usernames: filters.value.excludeUsernames?.length
      ? filters.value.excludeUsernames.join(',')
      : undefined,
  }
  if (filters.value.selectedLanguages.length > 0) {
    params.languages = filters.value.selectedLanguages.join(',')
  }
  if (!filters.value.selmaho) {
    params.word_type = filters.value.word_type || undefined
  } else {
    params.selmaho = filters.value.selmaho
  }
  if (filters.value.source_langid && filters.value.source_langid !== 1) {
    params.source_langid = filters.value.source_langid
  }
  if (filters.value.searchInPhrases !== undefined && filters.value.searchInPhrases !== null) {
    params.search_in_phrases = filters.value.searchInPhrases
  }
  return params
}

const loadAllDefinitionIdsFromCollections = async (
  collectionIds: number[],
  trimmedSearch: string,
  useSemantic: boolean,
  onProgress?: LoadAllProgress,
  progressBase = 0,
  progressExpectedBase = 0
): Promise<{ ids: number[]; reportedTotal: number }> => {
  const baseParams: Record<string, unknown> = {
    // OR with authors: do not restrict collection hits to include-authors.
    ...buildSharedSearchFilterParams({ includeAuthors: false }),
    collection_ids: collectionIds.join(','),
    per_page: ADD_ALL_COLLECTION_PER_PAGE,
    search: trimmedSearch || undefined,
  }
  if (useSemantic && trimmedSearch) {
    baseParams.semantic = true
  }

  const collected: number[] = []
  const seen = new Set<number>()
  let page = 1
  let reportedTotal = 0
  let pagesFetched = 0

  while (pagesFetched < ADD_ALL_PAGE_GUARD) {
    const response = await searchItemsInCollections({ ...baseParams, page })
    pagesFetched += 1

    const items = (response.data?.items || []) as Array<{ definition_id?: number | null }>
    reportedTotal = Number(response.data?.total ?? 0)

    for (const item of items) {
      const id = item.definition_id
      if (typeof id === 'number' && id > 0 && !seen.has(id)) {
        seen.add(id)
        collected.push(id)
      }
    }

    onProgress?.(progressBase + collected.length, progressExpectedBase + reportedTotal)

    const lastPage = Math.max(1, Math.ceil(reportedTotal / ADD_ALL_COLLECTION_PER_PAGE))
    if (items.length === 0 || page >= lastPage) break
    page += 1
  }

  return { ids: collected, reportedTotal }
}

const loadAllDefinitionIdsFromDictionary = async (
  trimmedSearch: string,
  useSemantic: boolean,
  onProgress?: LoadAllProgress,
  progressBase = 0,
  progressExpectedBase = 0
): Promise<{ ids: number[]; reportedTotal: number }> => {
  const baseParams: Record<string, unknown> = {
    ...buildSharedSearchFilterParams({ includeAuthors: true }),
    per_page: ADD_ALL_PER_PAGE,
    search: similarDefinitionId.value ? undefined : trimmedSearch || undefined,
    definition_id: similarDefinitionId.value || undefined,
    include_comments: false,
  }

  const collected: number[] = []
  const seen = new Set<number>()
  let page = 1
  let reportedTotal = 0
  let pagesFetched = 0

  while (pagesFetched < ADD_ALL_PAGE_GUARD) {
    const params: Record<string, unknown> = { ...baseParams, page }
    let response
    if (auth.state.isLoggedIn || useSemantic) {
      response = await searchDefinitions({ ...params, semantic: useSemantic })
    } else {
      const fastParams = { ...params } as Record<string, unknown>
      delete fastParams.include_comments
      response = await fastSearchDefinitions(fastParams)
    }
    pagesFetched += 1

    const defs = (response.data?.definitions || []) as Array<{ definitionid: number }>
    reportedTotal = Number(response.data?.total ?? 0)

    for (const d of defs) {
      if (typeof d.definitionid === 'number' && !seen.has(d.definitionid)) {
        seen.add(d.definitionid)
        collected.push(d.definitionid)
      }
    }

    onProgress?.(progressBase + collected.length, progressExpectedBase + reportedTotal)

    const lastPage = Math.max(1, Math.ceil(reportedTotal / ADD_ALL_PER_PAGE))
    if (defs.length === 0 || page >= lastPage) break
    page += 1
  }

  return { ids: collected, reportedTotal }
}

const loadAllDefinitionIdsForCurrentSearch = async (
  onProgress?: LoadAllProgress
): Promise<number[]> => {
  if (searchMode.value !== 'dictionary' && searchMode.value !== 'semantic') {
    return []
  }

  const trimmedSearch = similarDefinitionId.value ? '' : (searchQuery.value || '').trim()
  const useSemantic =
    !!similarDefinitionId.value || (searchMode.value === 'semantic' && trimmedSearch.length > 0)

  const collectionIds = (filters.value.selectedCollections || []).filter(
    (n: number) => Number.isFinite(n) && n > 0
  )
  const hasCollections = collectionIds.length > 0 && !similarDefinitionId.value
  const hasAuthors = !!filters.value.usernames?.length
  // Dictionary when: authors selected (author half of the union), or no collections
  // (normal search / other filters). Skip unscoped global fallback for collections-only.
  const needDictionary = hasAuthors || !hasCollections

  const collected: number[] = []
  const seen = new Set<number>()
  let expectedTotal = 0

  const mergeIds = (ids: number[]) => {
    for (const id of ids) {
      if (!seen.has(id)) {
        seen.add(id)
        collected.push(id)
      }
    }
  }

  if (hasCollections) {
    const fromCollections = await loadAllDefinitionIdsFromCollections(
      collectionIds,
      trimmedSearch,
      useSemantic,
      onProgress,
      0,
      0
    )
    mergeIds(fromCollections.ids)
    expectedTotal += fromCollections.reportedTotal
    onProgress?.(collected.length, expectedTotal)
  }

  if (needDictionary) {
    const fromDictionary = await loadAllDefinitionIdsFromDictionary(
      trimmedSearch,
      useSemantic,
      onProgress,
      collected.length,
      expectedTotal
    )
    mergeIds(fromDictionary.ids)
    expectedTotal += fromDictionary.reportedTotal
    onProgress?.(collected.length, expectedTotal)
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
    const trendingResponse = await getTopComments()
    trendingComments.value = trendingResponse.data

    // Always fetch recent changes to keep them fresh
    const recentResponse = await getRecentChanges({ limit: 10, home: true })
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

const dateKey = (d: Date) =>
  `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`

const groupedChanges = computed(() => {
  const groups = recentChanges.value.reduce<
    Record<string, { date: Date; changes: RecentChangeRow[] }>
  >((acc, change) => {
    const d = new Date(change.time * 1000)
    const key = dateKey(d)
    if (!acc[key]) {
      acc[key] = { date: d, changes: [] }
    }
    acc[key].changes.push(change)
    return acc
  }, {})
  return Object.values(groups).sort((a, b) => b.date.getTime() - a.date.getTime())
})

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
    query: compactQuery({
      ...route.query,
      wave_source: value === 'all' ? undefined : value,
      page: undefined,
    }),
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
    !similarDefinitionId.value &&
    !hasActiveSearchFilters(filters.value)
  ) {
    // Fetch trending/changes but ensure main loading is false
    await fetchTrendingAndChanges()
    isLoading.value = false // Ensure main loading is stopped
    collectionMatches.value = []
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
  // Keep searchMode in sync with the isSemantic toggle before building the URL,
  // so updateUrlWithFilters writes the correct mode in a single push.
  if (searchMode.value === 'semantic' || searchMode.value === 'dictionary') {
    const targetMode = filters.value.isSemantic ? 'semantic' : 'dictionary'
    if (searchMode.value !== targetMode) {
      searchMode.value = targetMode
    }
  }
  updateUrlWithFilters()
}

const handleFiltersReset = async () => {
  filters.value = {
    selmaho: '',
    usernames: [],
    excludeUsernames: [],
    isExpanded: false,
    selectedLanguages: [],
    selectedCollections: [],
    word_type: null,
    source_langid: 1,
    searchInPhrases: true,
    isSemantic: true,
  }
  currentPage.value = 1
  searchQuery.value = ''
  similarDefinitionId.value = null
  similarAnchorDefinition.value = null
  // if (searchFormRef.value) {
  //   searchFormRef.value.query = ''
  // }
  updateUrlWithFilters()
}

const updateUrlWithFilters = () => {
  router.push({
    query: commitHomeQuery({
      ...route.query,
      q: searchQuery.value || undefined,
      mode: searchMode.value,
      definition_id: similarDefinitionId.value ? String(similarDefinitionId.value) : undefined,
      ...combinedFiltersToQuery(filters.value),
      group_by_thread: groupByThread.value ? 'true' : undefined,
      wave_source: waveSource.value !== 'all' ? waveSource.value : undefined,
    }),
  })
}

// Search handling

const performSearch = ({ query, mode }: { query: string; mode: string }) => {
  // Use semantic mode if we're in dictionary mode and semantic search is enabled
  const effectiveMode = mode === 'dictionary' && filters.value.isSemantic ? 'semantic' : mode

  // Text search exits find-similar mode
  similarDefinitionId.value = null

  // Reset to first page whenever search query or mode changes
  const updateParams = commitHomeQuery({
    ...route.query,
    q: query || undefined, // Use undefined if query is empty
    mode: effectiveMode,
    definition_id: undefined,
    group_by_thread: groupByThread.value ? 'true' : undefined,
    page: undefined, // Always reset to page 1 for a new search
    ...combinedFiltersToQuery(filters.value),
    wave_source: waveSource.value !== 'all' ? waveSource.value : undefined,
  })

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

const handleLogoClear = () => {
  searchFormRef.value?.focusInput()
  similarDefinitionId.value = null
  performSearch({ query: '', mode: searchMode.value })
}

const goToSearchExport = () => {
  const mode =
    searchMode.value === 'dictionary' || searchMode.value === 'semantic'
      ? searchMode.value
      : filters.value.isSemantic
        ? 'semantic'
        : 'dictionary'
  router.push({
    path: '/export/search',
    query: compactQuery({
      q: searchQuery.value || undefined,
      mode,
      ...combinedFiltersToQuery(filters.value),
    }),
  })
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

  if (query.definition_id !== undefined && query.definition_id !== '') {
    const id = parseInt(queryStr(query.definition_id), 10)
    similarDefinitionId.value = Number.isFinite(id) && id > 0 ? id : null
    if (similarDefinitionId.value) {
      searchMode.value = 'semantic'
      filters.value.isSemantic = true
      searchQuery.value = ''
    }
  } else {
    similarDefinitionId.value = null
  }
  // groupByThread is now handled by its watcher and getInitialGroupByThread
  // if (query.group_by_thread !== undefined) {
  //   groupByThread.value = query.group_by_thread === 'true';
  //   if (typeof window !== 'undefined') localStorage.setItem('mailSearch_groupByThread', groupByThread.value.toString());
  // }

  if (query.page !== undefined) {
    currentPage.value = parseInt(queryStr(query.page), 10) || 1
  }

  // URL keys override stored filters; omitted keys keep the hydrated localStorage state
  Object.assign(filters.value, applyCombinedFiltersFromQuery(filters.value, query))

  // Sync isSemantic from searchMode which was synced from route mode above
  if (searchMode.value === 'semantic' || searchMode.value === 'dictionary') {
    filters.value.isSemantic = searchMode.value === 'semantic'
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
  window.addEventListener('lensisku:clear-search', handleLogoClear)
  try {
    const languagesResponse = await getLanguages()
    const initialLangs = getInitialLanguages(route, languagesResponse.data)
    filters.value.selectedLanguages = initialLangs
    languages.value = languagesResponse.data

    const queryToPush = commitHomeQuery({
      ...route.query,
      ...resolveHomeQuery(route.query),
      q: similarDefinitionId.value ? undefined : searchQuery.value || undefined,
      mode: searchMode.value,
      definition_id: similarDefinitionId.value ? String(similarDefinitionId.value) : undefined,
      ...combinedFiltersToQuery(filters.value),
      group_by_thread: groupByThread.value ? 'true' : undefined,
      wave_source: waveSource.value !== 'all' ? waveSource.value : undefined,
    })
    if (similarDefinitionId.value) {
      delete queryToPush.q
    }

    const currentCompact = compactQuery({ ...route.query })
    const pushNeeded = JSON.stringify(currentCompact) !== JSON.stringify(queryToPush)

    if (pushNeeded) {
      router.push({ query: queryToPush })
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
      newQuery.collections !== oldQuery?.collections ||
      newQuery.selmaho !== oldQuery?.selmaho ||
      newQuery.username !== oldQuery?.username ||
      newQuery.exclude_usernames !== oldQuery?.exclude_usernames ||
      newQuery.word_type !== oldQuery?.word_type ||
      newQuery.source_langid !== oldQuery?.source_langid ||
      newQuery.searchInPhrases !== oldQuery?.searchInPhrases ||
      newQuery.wave_source !== oldQuery?.wave_source ||
      newQuery.definition_id !== oldQuery?.definition_id

    const groupByThreadChanged = newQuery.group_by_thread !== oldQuery?.group_by_thread
    if (groupByThreadChanged) {
      groupByThread.value = newQuery.group_by_thread === 'true'
    }

    if (newQuery.isExpanded !== oldQuery?.isExpanded) {
      filters.value.isExpanded = queryStr(newQuery.isExpanded) === 'true'
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
        void preloadCollections()
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
