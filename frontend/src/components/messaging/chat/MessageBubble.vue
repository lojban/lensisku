<template>
  <div
    class="message-bubble cursor-pointer transition-all duration-200"
    :class="{ 'ml-auto': isFromSender, 'mr-auto': !isFromSender }"
    @click="$emit('click', message)"
  >
    <div class="relative rounded-lg shadow-sm" :class="bubbleClasses">
      <!-- Reply indicator -->
      <div
        v-if="message.reply_to_message_id"
        class="flex items-center gap-2 mb-1 text-xs opacity-70"
      >
        <Reply class="h-3 w-3" />
        <span>Replying to a message</span>
      </div>

      <!-- Message content -->
      <div class="whitespace-pre-wrap break-words">
        <div class="flex items-center gap-2">
          <Lock class="h-4 w-4 opacity-50" />
          <span>{{ getMessagePreview() }}</span>
        </div>
      </div>

      <!-- Message metadata -->
      <div class="flex items-center justify-between mt-1 text-xs" :class="metadataClasses">
        <span>{{ formatTime(message.created_at) }}</span>
        <div class="flex items-center gap-1">
          <!-- Edit indicator -->
          <span v-if="message.edit_count > 0" class="italic">(edited)</span>

          <!-- Delivery status for sent messages -->
          <div v-if="isFromSender" class="flex items-center gap-1">
            <Check
              v-if="message.delivery_status === 'sent'"
              class="h-3 w-3"
              :class="statusIconClasses"
            />
            <CheckCheck
              v-else-if="message.delivery_status === 'delivered'"
              class="h-3 w-3"
              :class="statusIconClasses"
            />
            <CheckCheck
              v-else-if="message.delivery_status === 'read'"
              class="h-3 w-3"
              :class="statusIconClasses"
            />
            <Clock v-else-if="message.delivery_status === 'sending'" class="h-3 w-3 animate-spin" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Reply, Lock, Check, CheckCheck, Clock } from 'lucide-vue-next'
import type { Message } from '@/types/messaging'

interface Props {
  message: Message
  isFromSender: boolean
}

const props = defineProps<Props>()

defineEmits<{
  click: [message: Message]
}>()

// Computed properties
const bubbleClasses = computed(() =>
  props.message.is_from_sender ? 'assistant-bubble-user' : 'assistant-bubble-assistant'
)

const metadataClasses = computed(() =>
  props.message.is_from_sender ? 'text-white/70' : 'text-gray-500'
)

const statusIconClasses = computed(() =>
  props.message.is_from_sender ? 'text-white/80' : 'text-gray-400'
)

// Methods
const getMessagePreview = (): string => {
  // This is a placeholder - in real implementation, you'd decrypt the content
  const _content = props.message.encrypted_content

  if (props.message.message_type === 'text') {
    return '🔒 Encrypted message'
  } else if (props.message.message_type === 'image') {
    return '🖼️ Image'
  } else if (props.message.message_type === 'file') {
    return '📎 File'
  } else {
    return '🔒 Encrypted content'
  }
}

const formatTime = (timestamp: string): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)

  if (diffMins < 1) {
    return 'now'
  } else if (diffMins < 60) {
    return `${diffMins}m`
  } else if (diffMins < 1440) {
    const diffHours = Math.floor(diffMins / 60)
    return `${diffHours}h`
  } else {
    return date.toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    })
  }
}
</script>

<style scoped>
.message-bubble {
  cursor: pointer;
  transition: transform 0.2s ease-in-out;
}

.message-bubble:hover {
  transform: scale(1.01);
}

.message-bubble:focus-visible {
  outline: none;
  box-shadow: 0 0 0 2px #3b82f6;
  outline-offset: 2px;
}
</style>
