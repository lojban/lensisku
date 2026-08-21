<template>
  <TabbedPageHeader
    :tabs="tabs"
    :active-tab="activeTab"
    :page-title="pageTitle"
    @tab-click="handleTabClick"
  />
  <ProfilePage v-if="activeTab === 'profile'" embedded />
  <div v-else-if="isLoading" class="space-y-4">
    <SkeletonActivityItem v-for="n in 5" :key="n" />
  </div>
  <div v-else class="space-y-4">
    <ActivityComments
      v-if="activeTab === 'comments'"
      :comments="comments"
      :format-date="formatDate"
    />
    <ActivityBookmarks
      v-else-if="activeTab === 'bookmarked'"
      :comments="bookmarks"
      :format-date="formatDate"
      :no-items-message="t('reactionsPage.noBookmarks')"
    />
    <ActivityReactions
      v-else-if="activeTab === 'reactions'"
      :comments="reactions"
      :format-date="formatDate"
      :no-items-message="t('reactionsPage.noReactions')"
    />
    <ActivityVotes
      v-else-if="activeTab === 'votes'"
      :votes="votes"
      :format-date="formatDate"
      :no-items-message="t('reactionsPage.noVotes')"
    />
    <div v-if="activeTab !== 'profile' && total > perPage">
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

<script setup lang="ts">
import { Vote, BookmarkCheck, User, MessageSquare } from '@lucide/vue'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'

import { getBookmarks, getMyReactions, getUserComments, getUserVotes } from '@/api'
import ActivityBookmarks from '@/components/activity/ActivityBookmarks.vue'
import ActivityComments from '@/components/activity/ActivityComments.vue'
import ActivityReactions from '@/components/activity/ActivityReactions.vue'
import ActivityVotes from '@/components/activity/ActivityVotes.vue'
import ReactionIcon from '@/components/icons/ReactionIcon.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import SkeletonActivityItem from '@/components/activity/SkeletonActivityItem.vue'
import TabbedPageHeader from '@/components/TabbedPageHeader.vue'
import ProfilePage from '@/pages/ProfilePage.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'
import { queryStr } from '@/utils/routeQuery'

const ME_TABS = ['profile', 'comments', 'votes', 'reactions', 'bookmarked']

const router = useRouter()
const route = useRoute()
const { t, locale } = useI18n()
const { showError, clearError } = useError()
const auth = useAuth()

const activeTab = ref('profile')
const votes = ref([])
const isLoading = ref(false)
const bookmarks = ref([])
const comments = ref([])
const reactions = ref([])
const currentPage = ref(1)
const perPage = ref(10)
const total = ref(0)

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

function normalizeTab(tabQ: string | undefined) {
  if (tabQ === 'bookmarks') return 'bookmarked'
  if (tabQ && ME_TABS.includes(tabQ)) return tabQ
  return 'profile'
}

const fetchData = async (tabKey) => {
  if (tabKey === 'profile') {
    isLoading.value = false
    return
  }
  isLoading.value = true
  clearError()

  try {
    let response
    switch (tabKey) {
      case 'comments': {
        const username = auth.state.username
        if (!username) {
          showError(t('reactionsPage.warnNoUsernameComments'))
          break
        }
        response = await getUserComments(username, {
          page: currentPage.value,
          per_page: perPage.value,
        })
        comments.value = (response.data.items || []).map((comment) => ({
          ...comment,
          username,
        }))
        break
      }
      case 'bookmarked':
        response = await getBookmarks({ page: currentPage.value, per_page: perPage.value })
        bookmarks.value = response.data.items || response.data.comments || []
        break
      case 'reactions':
        response = await getMyReactions({ page: currentPage.value, per_page: perPage.value })
        reactions.value = response.data.items || response.data.comments || []
        break
      case 'votes':
        response = await getUserVotes({
          page: currentPage.value,
          per_page: perPage.value,
        })
        votes.value = response.data.items
        break
      default:
        break
    }
    if (response) {
      total.value = response.data.total
      currentPage.value = response.data.page
      perPage.value = response.data.per_page
    }
  } catch (e) {
    showError(e.response?.data?.error || t('reactionsPage.loadError'))
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
  currentPage.value = 1
  activeTab.value = tabKey
  await router.replace({
    query: { ...route.query, tab: tabKey },
  })
  await fetchData(tabKey)
}

const tabs = computed(() => [
  { key: 'profile', label: t('mePage.profile'), icon: User },
  { key: 'comments', label: t('reactionsPage.comments'), icon: MessageSquare },
  { key: 'reactions', label: t('reactionsPage.reactions'), icon: ReactionIcon },
  { key: 'bookmarked', label: t('reactionsPage.bookmarks'), icon: BookmarkCheck },
  { key: 'votes', label: t('reactionsPage.votes'), icon: Vote },
])

const pageTitle = ref(t('mePage.profile'))
useSeoHead({ title: pageTitle, pathWithoutLocale: '/mi' })

watch(
  activeTab,
  (newTab) => {
    pageTitle.value = `${tabs.value.find((tab) => tab.key === newTab)?.label || t('nav.me')}`
  },
  { immediate: true }
)

onMounted(() => {
  watch(
    () => auth.state.isLoading,
    (loading) => {
      if (!loading) {
        const initialTab = normalizeTab(queryStr(route.query.tab))
        activeTab.value = initialTab
        fetchData(initialTab)
      }
    },
    { immediate: true }
  )
})
</script>
