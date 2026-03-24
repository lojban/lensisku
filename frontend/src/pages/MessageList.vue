<template>
  <div class="space-y-2">
    <div
      v-for="message in messages"
      :key="message.id"
      class="message-item bg-white border border-blue-200 rounded-lg hover:border-blue-300 transition-colors cursor-pointer shadow-sm"
      @click="handleClick(message)"
    >
      <div class="p-3">
        <!-- Message Header -->
        <div class="flex justify-between items-start">
          <h3
            class="text-lg font-semibold text-blue-700 hover:text-blue-800 hover:underline flex-grow"
          >
            <LazyMathJax
              :content="message.subject || ''"
              :enable-markdown="true"
              :search-term="props.searchTerm"
              curly-link-class="underline text-pink-600 hover:text-pink-800"
            />
          </h3>
          <span class="text-sm text-gray-500 whitespace-nowrap ml-4">
            {{ formatDate(message.date) }}
          </span>
        </div>

        <!-- Message Details -->
        <div class="space-y-2">
          <div class="flex items-center space-x-2 text-sm text-gray-600">
            <span class="font-medium text-gray-700">From:</span>
            <span>{{ formatEmailAddress(message.from_address) }}</span>
          </div>

          <div v-if="message.to_address" class="flex items-center space-x-2 text-sm text-gray-600">
            <span class="font-medium text-gray-700">To:</span>
            <span>{{ formatEmailAddress(message.to_address) }}</span>
          </div>
        </div>

        <!-- Message Parts -->
        <div v-if="showContent && message.parts_json" class="mt-1 pt-1 border-t border-gray-100">
          <!-- Show first text/plain part -->
          <div
            v-for="part in message.parts_json.filter((p) => p.mime_type === 'text/plain')"
            :key="part.id"
            class="text-gray-700 text-sm line-clamp-3"
            v-html="highlightTextPlain(part.content)"
          />

          <!-- Attachments -->
          <div
            v-if="message.parts_json.filter((p) => p.mime_type !== 'text/plain').length"
            class="mt-2 flex items-center gap-2 flex-wrap"
          >
            <div
              v-for="part in message.parts_json.filter((p) => p.mime_type !== 'text/plain')"
              :key="part.id"
              class="px-2 py-1 bg-gray-100 rounded text-xs flex items-center gap-1"
            >
              <AttachmentIcon :mime-type="part.mime_type" class="w-4 h-4" />
              <span>{{ part.content_type }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div
      v-if="messages.length === 0"
      class="text-center p-8 bg-white border border-blue-200 rounded-lg"
    >
      <p class="text-gray-600">
        {{ t('messageList.noMessages') }}
      </p>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import AttachmentIcon from '@/components/icons/AttachmentIcon.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import { useSeoHead } from '@/composables/useSeoHead'

const props = defineProps({
  messages: {
    type: Array,
    required: true,
  },
  showContent: {
    type: Boolean,
    default: true,
  },
  isGroupedByThread: {
    type: Boolean,
    default: false,
  },
  searchTerm: {
    type: String,
    default: '',
  },
})

const { t, locale } = useI18n()
const pageTitle = computed(() => {
  return props.searchTerm ? `Searching Messages: ${props.searchTerm}` : 'Message List'
})

useSeoHead({ title: pageTitle }, locale.value)

const emit = defineEmits(['view-message', 'view-thread-summary'])

const handleClick = (message) => {
  if (props.isGroupedByThread) {
    // When grouped, message.subject is the cleaned_subject from the backend
    emit('view-thread-summary', message.subject)
  } else {
    emit('view-message', message.id)
  }
}

const highlightTextPlain = (text) => {
  if (!text) return ''
  const trimmedText = text.replace(/[\n\r ]+$/, '')
  if (!props.searchTerm) return trimmedText.replace(/\n/g, '<br>')
  const regex = new RegExp(`(${props.searchTerm})`, 'gi')
  return trimmedText.replace(regex, '<mark>$1</mark>').replace(/\n/g, '<br>')
}
const formatDate = (dateStr) => {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  return date.toLocaleDateString(locale.value, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

const formatEmailAddress = (email) => {
  if (!email) return ''
  // Handle "Name <email@example.com>" format
  const match = email.match(/(.*?)\s*<(.+?)>/)
  if (match) {
    const [, name, address] = match
    return name.trim() || address
  }
  return email
}
</script>

<style scoped>
mark {
  background-color: #fff9c4;
  padding: 0.1em 0;
  border-radius: 2px;
}

.line-clamp-3 {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* Hover effect */
.message-item:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

/* Keep long email addresses from breaking layout */
.message-item {
  word-break: break-word;
}
</style>
