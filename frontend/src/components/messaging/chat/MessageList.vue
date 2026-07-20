<template>
  <div class="message-list space-y-4">
    <!-- Date divider -->
    <div v-for="(group, date) in groupedMessages" :key="date" class="space-y-4">
      <div class="flex items-center justify-center">
        <span class="badge badge-muted">
          {{ formatDate(date) }}
        </span>
      </div>

      <!-- Messages for this date -->
      <div
        v-for="message in group"
        :key="message.message_id"
        class="flex"
        :class="{ 'justify-end': message.is_from_sender }"
      >
        <MessageBubble
          :message="message"
          :is-from-sender="message.is_from_sender"
          @click="$emit('message-click', message)"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import MessageBubble from './MessageBubble.vue'
import type { Message } from '@/types/messaging'

interface Props {
  messages: Message[]
  currentUserId: number
}

const props = defineProps<Props>()

defineEmits<{
  'message-click': [message: Message]
}>()

// Group messages by date
const groupedMessages = computed(() => {
  const groups: Record<string, Message[]> = {}

  props.messages.forEach((message) => {
    const date = new Date(message.created_at).toDateString()
    if (!groups[date]) {
      groups[date] = []
    }
    groups[date].push(message)
  })

  return groups
})

const formatDate = (dateString: string): string => {
  const date = new Date(dateString)
  const today = new Date()
  const yesterday = new Date(today)
  yesterday.setDate(yesterday.getDate() - 1)

  if (date.toDateString() === today.toDateString()) {
    return 'Today'
  } else if (date.toDateString() === yesterday.toDateString()) {
    return 'Yesterday'
  } else {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: date.getFullYear() !== today.getFullYear() ? 'numeric' : undefined,
    })
  }
}
</script>

<style scoped>
.message-list {
  scroll-behavior: smooth;
}
</style>
