<template>
  <div class="relative">
    <NavLink
      to="/messages"
      class="navbar-item relative"
      :class="{ 'text-blue-600': isActive }"
      :aria-label="`Messages${unreadCount > 0 ? ` (${unreadCount} unread)` : ''}`"
    >
      <div class="relative">
        <MessageCircle class="h-5 w-5" />
        <!-- Notification badge -->
        <div
          v-if="unreadCount > 0"
          class="absolute -top-1 -right-1 inline-flex h-5 w-5 items-center justify-center rounded-full bg-red-500 text-xs font-medium text-white"
          :class="unreadCount > 99 ? 'h-6 w-6' : ''"
        >
          {{ unreadCount > 99 ? '99+' : unreadCount }}
        </div>
      </div>
      <span class="sr-only">Messages</span>
    </NavLink>

    <!-- Quick notification dropdown for desktop -->
    <div
      v-if="showNotifications && recentNotifications.length > 0"
      class="card-base card-elevated absolute right-0 top-full z-50 mt-2 w-80 overflow-hidden"
    >
      <div class="p-3 border-b border-gray-100">
        <h3 class="text-sm font-medium text-gray-900">Recent Messages</h3>
      </div>
      <div class="max-h-96 overflow-y-auto">
        <button
          v-for="notification in recentNotifications"
          :key="notification.id"
          type="button"
          class="flex items-start gap-3 w-full p-3 hover:bg-gray-100 transition-colors text-left"
          @click="handleNotificationClick(notification)"
        >
          <div class="shrink-0">
            <div class="avatar-placeholder-sm !h-8 !w-8">
              <User class="h-4 w-4 text-gray-500" />
            </div>
          </div>
          <div class="min-w-0 flex-1">
            <p class="text-sm font-medium text-gray-900 truncate">
              {{ notification.sender_name }}
            </p>
            <p class="card-description truncate">
              {{ notification.preview }}
            </p>
            <p class="card-meta-date mt-1">
              {{ formatTime(notification.created_at) }}
            </p>
          </div>
          <div
            v-if="!notification.is_read"
            class="shrink-0 h-2 w-2 rounded-full bg-blue-500 mt-2"
          />
        </button>
      </div>
      <div class="p-2 border-t border-gray-100">
        <button type="button" class="ui-btn--link w-full" @click="goToMessages">
          View all messages
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { MessageCircle, User } from 'lucide-vue-next'
import NavLink from '@/components/NavLink.vue'
import { useAuth } from '@/composables/useAuth'
import { getThreads } from '@/services/messaging/messagingApi'
import { webSocketService } from '@/services/messaging/WebSocketService'
import type { Thread } from '@/types/messaging'

interface MessageNotification {
  id: string
  sender_name: string
  preview: string
  thread_id: number
  created_at: string
  is_read: boolean
}

const router = useRouter()
const route = useRoute()
const auth = useAuth()

// Reactive state
const unreadCount = ref(0)
const recentNotifications = ref<MessageNotification[]>([])
const showNotifications = ref(false)
const threads = ref<Thread[]>([])

// Computed properties
const isActive = computed(() => route.path.startsWith('/messages'))

// Methods
const fetchUnreadCount = async () => {
  try {
    const response = await getThreads({ per_page: 50 })
    threads.value = response.threads

    // Calculate total unread count
    unreadCount.value = response.threads.reduce((total, thread) => total + thread.unread_count, 0)

    // Create recent notifications from threads with unread messages
    recentNotifications.value = response.threads
      .filter((thread) => thread.unread_count > 0)
      .slice(0, 5)
      .map((thread) => ({
        id: `thread-${thread.thread_id}`,
        sender_name: getOtherParticipantName(thread),
        preview: thread.last_message_preview || 'New message',
        thread_id: thread.thread_id,
        created_at: thread.last_message_at || thread.updated_at,
        is_read: thread.unread_count === 0,
      }))
  } catch (error) {
    console.error('Failed to fetch unread count:', error)
  }
}

const getOtherParticipantName = (thread: Thread): string => {
  // For direct messages, return the other participant's name
  if (thread.thread_type === 'direct' && thread.participants) {
    const otherParticipant = thread.participants.find(
      (p) => p.user_id !== (auth.state.username as unknown)
    )
    return otherParticipant?.username || 'Unknown'
  }
  // For group messages, return thread name or "Group Chat"
  return thread.thread_name || 'Group Chat'
}

const handleNotificationClick = (notification: MessageNotification) => {
  router.push(`/messages/${notification.thread_id}`)
  showNotifications.value = false
}

const goToMessages = () => {
  router.push('/messages')
  showNotifications.value = false
}

const formatTime = (timestamp: string): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)

  if (diffMins < 1) {
    return 'Just now'
  } else if (diffMins < 60) {
    return `${diffMins}m ago`
  } else if (diffMins < 1440) {
    const diffHours = Math.floor(diffMins / 60)
    return `${diffHours}h ago`
  } else {
    return date.toLocaleDateString()
  }
}

// WebSocket event handlers
const handleNewMessage = () => {
  fetchUnreadCount()
}

const handleThreadUpdate = () => {
  fetchUnreadCount()
}

// Lifecycle
onMounted(async () => {
  if (auth.state.isLoggedIn) {
    await fetchUnreadCount()

    // Set up WebSocket listeners
    webSocketService.on('message:new', handleNewMessage)
    webSocketService.on('thread:updated', handleThreadUpdate)

    // Connect to WebSocket for real-time updates
    try {
      await webSocketService.connect()
    } catch (error) {
      console.warn('Failed to connect WebSocket for messaging:', error)
    }
  }
})

onUnmounted(() => {
  // Clean up WebSocket listeners
  webSocketService.off('message:new', handleNewMessage)
  webSocketService.off('thread:updated', handleThreadUpdate)
})

// Close dropdown when clicking outside
const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as Element
  if (!target.closest('.relative')) {
    showNotifications.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>
