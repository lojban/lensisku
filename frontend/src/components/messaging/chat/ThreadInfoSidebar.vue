<template>
  <div class="fixed inset-0 z-50 overflow-hidden">
    <!-- Background overlay -->
    <div class="absolute inset-0 bg-black/50" @click="$emit('close')"></div>

    <!-- Sidebar -->
    <div
      class="card-base absolute right-0 top-0 h-full w-full max-w-md shadow-xl rounded-r-none rounded-l-2xl"
    >
      <div class="h-full flex flex-col">
        <!-- Header -->
        <div class="flex items-center justify-between p-4 border-b border-gray-200">
          <h2 class="text-lg font-semibold text-gray-900">Thread Info</h2>
          <button type="button" class="icon-btn-ghost" @click="$emit('close')">
            <X class="h-5 w-5" />
          </button>
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-y-auto">
          <!-- Thread details -->
          <div class="p-4 border-b border-gray-200">
            <div class="flex items-center gap-3 mb-4">
              <div class="avatar-placeholder-sm">
                {{ getThreadInitials() }}
              </div>
              <div>
                <h3 class="text-lg font-medium text-gray-900">
                  {{ getThreadDisplayName() }}
                </h3>
                <p class="text-sm text-gray-500">
                  {{ getThreadTypeText() }}
                </p>
              </div>
            </div>

            <div class="space-y-2 text-sm text-gray-600">
              <div class="flex justify-between">
                <span>Created:</span>
                <span>{{ formatDate(thread.created_at) }}</span>
              </div>
              <div class="flex justify-between">
                <span>Messages:</span>
                <span>{{ thread.message_count }}</span>
              </div>
              <div v-if="thread.thread_type === 'group'" class="flex justify-between">
                <span>Members:</span>
                <span>{{ thread.participant_count }}</span>
              </div>
            </div>
          </div>

          <!-- Participants -->
          <div class="p-4">
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-lg font-medium text-gray-900">Participants</h3>
              <button
                v-if="thread.thread_type === 'group' && thread.is_admin"
                type="button"
                class="ui-btn--create"
                @click="showAddParticipant = true"
              >
                Add
              </button>
            </div>

            <div class="space-y-2">
              <div
                v-for="participant in participants"
                :key="participant.user_id"
                class="flex items-center justify-between p-2 rounded-lg hover:bg-gray-50"
              >
                <div class="flex items-center gap-3">
                  <div class="relative">
                    <div class="avatar-placeholder-sm !h-8 !w-8 text-xs">
                      {{ participant.username[0]?.toUpperCase() }}
                    </div>
                    <div
                      v-if="participant.is_online"
                      class="absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full border-2 border-white bg-green-400"
                    />
                  </div>
                  <div>
                    <p class="text-sm font-medium text-gray-900">
                      {{ participant.username }}
                    </p>
                    <p class="text-xs text-gray-500">
                      {{ getRoleText(participant.role) }}
                    </p>
                  </div>
                </div>

                <!-- Actions -->
                <div class="flex items-center gap-2">
                  <button
                    v-if="
                      thread.thread_type === 'group' &&
                      thread.is_admin &&
                      participant.user_id !== currentUserId
                    "
                    type="button"
                    class="icon-btn-ghost"
                    title="Change role"
                    @click="updateParticipantRole(participant)"
                  >
                    <Shield class="h-4 w-4" />
                  </button>
                  <button
                    v-if="canRemoveParticipant(participant)"
                    type="button"
                    class="icon-btn-ghost-danger"
                    title="Remove from thread"
                    @click="removeParticipant(participant)"
                  >
                    <UserMinus class="h-4 w-4" />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Actions -->
          <div class="p-4 border-t border-gray-200 space-y-2">
            <button
              v-if="thread.thread_type === 'group' && !thread.is_admin"
              type="button"
              class="ui-btn--neutral-muted w-full justify-start"
              @click="leaveThread"
            >
              Leave Group
            </button>
            <button type="button" class="ui-btn--delete w-full justify-start" @click="deleteThread">
              Delete Conversation
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Participant Modal -->
    <AddParticipantModal
      v-if="showAddParticipant"
      :thread-id="thread.thread_id"
      @close="showAddParticipant = false"
      @participant-added="handleParticipantAdded"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { X, Shield, UserMinus } from 'lucide-vue-next'
import { useAuth } from '@/composables/useAuth'
import { usePresence } from '@/services/messaging/PresenceService'
import AddParticipantModal from './AddParticipantModal.vue'
import type { Thread, ThreadParticipant } from '@/types/messaging'

interface Props {
  thread: Thread
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  'participant-add': [userId: number]
  'participant-remove': [userId: number]
}>()

const auth = useAuth()
const { isUserOnline: _isUserOnline } = usePresence()

// Reactive state
const showAddParticipant = ref(false)

// Computed properties
const currentUserId = computed(() => auth.state.username as unknown)
const participants = computed(() => {
  return props.thread.participants || []
})

// Methods
const getThreadDisplayName = (): string => {
  if (props.thread.thread_name) {
    return props.thread.thread_name
  }

  if (props.thread.thread_type === 'direct') {
    const otherParticipant = participants.value.find((p) => p.user_id !== currentUserId.value)
    return otherParticipant?.username || 'Unknown User'
  }

  return 'Group Chat'
}

const getThreadInitials = (): string => {
  const name = getThreadDisplayName()

  if (props.thread.thread_type === 'group') {
    return name
      .split(' ')
      .map((word) => word[0])
      .join('')
      .toUpperCase()
      .slice(0, 2)
  }

  return name[0]?.toUpperCase() || '?'
}

const getThreadTypeText = (): string => {
  if (props.thread.thread_type === 'direct') {
    return 'Direct Message'
  }
  return 'Group Chat'
}

const getRoleText = (role: string): string => {
  return role === 'admin' ? 'Admin' : 'Member'
}

const formatDate = (timestamp: string): string => {
  return new Date(timestamp).toLocaleDateString()
}

const canRemoveParticipant = (participant: ThreadParticipant): boolean => {
  // Can remove if:
  // - You are the thread creator/admin
  // - You are removing yourself
  // - It's a direct message (both can leave)
  return (
    props.thread.is_admin ||
    participant.user_id === currentUserId.value ||
    props.thread.thread_type === 'direct'
  )
}

const updateParticipantRole = (participant: ThreadParticipant) => {
  const newRole = participant.role === 'admin' ? 'member' : 'admin'
  // Implementation would call API to update role
  console.log('Update role:', participant.user_id, newRole)
}

const removeParticipant = (participant: ThreadParticipant) => {
  if (confirm(`Remove ${participant.username} from the conversation?`)) {
    emit('participant-remove', participant.user_id)
  }
}

const handleParticipantAdded = (userId: number) => {
  showAddParticipant.value = false
  emit('participant-add', userId)
}

const leaveThread = () => {
  if (confirm('Are you sure you want to leave this group?')) {
    emit('participant-remove', currentUserId.value as number)
  }
}

const deleteThread = () => {
  if (confirm('Are you sure you want to delete this conversation? This cannot be undone.')) {
    // Implementation would call delete thread API
    console.log('Delete thread:', props.thread.thread_id)
  }
}
</script>

<style scoped>
/* Custom scrollbar */
.overflow-y-auto {
  scrollbar-width: thin;
  scrollbar-color: #e5e7eb #f9fafb;
}

.overflow-y-auto::-webkit-scrollbar {
  width: 6px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: #f9fafb;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background-color: #e5e7eb;
  border-radius: 3px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background-color: #d1d5db;
}
</style>
