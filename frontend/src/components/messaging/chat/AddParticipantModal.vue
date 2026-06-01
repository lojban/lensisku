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
                Add Participants
              </h3>

              <!-- Search -->
              <div class="mb-4">
                <div class="relative">
                  <Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                  <input
                    v-model="searchQuery"
                    type="text"
                    placeholder="Search users..."
                    class="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    @input="handleSearch"
                  >
                </div>
              </div>

              <!-- Selected Participants -->
              <div v-if="selectedUsers.length > 0" class="mb-4">
                <p class="text-sm font-medium text-gray-700 mb-2">Selected:</p>
                <div class="flex flex-wrap gap-2">
                  <div
                    v-for="user in selectedUsers"
                    :key="user.user_id"
                    class="inline-flex items-center px-3 py-1 rounded-full text-sm bg-blue-100 text-blue-800"
                  >
                    {{ user.username }}
                    <button
                      @click="removeUser(user.user_id)"
                      class="ml-2 text-blue-600 hover:text-blue-800"
                    >
                      <X class="h-3 w-3" />
                    </button>
                  </div>
                </div>
              </div>

              <!-- Search Results -->
              <div v-if="searchResults.length > 0" class="mb-4">
                <div class="max-h-64 overflow-y-auto border border-gray-200 rounded-md">
                  <div
                    v-for="user in searchResults"
                    :key="user.user_id"
                    @click="toggleUser(user)"
                    class="flex items-center p-3 hover:bg-gray-50 cursor-pointer border-b border-gray-100 last:border-b-0"
                  >
                    <input
                      type="checkbox"
                      :checked="isUserSelected(user.user_id)"
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
            @click="addParticipants"
            :disabled="selectedUsers.length === 0 || isAdding"
            class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-blue-600 text-base font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <div v-if="isAdding" class="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
            {{ isAdding ? 'Adding...' : `Add ${selectedUsers.length} Participant${selectedUsers.length !== 1 ? 's' : ''}` }}
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
import { ref } from 'vue'
import { Search, X } from 'lucide-vue-next'
import { useAuth } from '@/composables/useAuth'
import { addParticipant } from '@/services/messaging/messagingApi'

interface Props {
  threadId: number
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  'participant-added': [userId: number]
}>()

const auth = useAuth()

// Reactive state
const searchQuery = ref('')
const searchResults = ref<Array<{user_id: number, username: string, is_online: boolean}>>([])
const selectedUsers = ref<Array<{user_id: number, username: string, is_online: boolean}>>([])
const isSearching = ref(false)
const isAdding = ref(false)

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
      { user_id: 5, username: 'david', is_online: true },
      { user_id: 6, username: 'eve', is_online: false },
      { user_id: 7, username: 'frank', is_online: true },
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

const toggleUser = (user: {user_id: number, username: string, is_online: boolean}) => {
  if (isUserSelected(user.user_id)) {
    removeUser(user.user_id)
  } else {
    selectedUsers.value.push(user)
  }
}

const isUserSelected = (userId: number): boolean => {
  return selectedUsers.value.some(u => u.user_id === userId)
}

const removeUser = (userId: number) => {
  selectedUsers.value = selectedUsers.value.filter(u => u.user_id !== userId)
}

const addParticipants = async () => {
  if (selectedUsers.value.length === 0) return

  isAdding.value = true
  try {
    // Add each participant
    for (const user of selectedUsers.value) {
      await addParticipant(props.threadId, { user_id: user.user_id })
      emit('participant-added', user.user_id)
    }
    
    // Clear selection and close modal
    selectedUsers.value = []
    emit('close')
  } catch (error) {
    console.error('Failed to add participants:', error)
  } finally {
    isAdding.value = false
  }
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
