<template>
  <div
    class="message-bubble max-w-xs lg:max-w-md"
    :class="{ 'message-sent': isFromSender, 'message-received': !isFromSender }"
    @click="$emit('click', message)"
  >
    <div class="relative px-4 py-2 rounded-lg shadow-sm" :class="bubbleClasses">
      <!-- Reply indicator -->
      <div
        v-if="message.reply_to_message_id"
        class="flex items-center space-x-2 mb-1 text-xs opacity-70"
      >
        <Reply class="h-3 w-3" />
        <span>Replying to a message</span>
      </div>

      <!-- Message content -->
      <div class="text-sm whitespace-pre-wrap break-words" :class="textClasses">
        <!-- For now, show encrypted content as placeholder -->
        <!-- In real implementation, this would be decrypted -->
        <div class="flex items-center space-x-2">
          <Lock class="h-4 w-4 opacity-50" />
          <span>{{ getMessagePreview() }}</span>
        </div>
      </div>

      <!-- Message metadata -->
      <div class="flex items-center justify-between mt-1 text-xs" :class="metadataClasses">
        <span>{{ formatTime(message.created_at) }}</span>
        <div class="flex items-center space-x-1">
          <!-- Edit indicator -->
          <span v-if="message.edit_count > 0" class="italic">(edited)</span>

          <!-- Delivery status for sent messages -->
          <div v-if="isFromSender" class="flex items-center space-x-1">
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
              class="h-3 w-3 text-blue-500"
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
const bubbleClasses = computed(() => {
  const base = 'transition-all duration-200 hover:shadow-md'

  if (props.message.is_from_sender) {
    return `${base} bg-blue-600 text-white`
  } else {
    return `${base} bg-white text-gray-900 border border-gray-200`
  }
})

const textClasses = computed(() => {
  return props.message.is_from_sender ? 'text-blue-50' : 'text-gray-900'
})

const metadataClasses = computed(() => {
  return props.message.is_from_sender ? 'text-blue-100' : 'text-gray-500'
})

const statusIconClasses = computed(() => {
  return props.message.is_from_sender ? 'text-blue-100' : 'text-gray-400'
})

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
  @apply transition-all duration-200;
}

.message-sent {
  @apply ml-auto;
}

.message-received {
  @apply mr-auto;
}

/* Hover effects */
.message-bubble:hover .relative {
  @apply transform scale-105;
}

/* Message status animations */
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

/* Typing indicator for sending messages */
.message-sent .relative {
  @apply transition-all duration-200;
}

/* Focus styles for accessibility */
.message-bubble:focus {
  @apply outline-none ring-2 ring-blue-500 ring-offset-2;
}
</style>
