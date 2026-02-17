<template>
  <!-- Header with word being discussed -->
  <div class="mb-6">
    <div v-if="valsiDetails" class="mb-4">
      <h2 v-if="!definitionId" class="text-2xl font-bold space-x-2 select-none">
        <span class="text-gray-500 italic">{{ t('commentList.discussingEntry') }}</span>
        <RouterLink v-if="valsiDetails.valsiid" :to="`/valsi/${valsiDetails.word.replace(/ /g, '_')}`"
          class="text-blue-700 hover:text-blue-800 hover:underline">
          {{ valsiDetails.word }}
        </RouterLink>
        <span v-else class="text-blue-700">
          {{ valsiDetails.word }}
        </span>
      </h2>
      <h2 v-else class="text-2xl font-bold space-x-2 select-none">
        <span class="text-gray-500 italic">{{ t('commentList.discussingDefinition') }}</span>
      </h2>
    </div>
    <div v-if="valsiDetails && definitionDetails" class="mb-4">
      <DefinitionCard :definition="definitionDetails" :languages="languages" :disable-discussion-button="true"
        :disable-discussion-toolbar-button="true" :show-definition-number="true" />
    </div>

    <!-- Action buttons -->
    <div class="flex flex-wrap gap-2 w-full lg:w-auto justify-center">
      <label class="inline-flex items-center" :disabled="flatStyleEnforced"
        :class="[!flatStyle && !flatStyleEnforced ? ' btn-aqua-slate' : 'btn-aqua-white']">
        <input type="checkbox" class="checkmark-aqua" :checked="!flatStyle && !flatStyleEnforced"
          :disabled="flatStyleEnforced" @change="toggleFlatStyle">
        <span class="text-sm select-none" :class="{ 'text-gray-400': flatStyleEnforced }">{{ t('commentList.threaded') }}</span>
      </label>
      <button v-if="auth.state.isLoggedIn && comments.length > 0" class="btn-aqua-emerald"
        @click="handleNewTopLevelComment">
        <AudioWaveform class="h-4 w-4" />
        <span>
          {{ t('commentList.newWave') }}
        </span>
      </button>
      <button v-if="commentId > 0 && !!currentComment?.parent_id" class="inline-flex items-center btn-aqua-purple"
        @click="goToParent">
        <ArrowLeft class="h-5 w-5" />
        {{ t('commentList.parent') }}
      </button>
      <button v-if="commentId > 0" class="inline-flex items-center btn-aqua-rose" @click="goToRoot">
        <Home class="h-5 w-5" />
        {{ t('commentList.waveRoot') }}
      </button>
    </div>
  </div>

  <!-- New top-level comment form -->
  <div v-if="showTopLevelForm" class="mb-6">
    <CommentForm :is-submitting="isSubmitting" :initial-values="newComment"
      class="border border-blue-200 rounded-lg shadow-sm" @submit="submitComment" @cancel="cancelComment" />
  </div>

  <!-- Comments list -->
  <div class="space-y-4">
    <template v-if="!isLoading">
      <!-- Process all comments -->
      <template v-if="commentId > 0">
        <!-- Single comment thread view -->
        <div v-for="comment in targetCommentThread" :key="comment.comment_id" class="relative">
          <div :style="{ marginLeft: `${flatStyle ? 0 : getReplyMargin(comment.level)}rem` }"
            @mouseup="handleTextSelection(comment.comment_id, $event)">
            <CommentItem :comment="comment" :valsi-id="valsiId" :natlang-word-id="natlangWordId"
              :definition-id="definitionId" :reply-enabled="true" :flat-style="flatStyle" @reply="handleReply" />

            <!-- Inline reply form -->
            <div v-if="replyToId === comment.comment_id" class="ml-4">
              <CommentForm :is-submitting="isSubmitting" :initial-values="newComment" is-reply @submit="submitComment"
                @cancel="cancelComment" />
            </div>
          </div>
        </div>
      </template>
      <template v-else>
        <!-- All comments in order -->
        <div v-for="comment in processedComments" :key="comment.comment_id" class="relative">
          <div :style="{ marginLeft: `${getReplyMargin(comment.level)}rem` }"
            @mouseup="handleTextSelection(comment.comment_id, $event)">
            <CommentItem :comment="comment" :level="comment.level" :valsi-id="valsiId" :natlang-word-id="natlangWordId"
              :definition-id="definitionId" :reply-enabled="true" :flat-style="flatStyle" @reply="handleReply" />

            <!-- Inline reply form -->
            <div v-if="replyToId === comment.comment_id" class="ml-4">
              <CommentForm :is-submitting="isSubmitting" :initial-values="newComment" is-reply @submit="submitComment"
                @cancel="cancelComment" />
            </div>
          </div>
        </div>
      </template>
    </template>

    <div v-if="!isLoading && totalPages > 1" class="mt-6">
      <PaginationComponent :current-page="currentPage" :total-pages="totalPages" :total="total" :per-page="perPage"
        @prev="changePage(currentPage - 1)" @next="changePage(currentPage + 1)" />
    </div>

    <!-- Loading state -->
    <div v-if="isLoading" class="flex justify-center py-8">
      <Loader2 class="animate-spin h-8 w-8 text-blue-600" />
    </div>

    <!-- Empty state -->
    <div v-if="!isLoading && comments.length === 0"
      class="flex flex-col justify-center text-center py-12 bg-blue-50 rounded-lg border border-blue-100 p-4">
      <MessageSquare class="mx-auto h-12 w-12 text-blue-400" />
      <p class="my-4 text-gray-600">
        {{ t('commentList.noComments') }}
      </p>
      <button v-if="auth.state.isLoggedIn" class="btn-aqua-emerald h-8 text-base mx-auto"
        @click="handleNewTopLevelComment">
        <AudioWaveform class="h-4 w-4" />
        <span>
          {{ t('commentList.newDiscussionWave') }}
        </span>
      </button>
    </div>
    <!-- Floating quote button -->
    <div v-if="quotePosition.visible" class="fixed z-50 bg-white border border-gray-300 rounded-md shadow-sm p-1"
      :style="{
        left: `${quotePosition.x}px`,
        top: `${quotePosition.y}px`
      }">
      <button @click="handleQuote" class="text-sm px-2 py-1 hover:bg-gray-100 rounded-md flex items-center">
        <Quote class="w-4 h-4 mr-1" />
        {{ t('commentList.quoteSelectedText') }}
      </button>
    </div>
  </div>
</template>

<script setup>
import { ArrowLeft, Home, MessageSquare, AudioWaveform, Loader2, Quote } from 'lucide-vue-next'
import { ref, computed, onMounted, watchEffect, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import {
  addComment,
  fetchComments,
  getValsiAndDefinitionDetails as getEntriesAndDefinitionDetails,
  getLanguages,
} from '@/api'
import CommentForm from '@/components/CommentForm.vue'
import CommentItem from '@/components/CommentItem.vue'
import DefinitionCard from '@/components/DefinitionCard.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

// Props
const props = defineProps({
  valsiId: {
    type: Number,
    default: 0,
  },
  natlangWordId: {
    type: Number,
    default: 0,
  },
  definitionId: {
    type: Number,
    default: 0,
  },
  commentId: {
    type: Number,
    default: 0,
  },
  scrollTo: {
    type: Number,
    default: 0,
  },
  threadId: {
    type: Number,
    default: 0,
  },
})

// Setup
const route = useRoute()
const router = useRouter()
const auth = useAuth()
const { showError } = useError() // Get showError function
const { t, locale } = useI18n()

const levelMap = new Map()

// State
const lastBuildQuery = ref('')
const comments = ref([])
const isLoading = ref(true)
const showTopLevelForm = ref(false)
const replyToId = ref(null)
const isSubmitting = ref(false)
const newComment = ref({
  subject: '',
  content: '',
})
const flatStyle = ref((typeof window === 'undefined') ? '' : localStorage.getItem('commentFlatStyle') === 'true')
const flatStyleEnforced = ref(false)
const selectedCommentId = ref(null)
const selectedText = ref('')
const quotePosition = ref({ x: 0, y: 0, visible: false })

const toggleFlatStyle = () => {
  if (typeof window === 'undefined') return;

  flatStyle.value = !flatStyle.value
  localStorage.setItem('commentFlatStyle', flatStyle.value)
}

const processedComments = ref([])

const processComments = (comments, isFlat) => {
  levelMap.clear()
  return comments.map(comment => {
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

// For single comment thread view
const targetCommentThread = computed(() => {
  if (!props.commentId) return []

  // Create a map for quick parent lookup
  const parentMap = new Map()
  comments.value.forEach((comment) => {
    if (!parentMap.has(comment.parent_id)) {
      parentMap.set(comment.parent_id, [])
    }
    parentMap.get(comment.parent_id).push(comment)
  })

  // Calculate levels using the same approach as processedComments
  const processLevel = (comment) => {
    let level = 0
    if (comment.parent_id !== 0) {
      level = (levelMap.get(comment.parent_id) || 0) + 1
    }
    levelMap.set(comment.comment_id, level)
    return level
  }

  // Start with target comment and add all related comments
  const thread = comments.value.map((comment) => ({
    ...comment,
    level: processLevel(comment),
  }))

  return thread
})

const scrollToComment = async (commentId) => {
  // Wait for Vue to update the DOM
  await nextTick()
  // Wait an additional tick to ensure comments are rendered
  await nextTick()

  const commentElement = document.querySelector(`[data-comment-id="${commentId}"]`)

  if (commentElement) {
    // Add a small delay to ensure smooth scrolling
    setTimeout(() => {
      commentElement.scrollIntoView({
        behavior: 'instant',
        block: 'start',
        inline: 'nearest',
      })
      commentElement.classList.add('highlight-comment')
      setTimeout(() => commentElement.classList.remove('highlight-comment'), 5800)
    }, 100)
  }
}

const getReplyMargin = (level) => {
  return Math.min(Math.max(level - 1, 0) * 2, 8);
}

const handleNewTopLevelComment = () => {
  if (props.valsiId === 0 && props.definitionId === 0) {
    router.push('/comments/new-thread');
  } else {
    showTopLevelForm.value = true;
    replyToId.value = null;
  }
}


const handleTextSelection = (commentId, event) => {
  const selection = window.getSelection()
  if (selection.toString().trim() && selection.rangeCount > 0) {
    const range = selection.getRangeAt(0)
    if (event.currentTarget.contains(range.commonAncestorContainer)) {
      selectedText.value = selection.toString().trim()
      selectedCommentId.value = commentId
      const rect = range.getBoundingClientRect()
      quotePosition.value = {
        x: rect.left + window.pageXOffset,
        y: rect.top + window.pageYOffset - 60,
        visible: true
      }
    } else {
      quotePosition.value.visible = false
    }
  } else {
    quotePosition.value.visible = false
  }
}

const handleQuote = () => {
  if (!selectedText.value || !selectedCommentId.value) return
  replyToId.value = selectedCommentId.value
  newComment.value.content = `> ${selectedText.value.split('\n').join('\n> ')}\n\n`
  selectedText.value = ''
  quotePosition.value.visible = false
  nextTick(() => {
    const formComponent = document.querySelector('.milkdown-editor')
    if (formComponent) {
      const observer = new MutationObserver(() => {
        const editorElement = formComponent.querySelector('.ProseMirror')
        if (editorElement) {
          editorElement.focus()
          observer.disconnect()
        }
      })

      observer.observe(formComponent, {
        childList: true,
        subtree: true
      })
    }
  })
}

const handleReply = (commentId) => {
  replyToId.value = commentId
  newComment.value = { subject: '', content: '' }
  showTopLevelForm.value = false
  nextTick(() => {
    const formComponent = document.querySelector('.milkdown-editor')
    if (formComponent) {
      const observer = new MutationObserver(() => {
        const editorElement = formComponent.querySelector('.ProseMirror')
        if (editorElement) {
          editorElement.focus()
          observer.disconnect()
        }
      })

      observer.observe(formComponent, {
        childList: true,
        subtree: true
      })
    }
  })
}

const performFetchComments = async (isInitialLoad = false, scrollTo) => {
  if (typeof window === 'undefined') return;

  scrollTo = scrollTo || props.scrollTo
  try {
    const buildQuery = buildQueryString(!isInitialLoad)

    // Only update comments if they've actually changed
    if (lastBuildQuery.value !== buildQuery) {
      const response = await fetchComments(buildQuery)
      lastBuildQuery.value = buildQuery
      comments.value = response.data.comments
      processedComments.value = processComments(response.data.comments, flatStyle.value)
      total.value = response.data.total

      // Automatically enable flat style if any comment level >4
      const unflattenedComments = processComments(response.data.comments, false)
      const maxLevel = Math.max(...unflattenedComments.map(c => c.level))
      if (maxLevel > 4) {
        flatStyle.value = true
        processedComments.value = processComments(response.data.comments, true)
        flatStyleEnforced.value = true
      } else {
        flatStyle.value = localStorage.getItem('commentFlatStyle') === 'true'
        flatStyleEnforced.value = false
      }

      // Wait for DOM updates if we changed comments
      if (comments.value !== response.data.comments) {
        await nextTick()
      }
    }

    await nextTick()

    if (scrollTo > 0) {
      // Add a small delay to ensure smooth scrolling
      setTimeout(() => {
        scrollToComment(scrollTo)
      }, 50)
    }
  } catch (error) {
    console.error('Error fetching comments:', error)
  } finally {
    isLoading.value = false
  }
}

const currentPage = ref(1)
const perPage = ref(10)
const total = ref(0)

const currentComment = computed(() => comments.value.find(c => c.comment_id === props.commentId))

const totalPages = computed(() => Math.ceil(total.value / perPage.value))

const buildQueryString = (includePage = true) => {
  const params = new URLSearchParams()
  if (props.valsiId) params.append('valsi_id', props.valsiId)
  if (props.natlangWordId) params.append('natlang_word_id', props.natlangWordId)
  if (props.definitionId) params.append('definition_id', props.definitionId)
  if (props.commentId) params.append('comment_id', props.commentId)
  if (props.scrollTo) params.append('scroll_to', props.scrollTo)
  if (props.threadId) params.append('thread_id', props.threadId)
  if (includePage) {
    params.append('page', currentPage.value)
  }
  params.append('per_page', perPage.value)

  return params.toString()
}

const changePage = (page) => {
  currentPage.value = page
  // Subsequent page changes should include page parameter
  performFetchComments(false)
  // window.scrollTo({ top: 0, behavior: 'smooth' });
}

const submitComment = async (formData) => {
  try {
    isSubmitting.value = true
    const response = await addComment({
      valsi_id: props.valsiId || undefined,
      natlang_word_id: props.natlangWordId || undefined,
      definition_id: props.definitionId,
      parent_id: replyToId.value || undefined,
      subject: formData.subject,
      content: formData.content,
    })
    if (response.status === 200) {
      const newCommentId = response.data.comment_id
      await performFetchComments()
      cancelComment()
      router.replace({
        query: {
          ...route.query,
          thread_id: response.data.thread_id,
          comment_id: response.data.comment_id,
          scroll_to: newCommentId
        }
      })
      await nextTick()
      scrollToComment(newCommentId)
    }
  } catch (error) {
    console.error('Error submitting comment:', error);
    // Use the useError composable to show the error
    showError(error.response?.data?.error || 'Failed to submit comment', error.response?.data?.details);
  } finally {
    isSubmitting.value = false;
  }
}

const cancelComment = () => {
  showTopLevelForm.value = false
  replyToId.value = null
  newComment.value = { subject: '', content: '' }
  quotePosition.value.visible = false
}

const goToParent = () => {
  const currentComment = comments.value.find((c) => c.comment_id === props.commentId)
  if (currentComment) {
    router.push({
      query: {
        ...route.query,
        comment_id: currentComment.parent_id || 0,
      },
    })
  }
}

const goToRoot = () => {
  router.push({
    query: {
      ...route.query,
      comment_id: 0,
    },
  })
}

const languages = ref([])
const valsiDetails = ref(null)
const definitionDetails = ref(null)

watch(flatStyle, () => {
  processedComments.value = processComments(comments.value, flatStyle.value)
})

// SEO: title and description at least as informative as the page content
const pageTitle = computed(() => {
  if (valsiDetails.value?.word) {
    if (definitionDetails.value) {
      return t('commentList.titleDefinition', { word: valsiDetails.value.word })
    }
    return t('commentList.titleEntry', { word: valsiDetails.value.word })
  }
  return t('commentList.titleDefault')
})

const metaDescription = computed(() => {
  if (valsiDetails.value?.word) {
    if (definitionDetails.value) {
      return t('commentList.metaDescriptionDefinition', { word: valsiDetails.value.word })
    }
    return t('commentList.metaDescriptionEntry', { word: valsiDetails.value.word })
  }
  return t('commentList.metaDescriptionDefault')
})

const canonicalPath = computed(() => route.fullPath)

useSeoHead({
  title: pageTitle,
  description: metaDescription,
  canonical: canonicalPath,
}, locale.value)

const fetchDefinitionsAndDetails = async () => {
  if (props.valsiId) {
    try {
      const result = await getEntriesAndDefinitionDetails(props.valsiId, props.definitionId)
      valsiDetails.value = result.valsi.valsi

      definitionDetails.value = result.definition

      const langsResponse = await getLanguages()
      languages.value = langsResponse.data
    } catch (error) {
      console.error('Error fetching details:', error)
    }
  }
}

onMounted(async () => {
  await fetchDefinitionsAndDetails()
  // Pass true for initial load
  await performFetchComments(true)
})

// Watch for route changes to refresh comments
watchEffect(async () => {
  const needsRefresh =
    route.query.valsi_id ||
    route.query.natlang_word_id ||
    route.query.definition_id ||
    route.query.comment_id ||
    route.query.thread_id

  if (needsRefresh) {
    await performFetchComments(true, route.query.scroll_to)
  } else if (route.query.scroll_to) {
    // Only scroll if the data hasn't changed
    setTimeout(() => {
      scrollToComment(route.query.scroll_to)
    }, 50)
  }
})
</script>

<style>
:root {
  --max-reply-margin: 8rem;
}

@media (max-width: 768px) {
  :root {
    --max-reply-margin: 6rem;
  }
}
</style>
