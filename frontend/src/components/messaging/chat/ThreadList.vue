<template>
  <div class="thread-list divide-y divide-gray-100">
    <div
      v-for="thread in threads"
      :key="thread.thread_id"
      class="message-thread-card message-thread-card--clickable flex items-center gap-3 p-3 sm:p-4"
      :class="{ 'bg-blue-50/60': thread.unread_count > 0 }"
      @click="$emit('thread-click', thread)"
    >
      <!-- Avatar -->
      <div class="shrink-0">
        <div class="relative">
          <div class="avatar-placeholder-sm">
            {{ getThreadInitials(thread) }}
          </div>
          <!-- Online indicator for direct messages -->
          <div
            v-if="thread.thread_type === 'direct' && isUserOnline(getOtherParticipantId(thread))"
            class="absolute -bottom-0.5 -right-0.5 h-3.5 w-3.5 rounded-full border-2 border-white bg-green-400"
          />
        </div>
      </div>

      <!-- Thread Info -->
      <div class="min-w-0 flex-1">
        <div class="flex items-center justify-between gap-2 mb-0.5">
          <h3 class="link-heading-primary">
            {{ getThreadDisplayName(thread) }}
          </h3>
          <span class="card-meta-date whitespace-nowrap">
            {{ formatTime(thread.last_message_at || thread.updated_at) }}
          </span>
        </div>

        <div class="flex items-center justify-between gap-2">
          <p class="card-description truncate">
            {{ thread.last_message_preview || 'No messages yet' }}
          </p>
          <div class="flex items-center gap-2 shrink-0">
            <!-- Unread count badge -->
            <span
              v-if="thread.unread_count > 0"
              class="inline-flex items-center justify-center rounded-full text-xs font-medium bg-blue-600 text-white"
              :class="thread.unread_count > 9 ? 'px-2 py-0.5' : 'h-5 w-5'"
            >
              {{ thread.unread_count > 99 ? '99+' : thread.unread_count }}
            </span>

            <!-- Thread type indicator -->
            <span class="text-gray-400">
              <Users v-if="thread.thread_type === 'group'" class="h-3 w-3" />
              <User v-else class="h-3 w-3" />
            </span>
          </div>
        </div>

        <!-- Participants for group chats -->
        <div
          v-if="thread.thread_type === 'group' && thread.participants"
          class="mt-1 flex items-center gap-1 text-xs text-gray-500"
        >
          <Users class="h-3 w-3" />
          {{ thread.participant_count }} members
        </div>
      </div>

      <!-- Actions -->
      <div class="shrink-0">
        <div class="relative">
          <button type="button" class="icon-btn-ghost" @click.stop="toggleThreadMenu(thread)">
            <MoreVertical class="h-4 w-4" />
          </button>

          <!-- Dropdown menu -->
          <div
            v-if="activeMenu === thread.thread_id"
            class="dropdown-menu-panel absolute right-0 top-full mt-1 w-48 py-1"
          >
            <button
              type="button"
              class="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
              @click.stop="markAsRead(thread)"
            >
              Mark as read
            </button>
            <button
              type="button"
              class="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
              @click.stop="muteThread(thread)"
            >
              Mute notifications
            </button>
            <button
              type="button"
              class="block w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-red-50"
              @click.stop="$emit('thread-delete', thread)"
            >
              Delete conversation
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="threads.length === 0" class="empty-state-panel">
      <MessageCircle class="h-12 w-12 text-gray-400 mb-4" />
      <h3 class="text-lg font-medium text-gray-900 mb-2">No conversations</h3>
      <p class="text-gray-500">Start a new conversation to see it here</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { MessageCircle, Users, User, MoreVertical } from 'lucide-vue-next'
import { useAuth } from '@/composables/useAuth'
import { usePresence } from '@/services/messaging/PresenceService'
import type { Thread } from '@/types/messaging'

interface Props {
  threads: Thread[]
}

defineProps<Props>()

defineEmits<{
  'thread-click': [thread: Thread]
  'thread-delete': [thread: Thread]
}>()

const auth = useAuth()
const { isUserOnline } = usePresence()

// Reactive state
const activeMenu = ref<number | null>(null)
let clickOutsideCleanup: (() => void) | null = null

// Methods
const getThreadDisplayName = (thread: Thread): string => {
  if (thread.thread_name) {
    return thread.thread_name
  }

  if (thread.thread_type === 'direct' && thread.participants) {
    const otherParticipant = thread.participants.find(
      (p) => p.user_id !== (auth.state.username as unknown)
    )
    return otherParticipant?.username || 'Unknown User'
  }

  return thread.thread_type === 'group' ? 'Group Chat' : 'Unknown'
}

const getThreadInitials = (thread: Thread): string => {
  const name = getThreadDisplayName(thread)

  if (thread.thread_type === 'group') {
    return name
      .split(' ')
      .map((word) => word[0])
      .join('')
      .toUpperCase()
      .slice(0, 2)
  }

  // For direct messages, use the other user's initial
  return name[0]?.toUpperCase() || '?'
}

const getOtherParticipantId = (thread: Thread): number => {
  if (thread.thread_type === 'direct' && thread.participants) {
    const otherParticipant = thread.participants.find(
      (p) => p.user_id !== (auth.state.username as unknown)
    )
    return otherParticipant?.user_id || 0
  }
  return 0
}

const formatTime = (timestamp: string): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMins / 60)
  const diffDays = Math.floor(diffHours / 24)

  if (diffMins < 1) {
    return 'now'
  } else if (diffMins < 60) {
    return `${diffMins}m`
  } else if (diffHours < 24) {
    return `${diffHours}h`
  } else if (diffDays < 7) {
    return `${diffDays}d`
  } else {
    return date.toLocaleDateString()
  }
}

const toggleThreadMenu = (thread: Thread) => {
  if (activeMenu.value === thread.thread_id) {
    activeMenu.value = null
  } else {
    activeMenu.value = thread.thread_id
  }
}

const markAsRead = (_thread: Thread) => {
  // Implementation would call API to mark all messages as read
  activeMenu.value = null
}

const muteThread = (_thread: Thread) => {
  // Implementation would call API to mute notifications
  activeMenu.value = null
}

// Close menu when clicking outside
const handleClickOutside = () => {
  activeMenu.value = null
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  clickOutsideCleanup = () => document.removeEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  clickOutsideCleanup?.()
})
</script>

<style scoped>
/* Custom scrollbar for thread list */
.thread-list {
  scrollbar-width: thin;
  scrollbar-color: #e5e7eb #f9fafb;
}

.thread-list::-webkit-scrollbar {
  width: 6px;
}

.thread-list::-webkit-scrollbar-track {
  background: #f9fafb;
}

.thread-list::-webkit-scrollbar-thumb {
  background-color: #e5e7eb;
  border-radius: 3px;
}

.thread-list::-webkit-scrollbar-thumb:hover {
  background-color: #d1d5db;
}
</style>
