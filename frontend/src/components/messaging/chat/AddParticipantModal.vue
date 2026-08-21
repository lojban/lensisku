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
          <Input
            v-model="searchQuery"
            type="text"
            search-icon
            placeholder="Search users..."
            class="input-field w-full"
            @input="handleSearch"
          />
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
              <IconButtonGhost class="--compact" @click="removeUser(user.user_id)">
                <X class="h-3 w-3" />
              </IconButtonGhost>
            </div>
          </div>
        </div>

        <!-- Search Results -->
        <div v-if="searchResults.length > 0" class="mb-4">
          <div
            class="max-h-64 overflow-y-auto rounded-lg border border-gray-200 divide-y divide-gray-100"
          >
            <Button
              v-for="user in searchResults"
              :key="user.user_id"
              variant="neutral"
              type="button"
              class="surface-list-row flex items-center gap-3 !p-3"
              @click="toggleUser(user)"
            >
              <Checkbox
                :checked="isUserSelected(user.user_id)"
                class="checkbox-toggle"
                @click.stop
              />
              <div class="avatar-placeholder-sm !h-8 !w-8 text-xs">
                {{ user.username[0]?.toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1 text-left">
                <p class="text-sm font-medium text-gray-900">{{ user.username }}</p>
                <p v-if="user.realname" class="text-xs text-gray-500">{{ user.realname }}</p>
              </div>
            </Button>
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
        <Button variant="cancel" type="button" @click="$emit('close')">Cancel</Button>
        <Button
          variant="create"
          type="button"
          :disabled="selectedUsers.length === 0 || isAdding"
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
        </Button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button, Checkbox, IconButtonGhost, Input } from '@packages/ui'
import { ref } from 'vue'
import { X } from '@lucide/vue'
import { useAuth } from '@/composables/useAuth'
import { addParticipant } from '@/services/messaging/messagingApi'
import { listUsers } from '@/api'

interface Props {
  threadId: number
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  'participant-added': [userId: number]
}>()

const auth = useAuth()

interface UserSearchResult {
  user_id: number
  username: string
  realname?: string
}

// Reactive state
const searchQuery = ref('')
const searchResults = ref<UserSearchResult[]>([])
const selectedUsers = ref<UserSearchResult[]>([])
const isSearching = ref(false)
const isAdding = ref(false)

let searchTimeout: ReturnType<typeof setTimeout> | null = null

// Methods
const handleSearch = async () => {
  const query = searchQuery.value.trim()
  if (!query) {
    searchResults.value = []
    isSearching.value = false
    return
  }

  if (searchTimeout) {
    clearTimeout(searchTimeout)
  }

  isSearching.value = true
  searchResults.value = []
  searchTimeout = setTimeout(async () => {
    try {
      const response = await listUsers({ search: query, per_page: 20 })
      const users = (response.data.users ?? []) as UserSearchResult[]
      searchResults.value = users.filter((user) => user.username !== auth.state.username)
    } catch (error) {
      console.error('Failed to search users:', error)
      searchResults.value = []
    } finally {
      isSearching.value = false
    }
  }, 300)
}

const toggleUser = (user: UserSearchResult) => {
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
