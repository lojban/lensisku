<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50">
    <!-- Background overlay -->
    <div class="absolute inset-0 bg-black/50" @click="$emit('close')"></div>

    <!-- Modal panel -->
    <div
      class="card-base card-elevated relative w-full max-w-lg max-h-[90vh] flex flex-col overflow-hidden"
    >
      <div class="px-5 pt-5 pb-2 shrink-0">
        <h3 class="text-lg font-semibold text-gray-900">Start New Conversation</h3>
      </div>

      <div class="modal-scroll-body px-4 pt-2 pb-6">
        <!-- Error Message -->
        <div
          v-if="errorMessage"
          role="alert"
          class="mb-4 rounded-md bg-red-50 px-3 py-2 text-sm text-red-600"
        >
          {{ errorMessage }}
        </div>

        <!-- Chat Type Selection -->
        <div class="mb-4">
          <label class="filters-field-label">Chat Type</label>
          <div class="flex gap-4">
            <label class="flex items-center gap-2">
              <input v-model="chatType" type="radio" value="direct" class="checkbox-toggle" />
              <span class="text-sm text-gray-700">Direct Message</span>
            </label>
            <label class="flex items-center gap-2">
              <input v-model="chatType" type="radio" value="group" class="checkbox-toggle" />
              <span class="text-sm text-gray-700">Group Chat</span>
            </label>
          </div>
        </div>

        <!-- Thread Name (for group chats) -->
        <div v-if="chatType === 'group'" class="mb-4">
          <label for="thread-name" class="filters-field-label">Group Name</label>
          <input
            id="thread-name"
            v-model="threadName"
            type="text"
            placeholder="Enter group name"
            class="input-field w-full"
          />
        </div>

        <!-- Participants Search -->
        <div class="mb-4">
          <label for="participants" class="filters-field-label">Participants</label>
          <div class="relative">
            <Search
              class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400"
            />
            <input
              id="participants"
              v-model="searchQuery"
              type="text"
              placeholder="Search users..."
              class="input-field w-full pl-9"
              @input="handleSearch"
            />
          </div>
        </div>

        <!-- Selected Participants -->
        <div v-if="selectedParticipants.length > 0" class="mb-4 flex flex-wrap gap-2">
          <div
            v-for="participant in selectedParticipants"
            :key="participant.user_id"
            class="badge bg-blue-100 text-blue-800 inline-flex items-center gap-1"
          >
            {{ participant.username }}
            <button
              type="button"
              class="icon-btn-ghost icon-btn-ghost--compact"
              @click="removeParticipant(participant.user_id)"
            >
              <X class="h-3 w-3" />
            </button>
          </div>
        </div>

        <!-- User Search Results -->
        <div v-if="searchResults.length > 0" class="mb-4">
          <div
            role="list"
            class="max-h-48 overflow-y-auto rounded-lg border border-gray-200 p-2 space-y-1"
          >
            <button
              v-for="user in searchResults"
              :key="user.user_id"
              type="button"
              role="listitem"
              class="group assistant-session-row flex w-full items-center gap-3"
              :class="
                isParticipantSelected(user.user_id)
                  ? 'assistant-session-row--active'
                  : 'assistant-session-row--idle'
              "
              :aria-pressed="
                chatType === 'direct' ? isParticipantSelected(user.user_id) : undefined
              "
              :aria-label="user.username"
              @click="handleUserClick(user)"
            >
              <input
                v-if="chatType === 'group'"
                type="checkbox"
                :checked="isParticipantSelected(user.user_id)"
                class="checkbox-toggle shrink-0"
                @click.stop
                @change="toggleParticipant(user)"
              />
              <div class="avatar-placeholder-sm !h-8 !w-8 shrink-0 text-xs">
                {{ user.username[0]?.toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1 text-left">
                <p class="truncate text-sm font-medium text-gray-900">{{ user.username }}</p>
                <p v-if="user.realname" class="truncate text-xs text-gray-500">
                  {{ user.realname }}
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
          :disabled="!canCreate || isCreating"
          class="ui-btn--create"
          @click="createThread"
        >
          <span
            v-if="isCreating"
            class="inline-block h-4 w-4 animate-spin rounded-full border-b-2 border-current"
            aria-hidden="true"
          />
          {{ isCreating ? 'Creating...' : 'Create Chat' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Search, X } from 'lucide-vue-next'
import { useAuth } from '@/composables/useAuth'
import { createThread as createThreadApi } from '@/services/messaging/messagingApi'
import { listUsers } from '@/api'
import type { Thread } from '@/types/messaging'

interface UserSearchResult {
  user_id: number
  username: string
  realname?: string
}

const props = withDefaults(
  defineProps<{
    existingThreads?: Thread[]
  }>(),
  {
    existingThreads: () => [],
  }
)

const emit = defineEmits<{
  close: []
  'thread-created': [thread: Thread]
  'open-thread': [thread: Thread]
}>()

const auth = useAuth()

// Reactive state
const chatType = ref<'direct' | 'group'>('direct')
const threadName = ref('')
const searchQuery = ref('')
const searchResults = ref<UserSearchResult[]>([])
const selectedParticipants = ref<UserSearchResult[]>([])
const isSearching = ref(false)
const isCreating = ref(false)
const errorMessage = ref('')

let searchTimeout: ReturnType<typeof setTimeout> | null = null

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
  errorMessage.value = ''
  searchResults.value = []
  searchTimeout = setTimeout(async () => {
    try {
      const response = await listUsers({ search: query, per_page: 20 })
      const users = (response.data.users ?? []) as UserSearchResult[]
      searchResults.value = users.filter((user) => user.username !== auth.state.username)
    } catch (error) {
      errorMessage.value = 'Failed to search users. Please try again.'
      console.error('Failed to search users:', error)
      searchResults.value = []
    } finally {
      isSearching.value = false
    }
  }, 300)
}

const handleUserClick = (user: UserSearchResult) => {
  errorMessage.value = ''
  if (chatType.value === 'direct') {
    selectedParticipants.value = [user]
    createThread()
  } else {
    toggleParticipant(user)
  }
}

const toggleParticipant = (user: UserSearchResult) => {
  if (isParticipantSelected(user.user_id)) {
    removeParticipant(user.user_id)
  } else {
    selectedParticipants.value.push(user)
  }
}

const isParticipantSelected = (userId: number): boolean => {
  return selectedParticipants.value.some((p) => p.user_id === userId)
}

const removeParticipant = (userId: number) => {
  selectedParticipants.value = selectedParticipants.value.filter((p) => p.user_id !== userId)
}

const createThread = async () => {
  if (isCreating.value || !canCreate.value) return

  isCreating.value = true
  errorMessage.value = ''

  try {
    if (chatType.value === 'direct') {
      const user = selectedParticipants.value[0]
      const existingThread = props.existingThreads.find(
        (t) => t.thread_type === 'direct' && t.participants?.some((p) => p.user_id === user.user_id)
      )
      if (existingThread) {
        emit('open-thread', existingThread)
        return
      }
    }

    const request = {
      thread_type: chatType.value,
      thread_name: chatType.value === 'group' ? threadName.value.trim() : undefined,
      participant_ids: selectedParticipants.value.map((p) => p.user_id),
    }

    const response = await createThreadApi(request)
    emit('thread-created', response.thread)
  } catch (error) {
    errorMessage.value = 'Failed to create chat. Please try again.'
    console.error('Failed to create thread:', error)
  } finally {
    isCreating.value = false
  }
}

// Watch for chat type changes to clear participants
watch(chatType, () => {
  selectedParticipants.value = []
  threadName.value = ''
  errorMessage.value = ''
})
</script>
