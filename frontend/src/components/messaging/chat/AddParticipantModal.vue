<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50">
    <!-- Background overlay -->
    <div class="absolute inset-0 bg-black/50" @click="$emit('close')"></div>

    <!-- Modal panel -->
    <div
      class="card-base card-elevated relative w-full max-w-lg max-h-[90vh] flex flex-col overflow-hidden"
    >
      <div class="px-5 pt-5 pb-2 shrink-0">
        <h3 class="text-lg font-semibold text-gray-900">Add Participants</h3>
      </div>

      <div class="modal-scroll-body px-4 pt-2 pb-6">
        <!-- Search -->
        <div class="mb-4">
          <div class="relative">
            <Search
              class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400"
            />
            <input
              v-model="searchQuery"
              type="text"
              placeholder="Search users..."
              class="input-field w-full pl-9"
              @input="handleSearch"
            />
          </div>
        </div>

        <!-- Selected Participants -->
        <div v-if="selectedUsers.length > 0" class="mb-4">
          <p class="filters-field-label">Selected:</p>
          <div class="flex flex-wrap gap-2">
            <div
              v-for="user in selectedUsers"
              :key="user.user_id"
              class="badge bg-blue-100 text-blue-800 inline-flex items-center gap-1"
            >
              {{ user.username }}
              <button
                type="button"
                class="icon-btn-ghost icon-btn-ghost--compact"
                @click="removeUser(user.user_id)"
              >
                <X class="h-3 w-3" />
              </button>
            </div>
          </div>
        </div>

        <!-- Search Results -->
        <div v-if="searchResults.length > 0" class="mb-4">
          <div
            class="max-h-64 overflow-y-auto rounded-lg border border-gray-200 divide-y divide-gray-100"
          >
            <button
              v-for="user in searchResults"
              :key="user.user_id"
              type="button"
              class="surface-list-row flex items-center gap-3 !p-3"
              @click="toggleUser(user)"
            >
              <input
                type="checkbox"
                :checked="isUserSelected(user.user_id)"
                class="checkbox-toggle"
                @click.stop
              />
              <div class="avatar-placeholder-sm !h-8 !w-8 text-xs">
                {{ user.username[0]?.toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1 text-left">
                <p class="text-sm font-medium text-gray-900">{{ user.username }}</p>
                <p class="text-xs text-gray-500">
                  {{ user.is_online ? 'Online' : 'Offline' }}
                </p>
              </div>
            </button>
          </div>
        </div>

        <!-- No Results -->
        <div
          v-if="searchQuery && searchResults.length === 0 && !isSearching"
          class="text-center py-4 text-gray-500"
        >
          No users found
        </div>
      </div>

      <!-- Modal Actions -->
      <div class="px-5 pb-5 pt-3 border-t border-gray-100 flex justify-end gap-3 shrink-0">
        <button type="button" class="ui-btn--cancel" @click="$emit('close')">Cancel</button>
        <button
          type="button"
          :disabled="selectedUsers.length === 0 || isAdding"
          class="ui-btn--create"
          @click="addParticipants"
        >
          <span
            v-if="isAdding"
            class="inline-block h-4 w-4 animate-spin rounded-full border-b-2 border-current"
            aria-hidden="true"
          />
          {{
            isAdding
              ? 'Adding...'
              : `Add ${selectedUsers.length} Participant${selectedUsers.length !== 1 ? 's' : ''}`
          }}
        </button>
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
const searchResults = ref<Array<{ user_id: number; username: string; is_online: boolean }>>([])
const selectedUsers = ref<Array<{ user_id: number; username: string; is_online: boolean }>>([])
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
    await new Promise((resolve) => setTimeout(resolve, 300))

    // Mock data - replace with actual API call
    searchResults.value = [
      { user_id: 5, username: 'david', is_online: true },
      { user_id: 6, username: 'eve', is_online: false },
      { user_id: 7, username: 'frank', is_online: true },
    ].filter(
      (user) =>
        user.username.toLowerCase().includes(searchQuery.value.toLowerCase()) &&
        user.user_id !== (auth.state.username as unknown)
    )
  } catch (error) {
    console.error('Failed to search users:', error)
    searchResults.value = []
  } finally {
    isSearching.value = false
  }
}

const toggleUser = (user: { user_id: number; username: string; is_online: boolean }) => {
  if (isUserSelected(user.user_id)) {
    removeUser(user.user_id)
  } else {
    selectedUsers.value.push(user)
  }
}

const isUserSelected = (userId: number): boolean => {
  return selectedUsers.value.some((u) => u.user_id === userId)
}

const removeUser = (userId: number) => {
  selectedUsers.value = selectedUsers.value.filter((u) => u.user_id !== userId)
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
