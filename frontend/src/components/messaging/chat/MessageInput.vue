<template>
  <div class="message-input">
    <form @submit.prevent="handleSubmit" class="flex items-end space-x-2">
      <!-- Attachment button -->
      <button
        type="button"
        @click="handleAttachment"
        class="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-full transition-colors"
        title="Attach file"
      >
        <Paperclip class="h-5 w-5" />
      </button>

      <!-- Message input -->
      <div class="flex-1 relative">
        <textarea
          v-model="messageText"
          :placeholder="placeholder"
          :disabled="disabled"
          rows="1"
          class="w-full px-4 py-2 pr-12 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none disabled:opacity-50 disabled:cursor-not-allowed"
          @input="handleInput"
          @keydown="handleKeydown"
          @focus="handleFocus"
          @blur="handleBlur"
          ref="textareaRef"
        ></textarea>
        
        <!-- Character count for long messages -->
        <div
          v-if="messageText.length > 100"
          class="absolute bottom-1 right-1 text-xs text-gray-400"
        >
          {{ messageText.length }}/2000
        </div>
      </div>

      <!-- Send button -->
      <button
        type="submit"
        :disabled="!canSend || isSending"
        class="p-2 text-white bg-blue-600 hover:bg-blue-700 rounded-full transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        title="Send message"
      >
        <div v-if="isSending" class="animate-spin rounded-full h-5 w-5 border-b-2 border-white"></div>
        <Send v-else class="h-5 w-5" />
      </button>
    </form>

    <!-- File input (hidden) -->
    <input
      ref="fileInputRef"
      type="file"
      class="hidden"
      @change="handleFileSelect"
      accept="image/*,application/pdf,.doc,.docx,.txt"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue'
import { Paperclip, Send } from 'lucide-vue-next'
import { useAuth } from '@/composables/useAuth'
import { sendMessage } from '@/services/messaging/messagingApi'
import type { Message, SendMessageRequest } from '@/types/messaging'

interface Props {
  threadId: number
  disabled?: boolean
  placeholder?: string
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  placeholder: 'Type a message...'
})

const emit = defineEmits<{
  'message-sent': [message: Message]
  'typing-start': []
  'typing-stop': []
}>()

const auth = useAuth()

// Reactive state
const messageText = ref('')
const isSending = ref(false)
const textareaRef = ref<HTMLTextAreaElement>()
const fileInputRef = ref<HTMLInputElement>()
const typingTimeout = ref<number>()

// Computed properties
const canSend = computed(() => {
  return messageText.value.trim().length > 0 && !props.disabled && !isSending.value
})

// Methods
const handleSubmit = async () => {
  if (!canSend.value) return

  const content = messageText.value.trim()
  if (!content) return

  isSending.value = true
  
  try {
    // This is a simplified implementation
    // In a real app, you would encrypt the message content here
    const request: SendMessageRequest = {
      thread_id: props.threadId,
      message_type: 'text',
      encrypted_content: btoa(content), // Simple base64 encoding as placeholder
      content_nonce: 'placeholder_nonce', // Would be generated in real implementation
    }

    const response = await sendMessage(request)
    
    // Clear input
    messageText.value = ''
    resizeTextarea()
    
    // Emit the sent message
    emit('message-sent', response.message)
    
  } catch (error) {
    console.error('Failed to send message:', error)
    // Show error to user (could add toast notification)
  } finally {
    isSending.value = false
  }
}

const handleInput = () => {
  resizeTextarea()
  
  // Emit typing events
  emit('typing-start')
  
  // Clear existing timeout
  if (typingTimeout.value) {
    clearTimeout(typingTimeout.value)
  }
  
  // Set new timeout to stop typing after 1 second of inactivity
  typingTimeout.value = setTimeout(() => {
    emit('typing-stop')
  }, 1000)
}

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    handleSubmit()
  }
}

const handleFocus = () => {
  resizeTextarea()
}

const handleBlur = () => {
  emit('typing-stop')
}

const handleAttachment = () => {
  fileInputRef.value?.click()
}

const handleFileSelect = (event: Event) => {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (file) {
    // Handle file attachment
    console.log('File selected:', file)
    // In real implementation, you would upload the file and send as message
  }
}

const resizeTextarea = () => {
  if (!textareaRef.value) return
  
  const textarea = textareaRef.value
  textarea.style.height = 'auto'
  
  // Calculate new height (max 5 rows)
  const maxHeight = parseInt(getComputedStyle(textarea).lineHeight) * 5
  const newHeight = Math.min(textarea.scrollHeight, maxHeight)
  
  textarea.style.height = `${newHeight}px`
}

const stopTyping = () => {
  if (typingTimeout.value) {
    clearTimeout(typingTimeout.value)
  }
  emit('typing-stop')
}

// Lifecycle
onMounted(() => {
  // Auto-resize textarea on mount
  nextTick(() => {
    resizeTextarea()
  })
})

onUnmounted(() => {
  stopTyping()
})

// Expose methods for parent component
defineExpose({
  focus: () => textareaRef.value?.focus(),
  blur: () => textareaRef.value?.blur(),
  clear: () => {
    messageText.value = ''
    resizeTextarea()
  }
})
</script>

<style scoped>
.message-input {
  @apply bg-white rounded-lg border border-gray-200 p-2;
}

/* Custom scrollbar for textarea */
textarea {
  scrollbar-width: thin;
  scrollbar-color: #e5e7eb #f9fafb;
}

textarea::-webkit-scrollbar {
  width: 4px;
}

textarea::-webkit-scrollbar-track {
  background: #f9fafb;
}

textarea::-webkit-scrollbar-thumb {
  background-color: #e5e7eb;
  border-radius: 2px;
}

textarea::-webkit-scrollbar-thumb:hover {
  background-color: #d1d5db;
}

/* Animation for send button */
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

/* Focus styles */
textarea:focus {
  @apply ring-2 ring-blue-500 ring-offset-0;
}

/* Mobile optimizations */
@media (max-width: 640px) {
  .message-input {
    @apply rounded-none border-l-0 border-r-0 border-b-0;
  }
}
</style>
