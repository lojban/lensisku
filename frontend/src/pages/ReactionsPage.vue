<template>
  <TabbedPageHeader :tabs="tabs" :active-tab="activeTab" :page-title="pageTitle" @tab-click="handleTabClick" />

  <!-- Loading State with Skeleton -->
  <div v-if="isLoading" class="space-y-4">
    <SkeletonActivityItem v-for="n in 5" :key="n" />
  </div>

  <!-- Content -->
  <div v-else class="space-y-4">
    <ActivityBookmarks v-if="activeTab === 'bookmarked'" :comments="bookmarks" :format-date="formatDate" :no-items-message="t('reactionsPage.noBookmarks')" />
    <ActivityReactions v-else-if="activeTab === 'reactions'" :comments="reactions" :format-date="formatDate" :no-items-message="t('reactionsPage.noReactions')" />
    <ActivityComments v-else-if="activeTab === 'comments'" :comments="comments" :format-date="formatDate" :no-items-message="t('reactionsPage.noComments')" />
    <ActivityDefinitions v-else-if="activeTab === 'definitions'" :definitions="definitions" :format-date="formatDate" :no-items-message="t('reactionsPage.noDefinitions')" />
    <ActivityVotes v-else-if="activeTab === 'votes'" :votes="votes" :format-date="formatDate" :no-items-message="t('reactionsPage.noVotes')" />

    <!-- PaginationComponent -->
    <div v-if="total > perPage">
      <PaginationComponent :current-page="currentPage" :total-pages="totalPages" :total="total" :per-page="perPage"
        @prev="() => changePage(currentPage - 1)" @next="() => changePage(currentPage + 1)" />
    </div>
  </div>
</template>

<script setup>
import { MessageSquare, Book, Vote, BookmarkCheck } from 'lucide-vue-next'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'

import { getBookmarks, getMyReactions, getUserComments, getUserDefinitions, getUserVotes } from '@/api'
import ActivityBookmarks from '@/components/activity/ActivityBookmarks.vue'
import ActivityComments from '@/components/activity/ActivityComments.vue'
import ActivityDefinitions from '@/components/activity/ActivityDefinitions.vue'
import ActivityReactions from '@/components/activity/ActivityReactions.vue'
import ActivityVotes from '@/components/activity/ActivityVotes.vue'
import ReactionIcon from '@/components/icons/ReactionIcon.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import SkeletonActivityItem from '@/components/activity/SkeletonActivityItem.vue'
import TabbedPageHeader from '@/components/TabbedPageHeader.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const router = useRouter()
const route = useRoute()
const { t, locale } = useI18n()

const { showError, clearError } = useError()

// State
const activeTab = ref('bookmarked')
const comments = ref([])
const definitions = ref([])
const votes = ref([])
const isLoading = ref(false)
const bookmarks = ref([])
const reactions = ref([])

// PaginationComponent state
const currentPage = ref(1)
const perPage = ref(10)
const total = ref(0)

const auth = useAuth()

const totalPages = computed(() => Math.ceil(total.value / perPage.value))

const formatDate = (timestamp) => {
  return new Date(timestamp).toLocaleString(locale.value, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}
const fetchData = async (tabKey) => {
  isLoading.value = true
  clearError()

  try {
    let response
    switch (tabKey) {
      case 'bookmarked':
        response = await getBookmarks({ page: currentPage.value, per_page: perPage.value })
        bookmarks.value = response.data.items || response.data.comments || []
        break
      case 'reactions':
        response = await getMyReactions({ page: currentPage.value, per_page: perPage.value })
        reactions.value = response.data.items || response.data.comments || []
        break
      case 'comments':
        if (!auth.state.username) {
          console.warn(t('reactionsPage.warnNoUsernameComments')) // Use t()
          comments.value = [] // Clear or handle appropriately
          total.value = 0
          break // Skip API call
        }
        response = await getUserComments(auth.state.username, {
          page: currentPage.value,
          per_page: perPage.value
        })
        comments.value = response.data.items
        break
      case 'definitions':
        if (!auth.state.username) {
          console.warn(t('reactionsPage.warnNoUsernameDefinitions')) // Use t()
          definitions.value = [] // Clear or handle appropriately
          total.value = 0
          break // Skip API call
        }
        response = await getUserDefinitions(auth.state.username, {
          page: currentPage.value,
          per_page: perPage.value
        })
        definitions.value = response.data.items
        break
      case 'votes':
        response = await getUserVotes({
          page: currentPage.value,
          per_page: perPage.value
        })
        votes.value = response.data.items
        break
    }
    if (response) {
      total.value = response.data.total
      currentPage.value = response.data.page
      perPage.value = response.data.per_page
    }
    perPage.value = response.data.per_page
  } catch (e) {
    showError(e.response?.data?.error || t('reactionsPage.loadError')) // Use t()
  } finally {
    isLoading.value = false
  }
}

const changePage = (newPage) => {
  if (newPage >= 1 && newPage <= totalPages.value) {
    currentPage.value = newPage
    fetchData(activeTab.value)
  }
}

const handleTabClick = async (tabKey) => {
  isLoading.value = true
  clearError()
  currentPage.value = 1 // Reset to first page on tab change

  try {
    await fetchData(tabKey)
    activeTab.value = tabKey
    router.replace({
      query: { ...route.query, tab: tabKey },
    })
  } catch (e) {
    showError(e.response?.data?.error || t('reactionsPage.loadError')) // Use t()
  } finally {
    isLoading.value = false
  }
}

// Reactive page title
const tabs = computed(() => [ // Make tabs computed for reactivity with t()
  { key: 'bookmarked', label: t('reactionsPage.bookmarks'), icon: BookmarkCheck },
  { key: 'reactions', label: t('reactionsPage.reactions'), icon: ReactionIcon },
  { key: 'comments', label: t('reactionsPage.comments'), icon: MessageSquare },
  { key: 'definitions', label: t('reactionsPage.definitions'), icon: Book },
  { key: 'votes', label: t('reactionsPage.votes'), icon: Vote },
])

const pageTitle = ref(tabs.value[0].label)
useSeoHead({ title: pageTitle }, locale.value)

// Update title when tab changes
watch(
  activeTab,
  (newTab) => {
    pageTitle.value = `${tabs.value.find(t => t.key === newTab)?.label || t('reactionsPage.activity')}` // Use tabs.value and t()
  },
  { immediate: true }
)

onMounted(() => {
  // Wait for auth state to settle before fetching initial data
  watch(() => auth.state.isLoading, (loading) => {
    if (!loading) {
      // Determine initial tab *after* auth is loaded
      const initialTab = route.query.tab && ['bookmarked', 'reactions', 'comments', 'definitions', 'votes'].includes(route.query.tab)
        ? route.query.tab
        : 'bookmarked'
      activeTab.value = initialTab
      fetchData(initialTab)
    }
  }, { immediate: true })
})
</script>
