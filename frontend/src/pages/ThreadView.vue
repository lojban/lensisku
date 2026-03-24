<template>
  <!-- Thread Header -->
  <div class="bg-white border border-blue-200 rounded-lg p-4 mb-4 shadow-sm">
    <h2 class="text-xl font-semibold text-gray-700 mb-4">
      {{ cleanedSubject }}
    </h2>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <!-- Sort Controls -->
      <div class="flex items-center space-x-3">
        <label class="text-sm text-gray-600 font-medium">{{ t('sort.sortByLabel') }}</label>
        <select v-model="sortOrder" class="input-field" @change="fetchThread">
          <option value="desc">
            {{ t('threadView.newestFirst') }}
          </option>
          <option value="asc">
            {{ t('threadView.oldestFirst') }}
          </option>
        </select>
      </div>

      <!-- Content Toggle -->
      <div class="flex items-center space-x-3">
        <input
          v-model="includeContent"
          type="checkbox"
          class="checkbox-toggle"
          @change="fetchThread"
        />
        <label class="text-sm text-gray-600 font-medium whitespace-nowrap cursor-pointer">{{
          t('threadView.showContent')
        }}</label>
      </div>
    </div>
  </div>

  <!-- Loading State -->
  <LoadingSpinner v-if="isLoading" class="py-12" />

  <!-- Messages List -->
  <div v-else-if="!isLoading && messages.length > 0" class="space-y-4">
    <div
      v-for="message in messages"
      :key="message.id"
      class="message-item bg-white border border-blue-200 rounded-lg hover:border-blue-300 transition-colors shadow-sm"
    >
      <div class="p-4">
        <!-- Message Header -->
        <div class="flex justify-between items-start mb-3">
          <h3 class="text-lg font-semibold text-blue-700">
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
            <span class="font-medium text-gray-700">{{ t('threadView.from') }}</span>
            <span>{{ formatEmailAddress(message.from_address) }}</span>
          </div>

          <div v-if="message.to_address" class="flex items-center space-x-2 text-sm text-gray-600">
            <span class="font-medium text-gray-700">{{ t('threadView.to') }}</span>
            <span>{{ formatEmailAddress(message.to_address) }}</span>
          </div>
        </div>

        <!-- Message Parts -->
        <div v-if="includeContent && message.parts_json" class="mt-4 pt-4 border-t border-gray-100">
          <!-- Text parts -->
          <div
            v-for="part in message.parts_json.filter((p) => p.mime_type.startsWith('text/'))"
            :key="part.id"
            class="text-gray-700 text-sm prose max-w-none"
            v-html="highlightText(part.content)"
          />

          <!-- Attachments -->
          <div
            v-if="message.parts_json.filter((p) => !p.mime_type.startsWith('text/')).length"
            class="mt-4"
          >
            <div class="text-sm font-medium text-gray-600 mb-2">
              {{ t('threadView.attachments') }}
            </div>
            <div class="flex flex-wrap gap-2">
              <div
                v-for="part in message.parts_json.filter((p) => !p.mime_type.startsWith('text/'))"
                :key="part.id"
                class="px-3 py-1.5 bg-gray-50 rounded-lg border border-gray-200 hover:border-blue-200 transition-colors flex items-center gap-2"
              >
                <AttachmentIcon :mime-type="part.mime_type" class="w-4 h-4 flex-shrink-0" />
                <span class="text-sm text-gray-700">{{ part.content_type }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Empty State -->
  <div
    v-else-if="!isLoading && messages.length === 0"
    class="text-center p-8 bg-white border border-blue-200 rounded-lg"
  >
    <p class="text-gray-600">
      {{ t('threadView.noMessages') }}
    </p>
  </div>

  <!-- PaginationComponent -->
  <PaginationComponent
    v-if="!isLoading && messages.length > 0 && totalPages > 1"
    :current-page="currentPage"
    :total-pages="totalPages"
    :total="total"
    :per-page="10"
    class="mt-4"
    @prev="() => changePage(currentPage - 1)"
    @next="() => changePage(currentPage + 1)"
  />
</template>

<script setup>
import { marked } from 'marked'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { getThread } from '@/api'
import AttachmentIcon from '@/components/icons/AttachmentIcon.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import { useSeoHead } from '@/composables/useSeoHead'

// Props
const props = defineProps({
  subject: {
    type: String,
    required: true,
  },
  searchTerm: {
    type: String,
    default: '',
  },
})

const router = useRouter()
const route = useRoute()
const { t, locale } = useI18n()
const messages = ref([])
const cleanedSubject = ref('')
const currentPage = ref(parseInt(route.query.page) || 1)
const totalPages = ref(1)
const total = ref(0)
const sortOrder = ref('desc')
const isLoading = ref(true)
const includeContent = ref(true)
const threadSubject = computed(() => props.subject.replace(/^(Re:\s*)+/, ''))

// Initialize page title
const pageTitle = computed(() => {
  if (!cleanedSubject.value) return t('threadView.loadingThread')
  return t('threadView.threadTitle', { subject: cleanedSubject.value })
})

useSeoHead({ title: pageTitle }, locale.value)

const fetchThread = async () => {
  isLoading.value = true
  try {
    const response = await getThread({
      subject: threadSubject.value,
      search: props.searchTerm,
      page: currentPage.value,
      per_page: 10,
      sort_by: 'sent_at',
      sort_order: sortOrder.value,
      include_content: includeContent.value,
    })
    messages.value = response.data.messages
    cleanedSubject.value = response.data.clean_subject
    total.value = response.data.total
    totalPages.value = Math.ceil(response.data.total / response.data.per_page)
  } catch (error) {
    console.error('Error fetching thread:', error)
  } finally {
    isLoading.value = false
  }
}

const changePage = async (page) => {
  currentPage.value = page
  await router.push({
    query: {
      ...route.query,
      page: page > 1 ? page : undefined,
    },
  })
  await fetchThread()
}

// Watch for page changes in URL
watch(
  () => route.query.page,
  (newPage) => {
    const page = parseInt(newPage) || 1
    if (page !== currentPage.value) {
      currentPage.value = page
      fetchThread()
    }
  }
)

// Watch for search term changes
watch(
  () => props.searchTerm,
  () => {
    currentPage.value = 1
    fetchThread()
  }
)

const highlightText = (text) => {
  if (!text) return ''
  const trimmedText = text.replace(/[\n\r ]+$/, '')

  // First parse with marked
  const parsedContent = marked(trimmedText, {
    renderer: new marked.Renderer(),
    gfm: true,
    breaks: true,
  })

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
  const match = email.match(/(.*?)\s*<(.+?)>/)
  if (match) {
    const [, name, address] = match
    return name.trim() || address
  }
  return email
}

onMounted(() => {
  // Set initial page from URL
  currentPage.value = parseInt(route.query.page) || 1
  fetchThread()
})
</script>

<style scoped>
/* Hover effect */
.message-item {
  word-break: break-word;
}

.message-item:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

/* Control panel styles */
select,
input[type='checkbox'] {
  cursor: pointer;
}

select:focus {
  outline: none;
}

/* Prose settings for message content */
.prose {
  max-width: none;
  color: inherit;
}

.prose {
  @apply max-w-none;
}

.prose :deep(p) {
  @apply my-2;
}

.prose :deep(a) {
  @apply text-blue-600 hover:text-blue-800 hover:underline;
}

.prose :deep(code) {
  @apply bg-gray-100 px-1 py-0.5 rounded text-sm font-mono;
}

.prose :deep(ul),
.prose :deep(ol) {
  @apply my-2 pl-6;
}

.prose :deep(li) {
  @apply my-1;
}

.prose :deep(blockquote) {
  @apply border-l-4 border-gray-300 pl-4 my-2 text-gray-600 italic;
}

.prose :deep(pre) {
  @apply bg-gray-100 p-2 rounded my-2 overflow-x-auto;
}

.prose :deep(h1),
.prose :deep(h2),
.prose :deep(h3),
.prose :deep(h4),
.prose :deep(h5),
.prose :deep(h6) {
  @apply font-bold mt-4 mb-2;
}

.prose :deep(h1) {
  @apply text-2xl;
}

.prose :deep(h2) {
  @apply text-xl;
}

.prose :deep(h3) {
  @apply text-lg;
}

.prose :deep(h4) {
  @apply text-base;
}

.prose :deep(h5) {
  @apply text-sm;
}

.prose :deep(h6) {
  @apply text-xs;
}
</style>
