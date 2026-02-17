<template>
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
    <!-- Header with definitions being discussed -->
    <div class="mb-8">
      <div v-if="linkDetails" class="space-y-6">
        <h1 class="text-3xl font-extrabold text-gray-900 tracking-tight mb-4">
          {{ t('definitionLinkDiscussion.title') }}
        </h1>
        
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 items-stretch">
          <!-- First Definition -->
          <div class="flex flex-col">
            <h2 class="text-lg font-semibold text-gray-700 mb-2 flex items-center">
              <span class="bg-blue-100 text-blue-800 text-xs font-medium mr-2 px-2.5 py-0.5 rounded-full">1</span>
              {{ linkDetails.def1_word }}
            </h2>
            <div class="flex-grow p-4 bg-white rounded-xl shadow-sm border border-gray-100 hover:shadow-md transition-shadow duration-200">
              <div class="prose prose-sm max-w-none text-gray-600" v-html="renderMarkdown(linkDetails.def1_content)"></div>
              <div class="mt-4">
                <RouterLink :to="`/valsi/${linkDetails.def1_word.replace(/ /g, '_')}`" 
                  class="text-blue-600 hover:text-blue-700 text-sm font-medium inline-flex items-center">
                  {{ t('definitionLinkDiscussion.viewEntry') }}
                  <ArrowUpRight class="ml-1 h-3 w-3" />
                </RouterLink>
              </div>
            </div>
          </div>

          <!-- Second Definition -->
          <div class="flex flex-col">
            <h2 class="text-lg font-semibold text-gray-700 mb-2 flex items-center">
              <span class="bg-purple-100 text-purple-800 text-xs font-medium mr-2 px-2.5 py-0.5 rounded-full">2</span>
              {{ linkDetails.def2_word }}
            </h2>
            <div class="flex-grow p-4 bg-white rounded-xl shadow-sm border border-gray-100 hover:shadow-md transition-shadow duration-200">
              <div class="prose prose-sm max-w-none text-gray-600" v-html="renderMarkdown(linkDetails.def2_content)"></div>
              <div class="mt-4">
                <RouterLink :to="`/valsi/${linkDetails.def2_word.replace(/ /g, '_')}`" 
                  class="text-blue-600 hover:text-blue-700 text-sm font-medium inline-flex items-center">
                  {{ t('definitionLinkDiscussion.viewEntry') }}
                  <ArrowUpRight class="ml-1 h-3 w-3" />
                </RouterLink>
              </div>
            </div>
          </div>
        </div>

        <!-- Connection Indicator -->
        <div class="flex justify-center -my-3 relative z-10 pointer-events-none">
          <div class="bg-white p-2 rounded-full border border-gray-200 shadow-sm">
            <LinkIcon class="h-5 w-5 text-gray-400" />
          </div>
        </div>
      </div>

      <!-- Loading State for Header -->
      <div v-else-if="isLoading" class="animate-pulse space-y-4">
        <div class="h-8 bg-gray-200 rounded w-1/3"></div>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div class="h-40 bg-gray-100 rounded-xl"></div>
          <div class="h-40 bg-gray-100 rounded-xl"></div>
        </div>
      </div>
    </div>

    <!-- Main Discussion Area -->
    <div class="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
      <!-- Discussion Header -->
      <div class="px-6 py-4 border-b border-gray-100 flex items-center justify-between bg-gray-50/50">
        <div class="flex items-center space-x-2">
          <MessageCircle class="h-5 w-5 text-blue-500" />
          <h3 class="font-bold text-gray-800">{{ t('definitionLinkDiscussion.discussion') }}</h3>
          <span v-if="!isLoadingComments" class="bg-gray-200 text-gray-700 text-xs font-bold px-2 py-0.5 rounded-full">
            {{ totalComments }}
          </span>
        </div>
        
        <div class="flex items-center space-x-3">
          <label class="inline-flex items-center cursor-pointer group">
            <input type="checkbox" class="sr-only peer" :checked="flatStyle" @change="toggleFlatStyle">
            <div class="w-9 h-5 bg-gray-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-blue-600 relative"></div>
            <span class="ml-2 text-sm text-gray-600 group-hover:text-gray-900 transition-colors">{{ t('commentList.threaded') }}</span>
          </label>
          
          <button v-if="auth.state.isLoggedIn" @click="handleNewTopLevelComment"
            class="inline-flex items-center px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-bold rounded-lg transition-all transform hover:scale-105 active:scale-95 shadow-sm">
            <Plus class="h-4 w-4 mr-1" />
            {{ t('commentList.newWave') }}
          </button>
        </div>
      </div>

      <!-- New top-level comment form -->
      <div v-if="showTopLevelForm" class="p-6 bg-blue-50/30 border-b border-gray-100">
        <CommentForm 
          :is-submitting="isSubmitting" 
          :initial-values="newComment"
          @submit="submitComment" 
          @cancel="cancelComment" 
        />
      </div>

      <!-- Comments list -->
      <div class="p-6 min-h-[400px]">
        <div v-if="isLoadingComments" class="flex justify-center items-center py-20">
          <Loader2 class="animate-spin h-10 w-10 text-blue-600" />
        </div>

        <template v-else-if="comments.length > 0">
          <div class="space-y-6">
            <div v-for="comment in processedComments" :key="comment.comment_id">
              <div :style="{ marginLeft: `${getReplyMargin(comment.level)}rem` }" class="transition-all duration-300">
                <CommentItem 
                  :comment="comment" 
                  :level="comment.level" 
                  :definition-link-id="id"
                  :reply-enabled="true" 
                  :flat-style="flatStyle" 
                  @reply="handleReply" 
                />

                <!-- Inline reply form -->
                <transition enter-active-class="transition duration-200 ease-out" enter-from-class="transform -translate-y-2 opacity-0" enter-to-class="transform translate-y-0 opacity-100" leave-active-class="transition duration-150 ease-in" leave-from-class="transform translate-y-0 opacity-100" leave-to-class="transform -translate-y-2 opacity-0">
                  <div v-if="replyToId === comment.comment_id" class="mt-4 ml-4">
                    <CommentForm 
                      :is-submitting="isSubmitting" 
                      :initial-values="newComment" 
                      is-reply 
                      @submit="submitComment"
                      @cancel="cancelComment" 
                    />
                  </div>
                </transition>
              </div>
            </div>
          </div>

          <div v-if="totalPages > 1" class="mt-8 pt-6 border-t border-gray-100">
            <PaginationComponent 
              :current-page="currentPage" 
              :total-pages="totalPages" 
              :total="totalComments" 
              :per-page="perPage"
              @prev="changePage(currentPage - 1)" 
              @next="changePage(currentPage + 1)" 
            />
          </div>
        </template>

        <!-- Empty state -->
        <div v-else class="flex flex-col items-center justify-center py-20 text-center">
          <div class="bg-gray-50 rounded-full p-6 mb-4">
            <MessageSquare class="h-12 w-12 text-gray-300" />
          </div>
          <h4 class="text-xl font-bold text-gray-900 mb-2">{{ t('commentList.noComments') }}</h4>
          <p class="text-gray-500 max-w-sm mb-8">{{ t('definitionLinkDiscussion.noCommentsHint') }}</p>
          <button v-if="auth.state.isLoggedIn" @click="handleNewTopLevelComment"
            class="inline-flex items-center px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-bold rounded-xl transition-all shadow-md active:shadow-sm transform active:translate-y-0.5">
            <Plus class="h-5 w-5 mr-2" />
            {{ t('commentList.newDiscussionWave') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { 
  ArrowUpRight, 
  MessageCircle, 
  MessageSquare, 
  Plus, 
  Loader2, 
  Link as LinkIcon 
} from 'lucide-vue-next'
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { Marked } from 'marked'

import {
  addComment,
  fetchComments,
  getDefinitionLink,
} from '@/api'
import CommentForm from '@/components/CommentForm.vue'
import CommentItem from '@/components/CommentItem.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const props = defineProps({
  id: {
    type: Number,
    required: true
  }
})

const { t, locale } = useI18n()
const route = useRoute()
const router = useRouter()
const auth = useAuth()
const { showError } = useError()
const marked = new Marked()

// State
const linkDetails = ref(null)
const comments = ref([])
const processedComments = ref([])
const isLoading = ref(true)
const isLoadingComments = ref(true)
const isSubmitting = ref(false)
const showTopLevelForm = ref(false)
const replyToId = ref(null)
const currentPage = ref(1)
const perPage = ref(10)
const totalComments = ref(0)
const flatStyle = ref(localStorage.getItem('commentFlatStyle') === 'true')
const newComment = ref({ subject: '', content: '' })

// Computed
const totalPages = computed(() => Math.ceil(totalComments.value / perPage.value))

const toggleFlatStyle = () => {
  flatStyle.value = !flatStyle.value
  localStorage.setItem('commentFlatStyle', flatStyle.value)
}

const renderMarkdown = (text) => {
  if (!text) return ''
  return marked.parse(text)
}

const getReplyMargin = (level) => {
  return Math.min(Math.max(level - 1, 0) * 1.5, 6)
}

const processComments = (commentsList, isFlat) => {
  const levelMap = new Map()
  return commentsList.map(comment => {
    let level = 0
    if (!isFlat && comment.parent_id !== 0) {
      level = (levelMap.get(comment.parent_id) || 0) + 1
    }
    levelMap.set(comment.comment_id, level)
    return {
      ...comment,
      level: isFlat ? 0 : level
    }
  })
}

const loadData = async () => {
  isLoading.value = true
  try {
    const response = await getDefinitionLink(props.id)
    linkDetails.value = response.data
  } catch (error) {
    console.error('Error fetching link details:', error)
    showError(t('definitionLinkDiscussion.errorLoadLink'))
  } finally {
    isLoading.value = false
  }
}

const loadComments = async () => {
  isLoadingComments.value = true
  try {
    const params = new URLSearchParams()
    params.append('definition_link_id', props.id)
    params.append('page', currentPage.value)
    params.append('per_page', perPage.value)
    
    if (route.query.comment_id) params.append('comment_id', route.query.comment_id)
    if (route.query.scroll_to) params.append('scroll_to', route.query.scroll_to)

    const response = await fetchComments(params.toString())
    comments.value = response.data.comments
    totalComments.value = response.data.total
    processedComments.value = processComments(comments.value, flatStyle.value)
    
    if (route.query.scroll_to) {
      nextTick(() => scrollToComment(parseInt(route.query.scroll_to)))
    }
  } catch (error) {
    console.error('Error fetching comments:', error)
  } finally {
    isLoadingComments.value = false
  }
}

const handleNewTopLevelComment = () => {
  showTopLevelForm.value = true
  replyToId.value = null
  newComment.value = { subject: '', content: '' }
}

const handleReply = (commentId) => {
  replyToId.value = commentId
  newComment.value = { subject: '', content: '' }
  showTopLevelForm.value = false
}

const submitComment = async (formData) => {
  isSubmitting.value = true
  try {
    const response = await addComment({
      definition_link_id: props.id,
      parent_id: replyToId.value || undefined,
      subject: formData.subject,
      content: formData.content,
    })
    
    if (response.status === 200) {
      cancelComment()
      await loadComments()
      const newId = response.data.comment_id
      router.replace({ query: { ...route.query, scroll_to: newId } })
      nextTick(() => scrollToComment(newId))
    }
  } catch (error) {
    showError(error.response?.data?.error || t('commentList.errorSubmit'))
  } finally {
    isSubmitting.value = false
  }
}

const cancelComment = () => {
  showTopLevelForm.value = false
  replyToId.value = null
  newComment.value = { subject: '', content: '' }
}

const changePage = (page) => {
  currentPage.value = page
  loadComments()
}

const scrollToComment = (commentId) => {
  const el = document.querySelector(`[data-comment-id="${commentId}"]`)
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' })
    el.classList.add('ring-2', 'ring-blue-400', 'ring-offset-2', 'rounded-lg')
    setTimeout(() => {
      el.classList.remove('ring-2', 'ring-blue-400', 'ring-offset-2')
    }, 3000)
  }
}

onMounted(() => {
  loadData()
  loadComments()
})

watch(flatStyle, (newVal) => {
  processedComments.value = processComments(comments.value, newVal)
})

// SEO
useSeoHead({
  title: computed(() => linkDetails.value 
    ? t('definitionLinkDiscussion.seoTitle', { w1: linkDetails.value.def1_word, w2: linkDetails.value.def2_word }) 
    : t('definitionLinkDiscussion.title')),
  description: computed(() => t('definitionLinkDiscussion.seoDesc'))
}, locale.value)
</script>

<style scoped>
.prose :deep(p) {
  margin-top: 0.5rem;
  margin-bottom: 0.5rem;
}
</style>
