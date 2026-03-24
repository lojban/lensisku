<template>
  <TabbedPageHeader
    :tabs="tabs"
    :active-tab="activeTab"
    :page-title="pageTitle"
    @tab-click="handleTabClick"
  />

  <!-- Loading State with Skeleton -->
  <div v-if="isLoading" class="space-y-4">
    <SkeletonActivityItem v-for="n in 5" :key="n" />
  </div>

  <!-- Content -->
  <div v-if="!error" class="space-y-4">
    <ActivityChanges
      v-if="activeTab === 'changes'"
      :grouped-changes="groupedChanges"
      :format-date="formatDate"
    />

    <ActivityThreads
      v-if="activeTab === 'threads'"
      :threads="threads"
      :format-date-for-thread="formatDateForThread"
      :format-time="formatTime"
    />

    <ActivityComments
      v-if="activeTab === 'all_comments'"
      :comments="allComments"
      :format-date="formatDateForThread"
    />

    <ActivityDefinitions
      v-if="activeTab === 'all_definitions'"
      :definitions="allDefinitions"
      :format-date="formatDateForThread"
    />

    <PaginationComponent
      v-if="
        (activeTab === 'changes' && (currentPage > 1 || nextCursor)) ||
        (['threads', 'all_comments', 'all_definitions'].includes(activeTab) && totalPages > 1)
      "
      :current-page="currentPage"
      :total-pages="
        activeTab === 'changes' ? (nextCursor ? currentPage + 1 : currentPage) : totalPages
      "
      :total="activeTab === 'changes' ? 0 : totalItems"
      :per-page="perPage"
      @prev="changePage(currentPage - 1)"
      @next="changePage(currentPage + 1)"
    />
  </div>
</template>

<script setup>
import { History, Waves, MessageSquare, Book } from 'lucide-vue-next'
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { getRecentChanges, list_wave_threads, list_comments, list_definitions } from '@/api'
import ActivityChanges from '@/components/activity/ActivityChanges.vue'
import ActivityComments from '@/components/activity/ActivityComments.vue'
import ActivityDefinitions from '@/components/activity/ActivityDefinitions.vue'
import ActivityThreads from '@/components/activity/ActivityThreads.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import SkeletonActivityItem from '@/components/activity/SkeletonActivityItem.vue'
import TabbedPageHeader from '@/components/TabbedPageHeader.vue'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const { t, locale } = useI18n()

const tabs = computed(() => [
  { key: 'changes', label: t('recentChanges.recentChanges'), icon: History },
  { key: 'threads', label: t('recentChanges.discussionWaves'), icon: Waves },
  { key: 'all_comments', label: t('recentChanges.allComments'), icon: MessageSquare },
  { key: 'all_definitions', label: t('recentChanges.allDefinitions'), icon: Book },
])

const STORAGE_KEY_TAB = 'recentChanges_activeTab'

const route = useRoute()
const router = useRouter()

// State for different tabs
const threads = ref([])
const allComments = ref([])
const allDefinitions = ref([])
const changes = ref([])

// Pagination state (shared for simplicity, adjust if needed per tab)
const currentPage = ref(1)
const perPage = ref(20)
const totalItems = ref(0) // Generic total for the active tab
const totalPages = ref(1)

// Cursor-based pagination for 'changes' tab: cursors[i] = cursor to request page i+1 (cursors[0] = undefined for page 1)
const cursors = ref([])
const nextCursor = ref(null) // next_cursor from last response (for changes tab)

// Loading and error state
const isLoading = ref(true)
const { error, showError, clearError } = useError()

// Active tab state
const getInitialTab = () => {
  if (typeof window === 'undefined') return 'changes'
  const storedTab = localStorage.getItem(STORAGE_KEY_TAB)
  const queryTab = route.query.tab
  const validTabs = tabs.value.map((t) => t.key) // Use tabs.value here
  if (queryTab && validTabs.includes(queryTab)) return queryTab
  if (storedTab && validTabs.includes(storedTab)) return storedTab
  return 'changes'
}
const activeTab = ref(getInitialTab())

// Pagination: update URL; route watcher will sync currentPage and fetch
const changePage = (newPage) => {
  if (activeTab.value === 'changes') {
    const canGo =
      newPage >= 1 &&
      (newPage <= currentPage.value || nextCursor.value || cursors.value[newPage - 1])
    if (canGo) router.replace({ query: { ...route.query, page: newPage } })
  } else if (newPage >= 1 && newPage <= totalPages.value) {
    router.replace({ query: { ...route.query, page: newPage } })
  }
}

// Unified fetch method with request deduplication and abort control
const fetchData = async (tabKey) => {
  isLoading.value = true
  clearError()

  // Abort any previous request
  if (abortController) {
    abortController.abort()
  }
  abortController = new AbortController()
  try {
    let response
    switch (tabKey) {
      case 'changes': {
        changes.value = []
        let pageToFetch = currentPage.value
        if (currentPage.value > 1 && !cursors.value[currentPage.value - 1]) {
          currentPage.value = 1
          pageToFetch = 1
          router.replace({ query: { ...route.query, page: 1 } })
        }
        const after = pageToFetch > 1 ? cursors.value[pageToFetch - 1] : undefined
        response = await getRecentChanges(
          { limit: perPage.value, ...(after && { after }) },
          abortController.signal
        )
        changes.value = response.data.changes
        nextCursor.value = response.data.next_cursor ?? null
        if (response.data.next_cursor) {
          const c = [...cursors.value]
          c[pageToFetch] = response.data.next_cursor
          cursors.value = c
        }
        totalItems.value = 0
        break
      }
      case 'threads':
        threads.value = []
        response = await list_wave_threads(
          {
            page: currentPage.value,
            per_page: perPage.value,
            sort_by: 'time',
            sort_order: 'desc',
          },
          abortController.signal
        )
        threads.value = response.data.items
        totalItems.value = response.data.total

        // Normalize comment threads for display (same shape as before for comment type)
        threads.value.forEach((item) => {
          if (item.source === 'comment') {
            const parseContent = (content) => {
              if (content && typeof content === 'string') {
                try {
                  return JSON.parse(content)
                } catch (e) {
                  return [{ type: 'text', data: content }]
                }
              }
              return Array.isArray(content) ? content : []
            }
            item.time = item.last_activity_time
            item.content = parseContent(item.first_comment_content)
            item.simple_content =
              item.simple_content ??
              (item.content
                ?.filter((p) => p.type === 'text')
                .map((p) => p.data)
                .join(' ') ||
                '')
            item.first_comment_content = Array.isArray(item.first_comment_content)
              ? item.first_comment_content
              : parseContent(item.first_comment_content)
            item.total_replies = item.total_replies ?? 0
          }
        })
        break
      case 'all_comments':
        allComments.value = []
        response = await list_comments(
          {
            page: currentPage.value,
            per_page: perPage.value,
            sort_order: 'desc',
          },
          abortController.signal
        )
        allComments.value = response.data.comments
        totalItems.value = response.data.total
        break
      case 'all_definitions':
        allDefinitions.value = []
        response = await list_definitions(
          {
            page: currentPage.value,
            per_page: perPage.value,
            sort_by: 'created_at',
            sort_order: 'desc',
          },
          abortController.signal
        )
        allDefinitions.value = response.data.definitions
        totalItems.value = response.data.total
        break
    }
    totalPages.value = Math.ceil(totalItems.value / perPage.value)
  } catch (e) {
    if (e.name !== 'AbortError') {
      // Ignore aborted requests
      const data = e.response?.data
      const msg = data?.detail ?? data?.error ?? `Failed to load ${tabKey}`
      showError(msg)
      // Reset relevant data on error
      switch (tabKey) {
        case 'changes':
          changes.value = []
          break
        case 'threads':
          threads.value = []
          break
        case 'all_comments':
          allComments.value = []
          break
        case 'all_definitions':
          allDefinitions.value = []
          break
      }
    }
  } finally {
    isLoading.value = false
  }
}

const isInitializing = ref(true)

// Watch activeTab and save to localStorage
watch(activeTab, (newTab) => {
  if (typeof window !== 'undefined') {
    localStorage.setItem(STORAGE_KEY_TAB, newTab)
  }
})

const handleTabClick = async (tabKey) => {
  if (tabKey === activeTab.value || isLoading.value) return

  isLoading.value = true
  clearError()
  currentPage.value = 1
  if (tabKey === 'changes') {
    cursors.value = []
    nextCursor.value = null
  }
  try {
    await fetchData(tabKey)
    activeTab.value = tabKey
    router.replace({
      query: { ...route.query, tab: tabKey, page: undefined },
    })
  } catch (e) {
    const data = e.response?.data
    showError(data?.detail ?? data?.error ?? 'Failed to load data')
  } finally {
    isLoading.value = false
  }
}

// Abort controller for canceling pending requests
let abortController = null

onMounted(async () => {
  const initialTab = getInitialTab()
  currentPage.value = parseInt(route.query.page) || 1
  await fetchData(initialTab)
  activeTab.value = initialTab // Set activeTab after initial fetch
  isInitializing.value = false
})

onUnmounted(() => {
  if (abortController) {
    abortController.abort()
  }
})

const groupedChanges = computed(() => {
  const groups = changes.value.reduce((acc, change) => {
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

const formatTime = (timestamp) =>
  new Date(timestamp * 1000).toLocaleTimeString(locale.value, {
    hour: '2-digit',
    minute: '2-digit',
  })

const formatDateForThread = (timestamp) =>
  new Date(timestamp * 1000).toLocaleDateString(locale.value, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })

// Reactive page title
const pageTitle = computed(() => {
  const currentTab = tabs.value.find((t) => t.key === activeTab.value)
  return currentTab ? currentTab.label : t('recentChanges.activityTitle')
})
useSeoHead({ title: pageTitle }, locale.value)

// Unified route watcher
// Additional flag to prevent race conditions with route changes
const isHandlingRouteChange = ref(false)

watch(
  () => route.query,
  async (newQuery) => {
    if (typeof window === 'undefined' || isHandlingRouteChange.value) return

    const newTab =
      newQuery.tab && tabs.value.map((t) => t.key).includes(newQuery.tab)
        ? newQuery.tab
        : getInitialTab()
    const newPage = parseInt(newQuery.page) || 1

    let needsFetch = false
    if (newTab !== activeTab.value) {
      activeTab.value = newTab
      needsFetch = true
      if (newTab === 'changes') {
        cursors.value = []
        nextCursor.value = null
      }
    }
    if (newPage !== currentPage.value) {
      currentPage.value = newPage
      needsFetch = true
    }

    if (!isInitializing.value && needsFetch) {
      isHandlingRouteChange.value = true
      try {
        await fetchData(activeTab.value)
      } finally {
        isHandlingRouteChange.value = false
      }
    }
  },
  { deep: true, immediate: true }
)
</script>
