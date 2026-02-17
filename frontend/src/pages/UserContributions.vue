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
  <div v-else class="space-y-4">
    <!-- Comments Tab -->
    <ActivityComments
      v-if="activeTab === 'comments'"
      :comments="comments"
      :format-date="formatDate"
    />

    <!-- Definitions Tab -->
    <ActivityDefinitions
      v-if="activeTab === 'definitions'"
      :definitions="definitions"
      :format-date="formatDate"
    />


    <!-- PaginationComponent -->
    <div v-if="totalPages > 1">
      <PaginationComponent
        :current-page="currentPage"
        :total-pages="totalPages"
        :total="total"
        :per-page="perPage"
        @prev="() => changePage(currentPage - 1)"
        @next="() => changePage(currentPage + 1)"
      />
    </div>
  </div>
</template>

<script setup>
import { MessageSquare, Book } from 'lucide-vue-next'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { getUserComments, getUserDefinitions, getUserVotes } from '@/api'
import ActivityComments from '@/components/activity/ActivityComments.vue'
import ActivityDefinitions from '@/components/activity/ActivityDefinitions.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import SkeletonActivityItem from '@/components/activity/SkeletonActivityItem.vue'
import TabbedPageHeader from '@/components/TabbedPageHeader.vue'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const props = defineProps({
  username: {
    type: String,
    required: true,
  },
})

const route = useRoute()
const router = useRouter()
const { showError, clearError } = useError()
const { t, locale } = useI18n()

// State
const isLoading = ref(false)
const activeTab = ref('comments')
const currentPage = ref(1)
const perPage = ref(20)
const total = ref(0)
const totalPages = computed(() => Math.ceil(total.value / perPage.value))

const comments = ref([])
const definitions = ref([])
const votes = ref([])

// Flag to prevent double fetches when updating route programmatically
const isUpdatingRoute = ref(false)

// Methods
const fetchComments = async () => {
  isLoading.value = true
  clearError()

  try {
    const response = await getUserComments(props.username, {
      page: currentPage.value,
      per_page: perPage.value,
    })
    comments.value = response.data.items.map((comment) => ({
      ...comment,
      username: props.username,
    }))
    total.value = response.data.total
  } catch (e) {
    showError(t('userContributions.loadCommentsError')) // Use t()
  } finally {
    isLoading.value = false
  }
}

const fetchDefinitions = async () => {
  isLoading.value = true
  clearError()

  try {
    const response = await getUserDefinitions(props.username, {
      page: currentPage.value,
      per_page: perPage.value,
    })
    definitions.value = response.data.items
    total.value = response.data.total
  } catch (e) {
    showError(t('userContributions.loadDefinitionsError')) // Use t()
  } finally {
    isLoading.value = false
  }
}

const fetchVotes = async () => {
  isLoading.value = true
  clearError()

  try {
    const response = await getUserVotes({
      page: currentPage.value,
      per_page: perPage.value,
    })
    votes.value = response.data.items
    total.value = response.data.total
  } catch (e) {
    showError(t('userContributions.loadVotesError')) // Use t()
  } finally {
    isLoading.value = false
  }
}

const fetchData = (tabKey) => {
  switch (tabKey) {
    case 'definitions':
      return fetchDefinitions()
    case 'votes':
      return fetchVotes()
    default:
      return fetchComments()
  }
}

const changePage = (newPage) => {
  if (newPage >= 1 && newPage <= totalPages.value) {
    isUpdatingRoute.value = true
    currentPage.value = newPage
    fetchData(activeTab.value)
    router.replace({
      query: { ...route.query, tab: activeTab.value, page: newPage },
    }).finally(() => {
      isUpdatingRoute.value = false
    })
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }
}

const formatDate = (timestamp) => {
  return new Date(timestamp).toLocaleString(locale.value, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

// Watch tab changes
const handleTabClick = async (tabKey) => {
  isLoading.value = true
  clearError()
  isUpdatingRoute.value = true
  currentPage.value = 1 // Reset to first page on tab change

  try {
    await fetchData(tabKey)
    activeTab.value = tabKey
    await router.replace({
      query: { ...route.query, tab: tabKey, page: 1 },
    })
  } catch (e) {
    showError(t('userContributions.loadDataError')) // Use t()
  } finally {
    isLoading.value = false
    isUpdatingRoute.value = false
  }
}

// Reactive page title
const pageTitle = ref(t('userContributions.activityTitle', { username: props.username })) // Use t()
useSeoHead({ title: pageTitle, locale: locale.value })

// Configure tabs
const tabs = computed(() => [ // Make tabs computed
  {
    key: 'comments',
    label: t('userContributions.comments'), // Use t()
    icon: MessageSquare
  },
  {
    key: 'definitions',
    label: t('userContributions.definitions'), // Use t()
    icon: Book
  }
])

// Update title when tab changes or username changes
watch(
  [activeTab, () => props.username],
  ([newTab, newUsername]) => {
    const tabTitle = tabs.value.find(t => t.key === newTab)?.label || t('userContributions.activityTitle', { username: '' }) // Use tabs.value and t()
    pageTitle.value = t('userContributions.activityTitle', { username: newUsername }) + ` - ${tabTitle}` // Use t()
  },
  { immediate: true }
)

onMounted(() => {
  const initialTab = route.query.tab && ['comments', 'definitions'].includes(route.query.tab)
    ? route.query.tab
    : 'comments'
  activeTab.value = initialTab
  
  // Read page from URL query parameter
  const pageFromQuery = parseInt(route.query.page, 10)
  if (pageFromQuery && pageFromQuery >= 1) {
    currentPage.value = pageFromQuery
  }
  
  fetchData(initialTab)
})

// Watch for route query changes (e.g., browser back/forward)
watch(
  () => route.query,
  (newQuery, oldQuery) => {
    // Skip if this is the initial mount (oldQuery will be undefined)
    if (!oldQuery) return
    
    // Skip if we're programmatically updating the route (to prevent double fetches)
    if (isUpdatingRoute.value) return
    
    const newTab = newQuery.tab && ['comments', 'definitions'].includes(newQuery.tab)
      ? newQuery.tab
      : 'comments'
    
    const pageFromQuery = parseInt(newQuery.page, 10)
    const newPage = pageFromQuery && pageFromQuery >= 1 ? pageFromQuery : 1
    
    // Only update if something actually changed
    const tabChanged = newTab !== activeTab.value
    const pageChanged = newPage !== currentPage.value
    
    if (tabChanged || pageChanged) {
      if (tabChanged) {
        activeTab.value = newTab
        currentPage.value = 1
      } else if (pageChanged) {
        currentPage.value = newPage
      }
      fetchData(activeTab.value)
    }
  }
)
</script>
