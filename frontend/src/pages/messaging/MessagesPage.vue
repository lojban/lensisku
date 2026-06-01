<template>
  <div class="messages-page h-full flex flex-col bg-gray-50">
    <!-- Header -->
    <header class="bg-white border-b border-gray-200 px-4 py-3 sm:px-6">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <h1 class="text-xl font-semibold text-gray-900">Messages</h1>
          <div
            v-if="unreadCount > 0"
            class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800"
          >
            {{ unreadCount }} unread
          </div>
        </div>
        <button
          @click="showNewChatModal = true"
          class="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        >
          <Plus class="h-4 w-4 mr-1" />
          New Chat
        </button>
      </div>

      <!-- Search and Filter -->
      <div class="mt-4 flex flex-col sm:flex-row gap-3">
        <div class="flex-1 relative">
          <Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search conversations..."
            class="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            @input="handleSearch"
          />
        </div>
        <select
          v-model="filterType"
          class="px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          @change="handleFilter"
        >
          <option value="all">All</option>
          <option value="direct">Direct Messages</option>
          <option value="group">Group Chats</option>
        </select>
      </div>
    </header>

    <!-- Thread List -->
    <main class="flex-1 overflow-hidden">
      <div v-if="isLoading" class="flex items-center justify-center h-full">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>

      <div v-else-if="filteredThreads.length === 0" class="flex flex-col items-center justify-center h-full text-center px-4">
        <MessageCircle class="h-12 w-12 text-gray-400 mb-4" />
        <h3 class="text-lg font-medium text-gray-900 mb-2">
          {{ searchQuery ? 'No conversations found' : 'No conversations yet' }}
        </h3>
        <p class="text-gray-500 mb-4">
          {{ searchQuery ? 'Try adjusting your search terms' : 'Start a new conversation to get started' }}
        </p>
        <button
          v-if="!searchQuery"
          @click="showNewChatModal = true"
          class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700"
        >
          <Plus class="h-4 w-4 mr-2" />
          Start New Chat
        </button>
      </div>

      <div v-else class="h-full overflow-y-auto">
        <ThreadList
          :threads="filteredThreads"
          @thread-click="handleThreadClick"
          @thread-delete="handleThreadDelete"
        />
        
        <!-- Load More -->
        <div
          v-if="hasMore"
          class="p-4 text-center"
        >
          <button
            @click="loadMore"
            :disabled="isLoadingMore"
            class="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
          >
            <div v-if="isLoadingMore" class="animate-spin rounded-full h-4 w-4 border-b-2 border-gray-600 mr-2"></div>
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
    filtered = filtered.filter(thread => 
      thread.thread_name?.toLowerCase().includes(query) ||
      thread.participants?.some(p => p.username.toLowerCase().includes(query))
    )
  }

  // Apply type filter
  if (filterType.value !== 'all') {
    filtered = filtered.filter(thread => thread.thread_type === filterType.value)
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
      search: searchQuery.value || undefined
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
  // Debounce search
  clearTimeout(searchDebounce.value)
  searchDebounce.value = setTimeout(() => {
    fetchThreads(1, false)
  }, 300)
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
      threads.value = threads.value.filter(t => t.thread_id !== thread.thread_id)
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
const searchDebounce = ref<number>()

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
watch(() => auth.state.isLoggedIn, (isLoggedIn) => {
  if (isLoggedIn) {
    fetchThreads()
  } else {
    threads.value = []
  }
})
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
