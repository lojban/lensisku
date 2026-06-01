<template>
  <div class="thread-list">
    <div
      v-for="thread in threads"
      :key="thread.thread_id"
      @click="$emit('thread-click', thread)"
      class="thread-item flex items-center p-4 hover:bg-gray-50 cursor-pointer transition-colors duration-200 border-b border-gray-100"
      :class="{ 'bg-blue-50': thread.unread_count > 0 }"
    >
      <!-- Avatar -->
      <div class="flex-shrink-0 mr-3">
        <div class="relative">
          <div class="h-12 w-12 rounded-full bg-gradient-to-br from-blue-400 to-blue-600 flex items-center justify-center text-white font-medium">
            {{ getThreadInitials(thread) }}
          </div>
          <!-- Online indicator for direct messages -->
          <div
            v-if="thread.thread_type === 'direct' && isUserOnline(getOtherParticipantId(thread))"
            class="absolute -bottom-0.5 -right-0.5 h-4 w-4 rounded-full border-2 border-white bg-green-400"
          />
        </div>
      </div>

      <!-- Thread Info -->
      <div class="flex-1 min-w-0">
        <div class="flex items-center justify-between mb-1">
          <h3 class="text-sm font-medium text-gray-900 truncate">
            {{ getThreadDisplayName(thread) }}
          </h3>
          <span class="text-xs text-gray-500 whitespace-nowrap ml-2">
            {{ formatTime(thread.last_message_at || thread.updated_at) }}
          </span>
        </div>
        
        <div class="flex items-center justify-between">
          <p class="text-sm text-gray-600 truncate">
            {{ thread.last_message_preview || 'No messages yet' }}
          </p>
          <div class="flex items-center space-x-2 ml-2">
            <!-- Unread count badge -->
            <div
              v-if="thread.unread_count > 0"
              class="inline-flex items-center justify-center px-2 py-1 text-xs font-medium text-white bg-blue-600 rounded-full"
              :class="{ 'h-5 w-5 text-xs': thread.unread_count <= 9, 'px-2': thread.unread_count > 9 }"
            >
              {{ thread.unread_count > 99 ? '99+' : thread.unread_count }}
            </div>
            
            <!-- Thread type indicator -->
            <div class="flex items-center text-gray-400">
              <Users v-if="thread.thread_type === 'group'" class="h-3 w-3" />
              <User v-else class="h-3 w-3" />
            </div>
          </div>
        </div>

        <!-- Participants for group chats -->
        <div
          v-if="thread.thread_type === 'group' && thread.participants"
          class="mt-1 flex items-center text-xs text-gray-500"
        >
          <Users class="h-3 w-3 mr-1" />
          {{ thread.participant_count }} members
        </div>
      </div>

      <!-- Actions -->
      <div class="flex-shrink-0 ml-3">
        <div class="relative">
          <button
            @click.stop="toggleThreadMenu(thread)"
            class="p-1 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100 transition-colors"
          >
            <MoreVertical class="h-4 w-4" />
          </button>
          
          <!-- Dropdown menu -->
          <div
            v-if="activeMenu === thread.thread_id"
            class="absolute right-0 top-full z-10 mt-1 w-48 rounded-md bg-white shadow-lg ring-1 ring-black ring-opacity-5"
          >
            <div class="py-1">
              <button
                @click.stop="markAsRead(thread)"
                class="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
              >
                Mark as read
              </button>
              <button
                @click.stop="muteThread(thread)"
                class="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
              >
                Mute notifications
              </button>
              <button
                @click.stop="$emit('thread-delete', thread)"
                class="w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-gray-100"
              >
                Delete conversation
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-if="threads.length === 0"
      class="flex flex-col items-center justify-center py-12 text-center"
    >
      <MessageCircle class="h-12 w-12 text-gray-400 mb-4" />
      <h3 class="text-lg font-medium text-gray-900 mb-2">No conversations</h3>
      <p class="text-gray-500">Start a new conversation to see it here</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { MessageCircle, Users, User, MoreVertical } from 'lucide-vue-next'
import { useAuth } from '@/composables/useAuth'
import { usePresence } from '@/services/messaging/PresenceService'
import type { Thread } from '@/types/messaging'

interface Props {
  threads: Thread[]
}

defineProps<Props>()

const emit = defineEmits<{
  'thread-click': [thread: Thread]
  'thread-delete': [thread: Thread]
}>()

const auth = useAuth()
const { isUserOnline } = usePresence()

// Reactive state
const activeMenu = ref<number | null>(null)

// Methods
const getThreadDisplayName = (thread: Thread): string => {
  if (thread.thread_name) {
    return thread.thread_name
  }
  
  if (thread.thread_type === 'direct' && thread.participants) {
    const otherParticipant = thread.participants.find(p => p.user_id !== (auth.state.username as any))
    return otherParticipant?.username || 'Unknown User'
  }
  
  return thread.thread_type === 'group' ? 'Group Chat' : 'Unknown'
}

const getThreadInitials = (thread: Thread): string => {
  const name = getThreadDisplayName(thread)
  
  if (thread.thread_type === 'group') {
    return name
      .split(' ')
      .map(word => word[0])
      .join('')
      .toUpperCase()
      .slice(0, 2)
  }
  
  // For direct messages, use the other user's initial
  return name[0]?.toUpperCase() || '?'
}

const getOtherParticipantId = (thread: Thread): number => {
  if (thread.thread_type === 'direct' && thread.participants) {
    const otherParticipant = thread.participants.find(p => p.user_id !== (auth.state.username as any))
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

const markAsRead = (thread: Thread) => {
  // Implementation would call API to mark all messages as read
  activeMenu.value = null
}

const muteThread = (thread: Thread) => {
  // Implementation would call API to mute notifications
  activeMenu.value = null
}

// Close menu when clicking outside
const handleClickOutside = () => {
  activeMenu.value = null
}

// Add click outside listener
if (typeof window !== 'undefined') {
  document.addEventListener('click', handleClickOutside)
}
</script>

<style scoped>
.thread-item {
  @apply transition-all duration-200;
}

.thread-item:hover {
  @apply transform translate-x-1;
}

.thread-item:active {
  @apply transform scale-98;
}

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
