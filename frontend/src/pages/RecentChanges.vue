<template>
  <TabbedPageHeader :tabs="tabs" :active-tab="activeTab" :page-title="pageTitle" @tab-click="handleTabClick" />

  <!-- Loading State with Skeleton -->
  <div v-if="isLoading" class="space-y-4">
    <SkeletonActivityItem v-for="n in 5" :key="n" />
  </div>

  <!-- Content -->
  <div v-if="!error" class="space-y-4">
    <ActivityChanges v-if="activeTab === 'changes'" v-model:days="days" :grouped-changes="groupedChanges"
      :format-date="formatDate" />

    <ActivityThreads v-if="activeTab === 'threads'" :threads="threads" :format-date-for-thread="formatDateForThread"
      :format-time="formatTime" />

    <ActivityComments v-if="activeTab === 'all_comments'" :comments="allComments" :format-date="formatDateForThread" />

    <ActivityDefinitions v-if="activeTab === 'all_definitions'" :definitions="allDefinitions"
      :format-date="formatDateForThread" />

    <PaginationComponent v-if="['threads', 'all_comments', 'all_definitions'].includes(activeTab) && totalPages > 1"
      :current-page="currentPage" :total-pages="totalPages" :total="totalItems" :per-page="perPage"
      @prev="changePage(currentPage - 1)" @next="changePage(currentPage + 1)" />
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

const STORAGE_KEY_DAYS = 'recentChanges_daysSelection'
const STORAGE_KEY_TAB = 'recentChanges_activeTab'
const DEFAULT_DAYS = 7

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

// Specific state for 'changes' tab
const getInitialDays = () => {
  if (typeof window === 'undefined') return DEFAULT_DAYS;
  const storedDays = localStorage.getItem(STORAGE_KEY_DAYS)
  return parseInt(route.query.days || storedDays || DEFAULT_DAYS)
}
const days = ref(getInitialDays())

// Loading and error state
const isLoading = ref(true)
const { error, showError, clearError } = useError()

// Active tab state
const getInitialTab = () => {
  if (typeof window === 'undefined') return 'changes';
  const storedTab = localStorage.getItem(STORAGE_KEY_TAB)
  const queryTab = route.query.tab
  const validTabs = tabs.value.map(t => t.key) // Use tabs.value here
  if (queryTab && validTabs.includes(queryTab)) return queryTab
  if (storedTab && validTabs.includes(storedTab)) return storedTab
  return 'changes'
}
const activeTab = ref(getInitialTab())

// Threads pagination
const changePage = (newPage) => {
  if (newPage >= 1 && newPage <= totalPages.value) {
    currentPage.value = newPage
    router.replace({
      query: { ...route.query, page: newPage },
    })
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
      case 'changes':
        changes.value = []
        response = await getRecentChanges({
          days: days.value,
          signal: abortController.signal
        })
        changes.value = response.data.changes
        totalItems.value = response.data.total // Assuming API returns total
        break
      case 'threads':
        threads.value = []
        response = await list_wave_threads({
          page: currentPage.value,
          per_page: perPage.value,
          sort_by: 'time',
          sort_order: 'desc',
          signal: abortController.signal
        })
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
            item.simple_content = item.simple_content ?? (item.content?.filter(p => p.type === 'text').map(p => p.data).join(' ') || '')
            item.first_comment_content = Array.isArray(item.first_comment_content) ? item.first_comment_content : parseContent(item.first_comment_content)
            item.total_replies = item.total_replies ?? 0
          }
        })
        break
      case 'all_comments':
        allComments.value = []
        response = await list_comments({
          page: currentPage.value,
          per_page: perPage.value,
          sort_order: 'desc',
          signal: abortController.signal
        })
        allComments.value = response.data.comments
        totalItems.value = response.data.total
        break
      case 'all_definitions':
        allDefinitions.value = []
        response = await list_definitions({
          page: currentPage.value,
          per_page: perPage.value,
          sort_by: 'created_at',
          sort_order: 'desc',
          signal: abortController.signal
        })
        allDefinitions.value = response.data.definitions
        totalItems.value = response.data.total
        break
    }
    totalPages.value = Math.ceil(totalItems.value / perPage.value)
  } catch (e) {
    if (e.name !== 'AbortError') { // Ignore aborted requests
      showError(e.response?.data?.error || `Failed to load ${tabKey}`)
      // Reset relevant data on error
      switch(tabKey) {
        case 'changes': changes.value = []; break
        case 'threads': threads.value = []; break
        case 'all_comments': allComments.value = []; break
        case 'all_definitions': allDefinitions.value = []; break
      }
    }
  } finally {
    isLoading.value = false
  }
}


const isInitializing = ref(true)

// Watch for changes in the 'days' ref
watch(days, (newDays, oldDays) => {
  if (typeof window === 'undefined') return;

  if (!isInitializing.value && newDays !== oldDays) {
    localStorage.setItem(STORAGE_KEY_DAYS, newDays.toString())
    router.replace({ query: { ...route.query, days: newDays, page: 1 } }) // Reset page on days change
    if (activeTab.value === 'changes') {
      fetchData('changes') // Refetch changes
    }
  }
})

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
  currentPage.value = 1 // Reset page on tab change
  try {
    await fetchData(tabKey)
    await fetchData(tabKey)
    // Only update active tab if fetch was successful
    activeTab.value = tabKey
    // Update URL query params for tab and reset page
    router.replace({
      query: { ...route.query, tab: tabKey, page: undefined }, // Remove page param if it's 1
    })
  } catch (e) {
    showError(e.response?.data?.error || 'Failed to load data')
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
  const currentTab = tabs.value.find(t => t.key === activeTab.value) // Use tabs.value
  const baseTitle = currentTab ? currentTab.label : t('recentChanges.activityTitle') // Default title if tab not found

  if (activeTab.value === 'changes') {
    return `${t('recentChanges.lastDays', { days: days.value })}`
  }
  return baseTitle
})
useSeoHead({ title: pageTitle }, locale.value)

// Unified route watcher
// Additional flag to prevent race conditions with route changes
const isHandlingRouteChange = ref(false)

watch(
  () => route.query,
  async (newQuery, oldQuery) => {
    if (typeof window === 'undefined' || isHandlingRouteChange.value) return;

    const newTab = newQuery.tab && tabs.value.map(t => t.key).includes(newQuery.tab) ? newQuery.tab : getInitialTab() // Use tabs.value
    const newPage = parseInt(newQuery.page) || 1
    const newDaysQuery = parseInt(newQuery.days) || getInitialDays()

    let needsFetch = false

    if (newTab !== activeTab.value) {
      activeTab.value = newTab
      needsFetch = true
    }
    if (newPage !== currentPage.value) {
      currentPage.value = newPage
      needsFetch = true
    }
    if (newDaysQuery !== days.value) {
      days.value = newDaysQuery
      localStorage.setItem(STORAGE_KEY_DAYS, newDaysQuery.toString())
      if (activeTab.value === 'changes') { // Only refetch if days changed and on changes tab
        needsFetch = true
      }
    }

    // Only fetch if not initializing and relevant params changed
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
