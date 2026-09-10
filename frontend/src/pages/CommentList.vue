<template>
  <div :class="embedded ? undefined : 'feed-page'">
    <!--
      Standalone: capped top strip (own scrollbar, < half viewport).
      Embedded: inline above the thread (modal / collection tab).
    -->
    <div
      v-if="showDefinitionPanel"
      :class="[
        embedded ? 'mb-4' : 'feed-page__definition',
        !embedded && definitionDockCollapsed ? 'feed-page__definition--collapsed' : null,
      ]"
    >
      <div class="discussion-definition-scroll">
        <div
          v-if="isDefinitionLoading"
          class="discussion-definition-skeleton"
          aria-hidden="true"
        >
          <div class="discussion-definition-skeleton__title ml-4 mt-0" />
          <div class="discussion-definition-skeleton__card">
            <div class="discussion-definition-skeleton__meta">
              <div class="discussion-definition-skeleton__chip w-16" />
              <div class="discussion-definition-skeleton__chip w-24" />
              <div class="discussion-definition-skeleton__chip w-28" />
            </div>
            <div class="discussion-definition-skeleton__line w-full" />
            <div class="discussion-definition-skeleton__line w-5/6" />
            <div class="discussion-definition-skeleton__line w-3/4" />
            <div class="discussion-definition-skeleton__line w-2/3" />
          </div>
        </div>
        <template v-else>
          <div
            v-if="valsiDetails && (embedded || !definitionDockCollapsed)"
            class="mb-1.5 px-4 pt-1"
          >
            <h2 v-if="!definitionId" class="text-lg font-bold space-x-2 select-none leading-snug">
              <span class="text-gray-500 italic">{{ t('commentList.discussingEntry') }}</span>
              <RouterLink
                v-if="valsiDetails.valsiid"
                :to="`/valsi/${valsiDetails.word.replace(/ /g, '_')}`"
                class="text-blue-700 hover:text-blue-800 hover:underline"
              >
                {{ valsiDetails.word }}
              </RouterLink>
              <span v-else class="text-blue-700"> {{ valsiDetails.word }} </span>
            </h2>

            <h2 v-else class="text-lg font-bold space-x-2 select-none leading-snug">
              <span class="text-gray-500 italic">{{ t('commentList.discussingDefinition') }}</span>
            </h2>
          </div>

          <DefinitionCard
            v-if="valsiDetails && definitionDetails"
            :definition="definitionDetails"
            :languages="languages"
            :disable-discussion-button="true"
            :disable-discussion-toolbar-button="true"
            :disable-toolbar="true"
            :disable-border="true"
            :show-definition-number="true"
          />
        </template>
      </div>
    </div>

    <!-- Lower part: own scrollbar when standalone -->
    <div
      :class="embedded ? undefined : 'feed-page__body'"
      @scroll.passive="onCommentsBodyScroll"
    >
      <!-- Action buttons -->
      <div class="flex flex-wrap gap-2 w-full lg:w-auto justify-center mb-4">
        <label
          class="inline-flex items-center"
          :disabled="flatStyleEnforced"
          :class="[!flatStyle && !flatStyleEnforced ? ' ui-btn--neutral-slate' : 'ui-btn--neutral']"
        >
          <Checkbox
            class="checkmark-aqua"
            :checked="!flatStyle && !flatStyleEnforced"
            :disabled="flatStyleEnforced"
            @change="toggleFlatStyle"
          />
          <span class="text-sm select-none" :class="{ 'text-gray-400': flatStyleEnforced }">{{
            t('commentList.threaded')
          }}</span>
        </label>
        <Button
          v-if="commentId > 0 && !!currentComment?.parent_id"
          variant="accent-purple"
          class="inline-flex items-center ui-btn--accent-purple"
          @click="goToParent"
        >
          <ArrowLeft class="h-5 w-5" /> {{ t('commentList.parent') }}
        </Button>
        <Button
          v-if="commentId > 0"
          variant="neutral-slate"
          type="button"
          class="inline-flex items-center ui-btn--neutral-slate"
          :aria-label="t('commentList.waveRoot')"
          @click="goToRoot"
        >
          <Home class="h-5 w-5" /> {{ t('commentList.waveRoot') }}
        </Button>
      </div>

      <!-- Top-level comment composer -->
      <div v-if="showTopLevelComposer" class="mb-4">
        <CommentForm
          :key="composerKey"
          :is-submitting="isSubmitting"
          :initial-values="newComment"
          class="border border-blue-200 rounded-lg shadow-sm"
          @submit="submitComment"
          @cancel="resetComposerState"
        />
      </div>

      <!-- Comments list -->
      <div class="space-y-4">
        <template v-if="!isLoading">
          <template v-if="commentId > 0">
            <div v-for="comment in targetCommentThread" :key="comment.comment_id" class="relative">
              <div
                :style="{ marginLeft: `${flatStyle ? 0 : getReplyMargin(comment.level)}rem` }"
                @mouseup="handleTextSelection(comment.comment_id, $event)"
              >
                <CommentItem
                  :comment="comment"
                  :valsi-id="valsiId"
                  :natlang-word-id="natlangWordId"
                  :definition-id="definitionId"
                  :reply-enabled="true"
                  :flat-style="flatStyle"
                  @reply="handleReply"
                />
                <div v-if="replyToId === comment.comment_id" class="ml-4">
                  <CommentForm
                    :is-submitting="isSubmitting"
                    :initial-values="newComment"
                    :is-reply="true"
                    @submit="submitComment"
                    @cancel="cancelReply"
                  />
                </div>
              </div>
            </div>
          </template>
          <template v-else>
            <div v-for="comment in processedComments" :key="comment.comment_id" class="relative">
              <div
                :style="{ marginLeft: `${getReplyMargin(comment.level)}rem` }"
                @mouseup="handleTextSelection(comment.comment_id, $event)"
              >
                <CommentItem
                  :comment="comment"
                  :level="comment.level"
                  :valsi-id="valsiId"
                  :natlang-word-id="natlangWordId"
                  :definition-id="definitionId"
                  :reply-enabled="true"
                  :flat-style="flatStyle"
                  @reply="handleReply"
                />
                <div v-if="replyToId === comment.comment_id" class="ml-4">
                  <CommentForm
                    :is-submitting="isSubmitting"
                    :initial-values="newComment"
                    :is-reply="true"
                    @submit="submitComment"
                    @cancel="cancelReply"
                  />
                </div>
              </div>
            </div>
          </template>
        </template>

        <div v-if="embedded && !isLoading && totalPages > 1" class="mt-6">
          <PaginationComponent
            :current-page="currentPage"
            :total-pages="totalPages"
            :total="total"
            :per-page="perPage"
            @prev="changePage(currentPage - 1)"
            @next="changePage(currentPage + 1)"
          />
        </div>

        <div v-if="isLoading" class="flex justify-center py-8">
          <Loader2 class="animate-spin h-8 w-8 text-blue-600" />
        </div>

        <div
          v-if="!isLoading && comments.length === 0"
          class="flex flex-col justify-center text-center py-8 bg-blue-50 rounded-lg border border-blue-100 p-4"
        >
          <MessageSquare class="mx-auto h-12 w-12 text-blue-400" />
          <p class="mt-4 text-gray-600">{{ t('commentList.noComments') }}</p>
        </div>

        <div
          v-if="quotePosition.visible"
          class="fixed z-50 bg-white border border-gray-300 rounded-md shadow-sm p-1"
          :style="{
            left: `${quotePosition.x}px`,
            top: `${quotePosition.y}px`,
          }"
        >
          <Button
            variant="neutral"
            class="text-sm px-2 py-1 hover:bg-gray-100 rounded-md flex items-center"
            @click="handleQuote"
          >
            <Quote class="w-4 h-4 mr-1" /> {{ t('commentList.quoteSelectedText') }}
          </Button>
        </div>
      </div>
    </div>

    <!-- Pinned pagination footer (standalone, mirrors RecentChanges) -->
    <div v-if="!embedded && !isLoading && totalPages > 1" class="feed-page__footer">
      <div class="feed-page__footer-inner">
        <div class="feed-page__pagination">
          <PaginationComponent
            :current-page="currentPage"
            :total-pages="totalPages"
            :total="total"
            :per-page="perPage"
            @prev="changePage(currentPage - 1)"
            @next="changePage(currentPage + 1)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button, Checkbox } from '@packages/ui'
import { ArrowLeft, Home, MessageSquare, Loader2, Quote } from '@lucide/vue'
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
import { queryStr } from '@/utils/routeQuery'

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
  definitionLinkId: {
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
  collectionId: {
    type: Number,
    default: 0,
  },
  embedded: {
    type: Boolean,
    default: false,
  },
  initialSubject: {
    type: String,
    default: '',
  },
})

// Setup
const route = useRoute()
const router = useRouter()
const auth = useAuth()
const { showError } = useError() // Get showError function
const { t } = useI18n()

const levelMap = new Map()

// State
const lastBuildQuery = ref('')
const comments = ref([])
const isLoading = ref(true)
const replyToId = ref(null)
const composerKey = ref(0)
const isSubmitting = ref(false)
const newComment = ref({
  subject: '',
  content: '',
})
const flatStyle = ref(
  typeof window === 'undefined' ? false : localStorage.getItem('commentFlatStyle') === 'true'
)
const flatStyleEnforced = ref(false)
const selectedCommentId = ref(null)
const selectedText = ref('')
const quotePosition = ref({ x: 0, y: 0, visible: false })

const toggleFlatStyle = () => {
  if (typeof window === 'undefined') return

  flatStyle.value = !flatStyle.value
  localStorage.setItem('commentFlatStyle', String(flatStyle.value))
}

const processedComments = ref([])

const processComments = (
  comments: Array<Record<string, unknown> & { comment_id: number; parent_id: number }>,
  isFlat: boolean
) => {
  levelMap.clear()
  return comments.map((comment) => {
    let level = 0
    if (!isFlat && comment.parent_id !== 0) {
      level = (levelMap.get(comment.parent_id) || 0) + 1
    }
    levelMap.set(comment.comment_id, level)
    return {
      ...comment,
      level: isFlat ? 0 : level,
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

const scrollToComment = async (commentId: number | string) => {
  // Wait for Vue to update the DOM
  await nextTick()
  // Wait an additional tick to ensure comments are rendered
  await nextTick()

  const commentElement = document.querySelector(`[data-comment-id="${String(commentId)}"]`)

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

const getReplyMargin = (level: number) => {
  return Math.min(Math.max(level - 1, 0) * 2, 8)
}

const hasDiscussionContext = computed(
  () =>
    !!(
      props.valsiId ||
      props.definitionId ||
      props.collectionId ||
      props.natlangWordId ||
      props.definitionLinkId
    )
)

const showTopLevelComposer = computed(
  () => auth.state.isLoggedIn && hasDiscussionContext.value
)

/** Scroll the comments body only (same pattern as RecentChanges). */
function scrollCommentsBodyToTop() {
  if (typeof document === 'undefined') return
  if (props.embedded) return
  const el = document.querySelector('.feed-page__body')
  if (el) el.scrollTo({ top: 0, left: 0, behavior: 'smooth' })
  else window.scrollTo({ top: 0, left: 0, behavior: 'smooth' })
  definitionDockCollapsed.value = false
}

/** Collapse definition dock while comments are scrolled; restore at top. */
const definitionDockCollapsed = ref(false)
const DEFINITION_COLLAPSE_AT = 32
const DEFINITION_EXPAND_AT = 8

function onCommentsBodyScroll(event: Event) {
  if (props.embedded || !showDefinitionPanel.value) return
  const top = (event.target as HTMLElement).scrollTop
  if (!definitionDockCollapsed.value && top > DEFINITION_COLLAPSE_AT) {
    definitionDockCollapsed.value = true
  } else if (definitionDockCollapsed.value && top <= DEFINITION_EXPAND_AT) {
    definitionDockCollapsed.value = false
  }
}

const handleTextSelection = (commentId: number, event: MouseEvent) => {
  const selection = window.getSelection()
  if (selection.toString().trim() && selection.rangeCount > 0) {
    const range = selection.getRangeAt(0)
    if ((event.currentTarget as Node).contains(range.commonAncestorContainer)) {
      selectedText.value = selection.toString().trim()
      selectedCommentId.value = commentId
      const rect = range.getBoundingClientRect()
      quotePosition.value = {
        x: rect.left + window.pageXOffset,
        y: rect.top + window.pageYOffset - 60,
        visible: true,
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
          ;(editorElement as HTMLElement).focus()
          observer.disconnect()
        }
      })

      observer.observe(formComponent, {
        childList: true,
        subtree: true,
      })
    }
  })
}

const handleReply = (commentId: number) => {
  replyToId.value = commentId
  newComment.value = { subject: '', content: '' }
  nextTick(() => {
    const formComponent = document.querySelector('.milkdown-editor')
    if (formComponent) {
      const observer = new MutationObserver(() => {
        const editorElement = formComponent.querySelector('.ProseMirror')
        if (editorElement) {
          ;(editorElement as HTMLElement).focus()
          observer.disconnect()
        }
      })

      observer.observe(formComponent, {
        childList: true,
        subtree: true,
      })
    }
  })
}

const performFetchComments = async (isInitialLoad = false, scrollTo?: number) => {
  if (typeof window === 'undefined') return

  const effectiveScroll = scrollTo ?? props.scrollTo
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
      const maxLevel = Math.max(...unflattenedComments.map((c) => c.level))
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

    if (effectiveScroll > 0) {
      // Add a small delay to ensure smooth scrolling
      setTimeout(() => {
        scrollToComment(effectiveScroll)
      }, 50)
    }

    if (isInitialLoad && route.query.reply_to) {
      const replyTo = Number(queryStr(route.query.reply_to)) || 0
      if (replyTo > 0) {
        handleReply(replyTo)
      }
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

const currentComment = computed(() => comments.value.find((c) => c.comment_id === props.commentId))

const totalPages = computed(() => Math.ceil(total.value / perPage.value))

const buildQueryString = (includePage = true) => {
  const params = new URLSearchParams()
  if (props.valsiId) params.append('valsi_id', String(props.valsiId))
  if (props.natlangWordId) params.append('natlang_word_id', String(props.natlangWordId))
  if (props.definitionId) params.append('definition_id', String(props.definitionId))
  if (props.definitionLinkId) params.append('definition_link_id', String(props.definitionLinkId))
  if (props.collectionId) params.append('collection_id', String(props.collectionId))
  if (props.commentId) params.append('comment_id', String(props.commentId))
  if (props.scrollTo) params.append('scroll_to', String(props.scrollTo))
  if (props.threadId) params.append('thread_id', String(props.threadId))
  if (includePage) {
    params.append('page', String(currentPage.value))
  }
  params.append('per_page', String(perPage.value))

  return params.toString()
}

const changePage = (page: number) => {
  currentPage.value = page
  // Subsequent page changes should include page parameter
  performFetchComments(false)
  scrollCommentsBodyToTop()
}

const submitComment = async (formData: { subject: string; content: string }) => {
  try {
    isSubmitting.value = true
    const response = await addComment({
      valsi_id: props.valsiId || undefined,
      natlang_word_id: props.natlangWordId || undefined,
      definition_id: props.definitionId || undefined,
      definition_link_id: props.definitionLinkId || undefined,
      collection_id: props.collectionId || undefined,
      parent_id: replyToId.value || undefined,
      subject: formData.subject,
      content: formData.content,
    })
    if (response.status === 200) {
      const newCommentId = response.data.comment_id
      await performFetchComments(false)
      resetComposerState()
      router.replace({
        query: {
          ...route.query,
          thread_id: response.data.thread_id,
          comment_id: response.data.comment_id,
          scroll_to: newCommentId,
        },
      })
      await nextTick()
      scrollToComment(newCommentId)
    }
  } catch (error) {
    console.error('Error submitting comment:', error)
    showError(
      error.response?.data?.error || 'Failed to submit comment',
      error.response?.data?.details
    )
  } finally {
    isSubmitting.value = false
  }
}

const resetComposerState = () => {
  replyToId.value = null
  newComment.value = { subject: '', content: '' }
  quotePosition.value.visible = false
  composerKey.value += 1
}

const cancelReply = () => {
  replyToId.value = null
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
/** True while entry/definition fetch is in flight (starts true when route expects them). */
const isDefinitionLoading = ref(!!(props.valsiId || props.definitionId))

/** Route expects a definition dock (reserve space immediately to avoid CLS). */
const expectsDefinitionPanel = computed(() => !!(props.valsiId || props.definitionId))

/** Show dock when loading expected context, or once details have arrived. */
const showDefinitionPanel = computed(
  () =>
    expectsDefinitionPanel.value ||
    !!(valsiDetails.value || definitionDetails.value)
)

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

if (!props.embedded) {
  useSeoHead({
    pathWithoutLocale: '/comments',
    title: pageTitle,
    description: metaDescription,
    canonical: canonicalPath,
  })
}

const fetchDefinitionsAndDetails = async () => {
  if (!props.valsiId) {
    isDefinitionLoading.value = false
    return
  }
  isDefinitionLoading.value = true
  try {
    const result = await getEntriesAndDefinitionDetails(props.valsiId, props.definitionId)
    valsiDetails.value = result.valsi.valsi

    definitionDetails.value = result.definition

    const langsResponse = await getLanguages()
    languages.value = langsResponse.data
  } catch (error) {
    console.error('Error fetching details:', error)
  } finally {
    isDefinitionLoading.value = false
  }
}

onMounted(async () => {
  if (props.initialSubject && !newComment.value.subject) {
    newComment.value.subject = props.initialSubject
  }
  await fetchDefinitionsAndDetails()
  // Pass true for initial load
  await performFetchComments(true)
})

// Watch for route changes to refresh comments
watchEffect(async () => {
  if (props.embedded) return
  const needsRefresh =
    route.query.valsi_id ||
    route.query.natlang_word_id ||
    route.query.definition_id ||
    route.query.definition_link_id ||
    route.query.collection_id ||
    route.query.comment_id ||
    route.query.thread_id

  if (needsRefresh) {
    const st = route.query.scroll_to
    const scrollNum = st !== undefined ? Number(queryStr(st)) || 0 : undefined
    await performFetchComments(true, scrollNum)
  } else if (route.query.scroll_to) {
    // Only scroll if the data hasn't changed
    setTimeout(() => {
      scrollToComment(queryStr(route.query.scroll_to))
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
