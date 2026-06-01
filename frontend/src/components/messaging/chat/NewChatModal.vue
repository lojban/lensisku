<template>
  <div class="fixed inset-0 z-50 overflow-y-auto">
    <div class="flex items-center justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
      <!-- Background overlay -->
      <div class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" @click="$emit('close')"></div>

      <!-- Modal panel -->
      <div class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
        <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
          <div class="sm:flex sm:items-start">
            <div class="w-full">
              <h3 class="text-lg leading-6 font-medium text-gray-900 mb-4">
                Start New Conversation
              </h3>

              <!-- Chat Type Selection -->
              <div class="mb-4">
                <label class="block text-sm font-medium text-gray-700 mb-2">
                  Chat Type
                </label>
                <div class="flex space-x-4">
                  <label class="flex items-center">
                    <input
                      v-model="chatType"
                      type="radio"
                      value="direct"
                      class="mr-2"
                    >
                    <span class="text-sm text-gray-700">Direct Message</span>
                  </label>
                  <label class="flex items-center">
                    <input
                      v-model="chatType"
                      type="radio"
                      value="group"
                      class="mr-2"
                    >
                    <span class="text-sm text-gray-700">Group Chat</span>
                  </label>
                </div>
              </div>

              <!-- Thread Name (for group chats) -->
              <div v-if="chatType === 'group'" class="mb-4">
                <label for="thread-name" class="block text-sm font-medium text-gray-700 mb-2">
                  Group Name
                </label>
                <input
                  id="thread-name"
                  v-model="threadName"
                  type="text"
                  placeholder="Enter group name"
                  class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                >
              </div>

              <!-- Participants Search -->
              <div class="mb-4">
                <label for="participants" class="block text-sm font-medium text-gray-700 mb-2">
                  Participants
                </label>
                <div class="relative">
                  <Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                  <input
                    id="participants"
                    v-model="searchQuery"
                    type="text"
                    placeholder="Search users..."
                    class="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    @input="handleSearch"
                  >
                </div>
              </div>

              <!-- Selected Participants -->
              <div v-if="selectedParticipants.length > 0" class="mb-4">
                <div class="flex flex-wrap gap-2">
                  <div
                    v-for="participant in selectedParticipants"
                    :key="participant.user_id"
                    class="inline-flex items-center px-3 py-1 rounded-full text-sm bg-blue-100 text-blue-800"
                  >
                    {{ participant.username }}
                    <button
                      @click="removeParticipant(participant.user_id)"
                      class="ml-2 text-blue-600 hover:text-blue-800"
                    >
                      <X class="h-3 w-3" />
                    </button>
                  </div>
                </div>
              </div>

              <!-- User Search Results -->
              <div v-if="searchResults.length > 0" class="mb-4">
                <div class="max-h-48 overflow-y-auto border border-gray-200 rounded-md">
                  <div
                    v-for="user in searchResults"
                    :key="user.user_id"
                    @click="toggleParticipant(user)"
                    class="flex items-center p-3 hover:bg-gray-50 cursor-pointer border-b border-gray-100 last:border-b-0"
                  >
                    <input
                      type="checkbox"
                      :checked="isParticipantSelected(user.user_id)"
                      class="mr-3"
                      @click.stop
                    >
                    <div class="h-8 w-8 rounded-full bg-gray-300 flex items-center justify-center text-sm font-medium text-gray-600 mr-3">
                      {{ user.username[0]?.toUpperCase() }}
                    </div>
                    <div class="flex-1">
                      <p class="text-sm font-medium text-gray-900">{{ user.username }}</p>
                      <p class="text-xs text-gray-500">{{ user.is_online ? 'Online' : 'Offline' }}</p>
                    </div>
                  </div>
                </div>
              </div>

              <!-- No Results -->
              <div v-if="searchQuery && searchResults.length === 0 && !isSearching" class="text-center py-4 text-gray-500">
                No users found
              </div>
            </div>
          </div>
        </div>

        <!-- Modal Actions -->
        <div class="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
          <button
            @click="createThread"
            :disabled="!canCreate || isCreating"
            class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-blue-600 text-base font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <div v-if="isCreating" class="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
            {{ isCreating ? 'Creating...' : 'Create Chat' }}
          </button>
          <button
            @click="$emit('close')"
            class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Search, X } from 'lucide-vue-next'
import { useAuth } from '@/composables/useAuth'
import { createThread as createThreadApi } from '@/services/messaging/messagingApi'
import type { Thread } from '@/types/messaging'

interface User {
  user_id: number
  username: string
  is_online: boolean
}

const emit = defineEmits<{
  close: []
  'thread-created': [thread: Thread]
}>()

const auth = useAuth()

// Reactive state
const chatType = ref<'direct' | 'group'>('direct')
const threadName = ref('')
const searchQuery = ref('')
const searchResults = ref<User[]>([])
const selectedParticipants = ref<User[]>([])
const isSearching = ref(false)
const isCreating = ref(false)

// Computed properties
const canCreate = computed(() => {
  if (chatType.value === 'direct') {
    return selectedParticipants.value.length === 1
  } else {
    return selectedParticipants.value.length >= 2 && threadName.value.trim().length > 0
  }
})

// Methods
const handleSearch = async () => {
  if (!searchQuery.value.trim()) {
    searchResults.value = []
    return
  }

  isSearching.value = true
  try {
    // Mock search - in real implementation, call user search API
    await new Promise(resolve => setTimeout(resolve, 300))
    
    // Mock data - replace with actual API call
    searchResults.value = [
      { user_id: 2, username: 'alice', is_online: true },
      { user_id: 3, username: 'bob', is_online: false },
      { user_id: 4, username: 'charlie', is_online: true },
    ].filter(user => 
      user.username.toLowerCase().includes(searchQuery.value.toLowerCase()) &&
      user.user_id !== (auth.state.username as any)
    )
  } catch (error) {
    console.error('Failed to search users:', error)
    searchResults.value = []
  } finally {
    isSearching.value = false
  }
}

const toggleParticipant = (user: User) => {
  if (isParticipantSelected(user.user_id)) {
    removeParticipant(user.user_id)
  } else {
    if (chatType.value === 'direct') {
      // For direct messages, only allow one participant
      selectedParticipants.value = [user]
    } else {
      selectedParticipants.value.push(user)
    }
  }
}

const isParticipantSelected = (userId: number): boolean => {
  return selectedParticipants.value.some(p => p.user_id === userId)
}

const removeParticipant = (userId: number) => {
  selectedParticipants.value = selectedParticipants.value.filter(p => p.user_id !== userId)
}

const createThread = async () => {
  if (!canCreate.value) return

  isCreating.value = true
  try {
    const request = {
      thread_type: chatType.value,
      thread_name: chatType.value === 'group' ? threadName.value.trim() : undefined,
      participant_ids: selectedParticipants.value.map(p => p.user_id)
    }

    const response = await createThreadApi(request)
    emit('thread-created', response.thread)
  } catch (error) {
    console.error('Failed to create thread:', error)
  } finally {
    isCreating.value = false
  }
}

// Watch for chat type changes to clear participants
const handleChatTypeChange = () => {
  selectedParticipants.value = []
  threadName.value = ''
}

// Add watcher
const unwatch = () => {
  // This would be set up in onMounted/onUnmounted
}
</script>

<style scoped>
.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
