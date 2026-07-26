<template>
  <div class="message-input surface-comment-form !m-0 rounded-xl p-2 shadow-sm">
    <form class="flex items-end gap-2" @submit.prevent="handleSubmit">
      <!-- Attachment button -->
      <IconButtonGhost title="Attach file" @click="handleAttachment">
        <Paperclip class="h-5 w-5" />
      </IconButtonGhost>

      <!-- Message input -->
      <div class="flex-1 relative">
        <Textarea
          ref="textareaRef"
          v-model="messageText"
          :placeholder="placeholder"
          :disabled="disabled"
          rows="3"
          class="textarea-field resize-none pr-12"
          @input="handleInput"
          @keydown="handleKeydown"
          @focus="handleFocus"
          @blur="handleBlur"
        />

        <!-- Character count for long messages -->
        <div
          v-if="messageText.length > 100"
          class="absolute bottom-1 right-1 text-xs text-gray-400"
        >
          {{ messageText.length }}/2000
        </div>
      </div>

      <!-- Send button -->
      <Button
        variant="primary"
        type="submit"
        :disabled="!canSend || isSending"
        title="Send message"
      >
        <span
          v-if="isSending"
          class="inline-block h-4 w-4 animate-spin rounded-full border-b-2 border-current"
          aria-hidden="true"
        />
        <Send v-else class="h-4 w-4" />
      </Button>
    </form>

    <!-- File input (hidden) -->
    <FileInput
      ref="fileInputRef"
      type="file"
      class="hidden"
      accept="image/*,application/pdf,.doc,.docx,.txt"
      @change="handleFileSelect"
    />
  </div>
</template>

<script setup lang="ts">
import { Button, FileInput, IconButtonGhost, Textarea } from '@packages/ui'
import { ref, computed, nextTick, onMounted, onUnmounted, type ComponentPublicInstance } from 'vue'
import { Paperclip, Send } from 'lucide-vue-next'
import { sendMessage } from '@/services/messaging/messagingApi'
import { utf8ToBase64, generateNonce } from '@/utils/crypto'
import type { Message, SendMessageRequest } from '@/types/messaging'

interface Props {
  threadId: number
  disabled?: boolean
  placeholder?: string
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  placeholder: 'Type a message...',
})

const emit = defineEmits<{
  'message-sent': [message: Message]
  'typing-start': []
  'typing-stop': []
}>()

type TextareaInstance = ComponentPublicInstance & { focus: () => void; blur: () => void }

// Reactive state
const messageText = ref('')
const isSending = ref(false)
const textareaRef = ref<TextareaInstance | null>(null)
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
    const request: SendMessageRequest = {
      thread_id: props.threadId,
      message_type: 'text',
      encrypted_content: utf8ToBase64(content),
      content_nonce: generateNonce(),
    }

    const response = await sendMessage(request)

    // Clear input
    messageText.value = ''
    resizeTextarea()

    // Emit the sent message
    emit('message-sent', response)
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

  const textarea = textareaRef.value?.$el as HTMLTextAreaElement | undefined
  if (!textarea) return

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
  },
})
</script>

<style scoped>
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
</style>
