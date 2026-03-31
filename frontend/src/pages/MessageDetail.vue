<template>
   <!-- Loading State -->
  <div v-if="!message" class="bg-white border border-blue-200 rounded-lg p-6 flex justify-center">

    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />

  </div>
   <!-- Message Content -->
  <div v-else>
     <!-- Action Buttons --> <MessageActions
      ref="messageActions"
      class="mb-6"
      :message-id="message?.id"
      :spam-vote-count="message?.spam_vote_count"
      :cleaned-subject="message?.cleaned_subject"
      :show-spam-button="true"
      :current-user-voted-spam="currentUserVotedSpam"
      @toggle-spam-vote="toggleSpamVote"
    /> <!-- Message Header -->
    <div class="p-4 bg-white rounded-lg shadow-sm border border-gray-100">

      <div class="space-y-6">
         <!-- Subject -->
        <h2 class="text-2xl font-bold text-gray-800 mb-4 pb-4 border-b border-gray-100">
           <LazyMathJax
            :content="message.subject || ''"
            :enable-markdown="true"
            :search-term="props.searchTerm"
            curly-link-class="underline text-pink-600 hover:text-pink-800"
          />
        </h2>
         <!-- Message Meta -->
        <div class="flex flex-col md:flex-row gap-4 md:gap-6">
           <!-- Left Column -->
          <div class="space-y-4 md:space-y-6 md:flex-1 min-w-[280px]">
             <!-- From -->
            <div class="space-y-1">

              <div class="text-xs font-medium text-gray-500 uppercase tracking-wider">
                 {{ t('components.messageDetail.fromLabel') }}
              </div>

              <div class="text-gray-700 break-words">
                 {{ formatEmailAddress(message.from_address) }}
              </div>

            </div>
             <!-- To -->
            <div class="space-y-1">

              <div class="text-xs font-medium text-gray-500 uppercase tracking-wider">
                 {{ t('components.messageDetail.toLabel') }}
              </div>

              <div class="text-gray-700 break-words">
                 {{ formatEmailAddress(message.to_address) }}
              </div>

            </div>

          </div>
           <!-- Right Column -->
          <div class="space-y-4 md:space-y-6 md:flex-1 min-w-[280px]">
             <!-- Date -->
            <div class="space-y-1">

              <div class="text-xs font-medium text-gray-500 uppercase tracking-wider">
                 {{ t('components.messageDetail.dateLabel') }}
              </div>

              <div class="text-gray-700"> {{ formatDate(message.date) }} </div>

            </div>
             <!-- Message ID -->
            <div v-if="message.message_id" class="space-y-1">

              <div class="text-xs font-medium text-gray-500 uppercase tracking-wider">
                 {{ t('components.messageDetail.messageIdLabel') }}
              </div>

              <div class="text-gray-700 text-sm break-words"> {{ message.message_id }} </div>

            </div>

            <div v-if="message.message_id" class="space-y-1">

              <div class="text-xs font-medium text-gray-500 uppercase tracking-wider">
                 {{ t('components.messageDetail.filenameLabel') }}
              </div>

              <div class="text-gray-700 text-sm break-words"> {{ message.file_path }} </div>

            </div>

          </div>

        </div>

      </div>

    </div>
     <!-- Message Parts -->
    <div class="mt-6 p-4 bg-white rounded-lg shadow-sm border border-gray-100 space-y-6">
       <!-- Text Parts --> <template
        v-for="part in message.parts_json.filter((p) => p.mime_type.startsWith('text/'))"
        >
        <div
          v-if="part.mime_type === 'text/plain' || part.mime_type === 'text/html'"
          :key="part.id"
          class="prose max-w-none text-gray-700 message-content"
          v-html="highlightText(replaceCidReferences(part.content, part.mime_type))"
        />
         </template
      > <!-- Attachments -->
      <div
        v-if="message.parts_json.filter((p) => !p.mime_type.startsWith('text/')).length"
        class="pt-4 border-t border-gray-100"
      >

        <h3 class="text-lg font-semibold text-gray-800 mb-4">
           {{ t('components.messageDetail.attachmentsTitle') }}
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">

          <div
            v-for="part in message.parts_json.filter((p) => !p.mime_type.startsWith('text/'))"
            :key="part.id"
            class="p-3 bg-gray-50 rounded-lg border border-gray-200 hover:border-blue-200 transition-colors flex items-start gap-3"
          >

            <div class="flex-shrink-0">
               <AttachmentIcon :mime-type="part.mime_type" class="w-6 h-6 text-gray-500" />
            </div>

            <div class="flex-1 min-w-0">

              <div class="text-sm font-medium text-gray-700 truncate">
                 {{ part.filename || part.content_type }}
              </div>

              <div class="text-xs text-gray-500 mt-1"> {{ part.mime_type }} </div>

            </div>
             <button
              class="ui-btn--read"
              :title="t('components.messageDetail.downloadAttachmentTitle')"
              @click="downloadAttachment(part)"
            >
               <Download class="h-5 w-5" /> </button
            >
          </div>

        </div>

      </div>

    </div>
     <!-- Action Buttons --> <MessageActions
      ref="messageActions"
      class="mt-6 pb-6"
      :message-id="message?.id"
      :spam-vote-count="message?.spam_vote_count"
      :cleaned-subject="message?.cleaned_subject"
      :show-spam-button="true"
      :current-user-voted-spam="currentUserVotedSpam"
      @toggle-spam-vote="toggleSpamVote"
    />
  </div>

</template>

<script setup lang="ts">
import { Download } from 'lucide-vue-next'
import { marked } from 'marked'
import { ref, watch, computed } from 'vue'

import { getMessageDetails, voteSpamMessage } from '@/api'
import AttachmentIcon from '@/components/icons/AttachmentIcon.vue'
import MessageActions from '@/components/MessageActions.vue'
import { useSeoHead } from '@/composables/useSeoHead'
import LazyMathJax from '@/components/LazyMathJax.vue'
import { useI18n } from 'vue-i18n'
const { t, locale } = useI18n()

const props = defineProps({
  id: {
    type: String,
    required: true,
  },
  searchTerm: {
    type: String,
    default: '',
  },
})

const message = ref(null)
const currentUserVotedSpam = ref(false)

const fetchMessage = async () => {
  try {
    const response = await getMessageDetails(props.id)
    message.value = response.data
    currentUserVotedSpam.value = response.data.current_user_voted_spam || false
  } catch (error) {
    console.error('Error fetching message details:', error)
  }
}

const replaceCidReferences = (content, mimeType) => {
  // Handle CID replacements for HTML content
  if (mimeType === 'text/html') {
    return content.replace(/src=["']cid:([^'"]+)["']/gi, (match, cid) => {
      const part = message.value.parts_json.find((p) => p.content_id === cid)
      if (part) {
        return `src="data:${part.mime_type};base64,${part.content}"`
      }
      return match // Return original if not found
    })
  }
  return content
}

const highlightText = (text) => {
  if (!text) return ''
  const trimmedText = text.replace(/[\n\r ]+$/, '')

  // First parse with marked
  const parsedContent = marked.parse(trimmedText, {
    renderer: new marked.Renderer(),
    gfm: true,
    breaks: true,
    async: false,
  }) as string

  // Then apply search term highlighting if needed
  if (props.searchTerm) {
    const regex = new RegExp(`(${props.searchTerm})`, 'gi')
    return parsedContent.replace(regex, '<mark>$1</mark>')
  }

  return parsedContent
}

const formatDate = (dateStr) => {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  return new Intl.DateTimeFormat(undefined, {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: 'numeric',
    minute: 'numeric',
    timeZoneName: 'short',
  }).format(date)
}

const downloadAttachment = (part) => {
  let content = part.content

  if (part.is_base64) {
    // Properly decode base64 content to binary data
    content = Uint8Array.from(atob(content.replace(/[\r\n\s]/g, '')), (c) => c.charCodeAt(0))
  }

  const blob = new Blob([content], { type: part.mime_type })
  const url = window.URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = part.filename || `attachment.${part.mime_type.split('/').pop()}`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  window.URL.revokeObjectURL(url)
}

const formatEmailAddress = (email) => {
  if (!email) return ''
  const match = email.match(/(.*?)\s*<(.+?)>/)
  if (match) {
    const [, name, address] = match
    return `${name.trim()} <${address}>`
  }
  return email
}

const toggleSpamVote = async () => {
  if (!message.value) return
  try {
    const response = await voteSpamMessage(message.value.id)
    if (response.data.success) {
      message.value.spam_vote_count = response.data.spam_vote_count
      currentUserVotedSpam.value = response.data.user_voted
    }
  } catch (error) {
    console.error('Error toggling spam vote:', error)
  }
}

const messageActions = ref(null)

watch(() => props.id, fetchMessage, { immediate: true })

const pageTitle = computed(() => {
  if (!message.value) return t('components.messageDetail.loading')
  return t('components.messageDetail.title', { subject: message.value.subject })
})

useSeoHead({ title: pageTitle })
</script>

<style scoped>
mark {
  background-color: #fff9c4;
  padding: 0.1em 0;
  border-radius: 2px;
}

.message-content {
  line-height: 1.6;
  font-size: 0.9375rem;
  color: #374151;
  overflow-wrap: break-word;
  word-break: break-word;
}

.message-content :deep(p) {
  margin: 1em 0;
}

.message-content :deep(a) {
  color: #3b82f6;
  text-decoration: underline;
  font-weight: 500;
}

.message-content :deep(a):hover {
  color: #2563eb;
}

.message-content :deep(code) {
  background-color: #f3f4f6;
  padding: 0.2em 0.4em;
  border-radius: 0.25em;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.875em;
}

.message-content :deep(ul),
.message-content :deep(ol) {
  margin: 1em 0;
  padding-left: 1.5em;
}

.message-content :deep(li) {
  margin: 0.5em 0;
}

.message-content :deep(a) {
  color: #3b82f6;
  text-decoration: underline;
  font-weight: 500;
}

.message-content :deep(a):hover {
  color: #2563eb;
}

.message-content :deep(blockquote) {
  border-left: 3px solid #e5e7eb;
  padding-left: 1rem;
  margin: 1.5rem 0;
  color: #4b5563;
  font-style: italic;
}

.message-content :deep(pre) {
  white-space: pre-wrap;
  font-family: 'JetBrains Mono', monospace;
  margin: 1.5rem 0;
  padding: 1rem;
  background-color: #f9fafb;
  border-radius: 0.375rem;
  border: 1px solid #e5e7eb;
  color: #1f2937;
  font-size: 0.875rem;
  line-height: 1.5;
}

.message-content :deep(pre)::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.message-content :deep(pre)::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 4px;
}

.message-content :deep(pre)::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 4px;
}

.message-content :deep(pre)::-webkit-scrollbar-thumb:hover {
  background: #a1a1a1;
}
</style>

