<template>
  <div class="messages-page page-sections h-full min-h-0">
    <!-- Header -->
    <header class="page-header-shell">
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-3">
          <h1 class="page-section-title">Messages</h1>
          <span v-if="unreadCount > 0" class="badge badge-muted"> {{ unreadCount }} unread </span>
        </div>
        <button type="button" class="ui-btn--create" @click="showNewChatModal = true">
          <Plus class="h-4 w-4" />
          New Chat
        </button>
      </div>

      <!-- Search and Filter -->
      <div class="toolbar-row mt-4">
        <div class="toolbar-search-slot relative">
          <Search
            class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400"
          />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search conversations..."
            class="input-field pl-9"
            @input="handleSearch"
          />
        </div>
        <select v-model="filterType" class="input-field" @change="handleFilter">
          <option value="all">All</option>
          <option value="direct">Direct Messages</option>
          <option value="group">Group Chats</option>
        </select>
      </div>
    </header>

    <!-- Thread List -->
    <main class="assistant-messages-pane flex flex-col min-h-0 flex-1 overflow-hidden">
      <div v-if="isLoading" class="card-study-area">
        <span
          class="inline-block h-8 w-8 animate-spin rounded-full border-b-2 border-blue-600"
          aria-hidden="true"
        />
      </div>

      <div v-else-if="filteredThreads.length === 0" class="empty-state-panel flex-1 justify-center">
        <MessageCircle class="h-12 w-12 text-gray-400 mb-4" />
        <h3 class="text-lg font-medium text-gray-900 mb-2">
          {{ searchQuery ? 'No conversations found' : 'No conversations yet' }}
        </h3>
        <p class="text-gray-500 mb-4 max-w-xs">
          {{
            searchQuery
              ? 'Try adjusting your search terms'
              : 'Start a new conversation to get started'
          }}
        </p>
        <button
          v-if="!searchQuery"
          type="button"
          class="ui-btn--create"
          @click="showNewChatModal = true"
        >
          <Plus class="h-4 w-4" />
          Start New Chat
        </button>
      </div>

      <div v-else class="flex flex-col min-h-0 flex-1 overflow-hidden">
        <div class="flex-1 overflow-y-auto min-h-0">
          <ThreadList
            :threads="filteredThreads"
            @thread-click="handleThreadClick"
            @thread-delete="handleThreadDelete"
          />
        </div>

        <!-- Load More -->
        <div v-if="hasMore" class="p-4 text-center shrink-0 border-t border-gray-200">
          <button type="button" :disabled="isLoadingMore" class="ui-btn--empty" @click="loadMore">
            <span
              v-if="isLoadingMore"
              class="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
              aria-hidden="true"
            ></span>
            {{ isLoadingMore ? 'Loading...' : 'Load More' }}
          </button>
        </div>
      </div>
    </main>

    <!-- New Chat Modal -->
    <NewChatModal
      v-if="showNewChatModal"
      @close="showNewChatModal = false"
      @thread-created="handleThreadCreated"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { MessageCircle, Plus, Search } from 'lucide-vue-next'
import { useAuth } from '@/composables/useAuth'
import { getThreads, deleteThread } from '@/services/messaging/messagingApi'
import { webSocketService } from '@/services/messaging/WebSocketService'
import { debounce } from '@/utils/mobileOptimization'
import ThreadList from '@/components/messaging/chat/ThreadList.vue'
import NewChatModal from '@/components/messaging/chat/NewChatModal.vue'
import type { Thread, GetThreadsQuery } from '@/types/messaging'

const router = useRouter()
const auth = useAuth()

// Reactive state
const threads = ref<Thread[]>([])
const isLoading = ref(true)
const isLoadingMore = ref(false)
const searchQuery = ref('')

// Mobile-optimized search with debounce
const debouncedSearch = debounce((_query: string) => {
  fetchThreads(1, false)
}, 300)

const filterType = ref<'all' | 'direct' | 'group'>('all')
const showNewChatModal = ref(false)
const currentPage = ref(1)
const hasMore = ref(true)
const total = ref(0)

// Computed properties
const filteredThreads = computed(() => {
  let filtered = threads.value

  // Apply search filter
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(
      (thread) =>
        thread.thread_name?.toLowerCase().includes(query) ||
        thread.participants?.some((p) => p.username.toLowerCase().includes(query))
    )
  }

  // Apply type filter
  if (filterType.value !== 'all') {
    filtered = filtered.filter((thread) => thread.thread_type === filterType.value)
  }

  return filtered
})

const unreadCount = computed(() => {
  return threads.value.reduce((total, thread) => total + thread.unread_count, 0)
})

// Methods
const fetchThreads = async (page: number = 1, append: boolean = false) => {
  try {
    if (!append) {
      isLoading.value = true
    } else {
      isLoadingMore.value = true
    }

    const query: GetThreadsQuery = {
      page,
      per_page: 20,
      thread_type: filterType.value !== 'all' ? filterType.value : undefined,
      search: searchQuery.value || undefined,
    }

    const response = await getThreads(query)

    if (append) {
      threads.value.push(...response.threads)
    } else {
      threads.value = response.threads
    }

    total.value = response.total
    hasMore.value = response.has_more
    currentPage.value = page
  } catch (error) {
    console.error('Failed to fetch threads:', error)
  } finally {
    isLoading.value = false
    isLoadingMore.value = false
  }
}

const loadMore = async () => {
  if (hasMore.value && !isLoadingMore.value) {
    await fetchThreads(currentPage.value + 1, true)
  }
}

const handleSearch = () => {
  debouncedSearch(searchQuery.value)
}

const handleFilter = () => {
  fetchThreads(1, false)
}

const handleThreadClick = (thread: Thread) => {
  router.push(`/messages/${thread.thread_id}`)
}

const handleThreadDelete = async (thread: Thread) => {
  if (confirm(`Are you sure you want to delete this conversation?`)) {
    try {
      await deleteThread(thread.thread_id)
      threads.value = threads.value.filter((t) => t.thread_id !== thread.thread_id)
    } catch (error) {
      console.error('Failed to delete thread:', error)
    }
  }
}

const handleThreadCreated = (thread: Thread) => {
  showNewChatModal.value = false
  router.push(`/messages/${thread.thread_id}`)
}

// WebSocket event handlers
const handleNewMessage = () => {
  fetchThreads(1, false)
}

const handleThreadUpdate = () => {
  fetchThreads(1, false)
}

// Debounce for search
const _searchDebounce = ref<number>()

// Lifecycle
onMounted(async () => {
  if (auth.state.isLoggedIn) {
    await fetchThreads()

    // Set up WebSocket listeners
    webSocketService.on('message:new', handleNewMessage)
    webSocketService.on('thread:updated', handleThreadUpdate)

    // Connect to WebSocket
    try {
      await webSocketService.connect()
    } catch (error) {
      console.warn('Failed to connect WebSocket:', error)
    }
  }
})

// Watch for auth changes
watch(
  () => auth.state.isLoggedIn,
  (isLoggedIn) => {
    if (isLoggedIn) {
      fetchThreads()
    } else {
      threads.value = []
    }
  }
)
</script>

<style scoped>
.messages-page {
  @apply min-h-screen;
}

@media (max-width: 640px) {
  .messages-page {
    @apply h-screen;
  }
}
</style>
