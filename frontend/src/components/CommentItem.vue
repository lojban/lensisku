<template>
  <div class="comment-item bg-white border rounded-lg p-3 my-2 hover:border-blue-300 transition-colors min-w-48"
    :data-comment-id="processedComment.comment_id">
    <div v-if="showContext && (processedComment.definition || processedComment.valsi_id)"
      class="mb-2 text-sm text-gray-600 whitespace-nowrap overflow-hidden flex items-center">
      <span class="bg-blue-50 text-blue-700 px-2.5 py-0.5 rounded-full">
        {{ processedComment.definition_id ? t('components.commentItem.inDefinition') : t('components.commentItem.inValsi') }}
      </span>
      <RouterLink v-if="processedComment.definition"
        :to="`/valsi/${processedComment.valsi_word}?highlight_definition_id=${processedComment.definition_id}`"
        class="hover:underline text-blue-700 font-medium ml-1.5 truncate inline-block max-w-[calc(100%-120px)]">
        <LazyMathJax :content="processedComment.definition" class="inline" />
      </RouterLink>
      <RouterLink v-else-if="processedComment.valsi_id" :to="`/valsi/${processedComment.valsi_word}`"
        class="hover:underline text-blue-700 font-medium ml-1.5 truncate inline-block max-w-[calc(100%-120px)]">
        {{ processedComment.valsi_word || t('components.commentItem.untitledEntry') }}
      </RouterLink>
    </div>
    <!-- Comment Header -->
    <div class="flex flex-col mb-2">
      <div class="flex items-start justify-between">
        <!-- Left side: Profile image and basic info -->
        <div class="flex items-start space-x-2">
          <RouterLink :to="`/user/${processedComment.username}`" class="flex-shrink-0">
            <!-- Skeleton while loading -->
            <div v-show="isProfileImageLoading" class="w-8 h-8 sm:w-10 sm:h-10 rounded-full bg-gray-200 animate-pulse border-2 border-white shadow-sm"></div>
            
            <!-- Actual image when loaded -->
            <img v-if="hasProfileImage" v-show="!isProfileImageLoading"
              :src="getProfileImageUrl(processedComment.username)"
              :alt="`${processedComment.username}'s profile picture`"
              class="w-8 h-8 sm:w-10 sm:h-10 rounded-full object-cover border-2 border-white shadow-sm"
              @load="handleImageLoad"
              @error="handleImageError">
              
            <!-- Placeholder when no image -->
            <div v-if="!hasProfileImage" v-show="!isProfileImageLoading"
              class="w-8 h-8 sm:w-10 sm:h-10 rounded-full bg-gray-200 flex items-center justify-center text-gray-400">
              <User class="h-4 w-4 sm:h-6 sm:w-6" />
            </div>
          </RouterLink>

          <div class="flex-1 min-w-0">
            <div class="flex flex-wrap items-baseline gap-1.5">
              <RouterLink :to="`/user/${processedComment.username}`"
                class="text-sm font-medium text-gray-700 hover:text-blue-600 hover:underline truncate">
                {{ processedComment.username }}
              </RouterLink>
            </div>
            <div class="text-xs text-gray-500">
              {{ formatDate(processedComment.time) }}
            </div>
          </div>
        </div>

        <div class="flex flex-row items-center gap-3">
          <RouterLink
            :to="`/comments/?comment_id=${processedComment.parent_id}&scroll_to=${processedComment.comment_id}&valsi_id=${props.valsiId || 0}&definition_id=${props.definitionId || 0}`"
            class="text-sm text-gray-500 hover:text-blue-600 hover:underline break-all">
            #{{ processedComment.comment_num }}
          </RouterLink>
        </div>
      </div>

      <div class="mt-2">
        <template v-if="processedComment.subject">
          <h4 class="font-medium text-blue-700 text-sm sm:text-base">
            {{ processedComment.subject }}
          </h4>
        </template>
      </div>
    </div>

    <div v-if="flatStyle && processedComment.parent_content" class="min-w-48 overflow-x-auto">
      <div class="mb-2 ml-6 pl-2 border rounded-md border-l-2 border-gray-300">
        <RouterLink
          :to="`/comments/?thread_id=${processedComment.thread_id}&comment_id=${processedComment.parent_id}&scroll_to=${processedComment.parent_id}&valsi_id=${valsiId || 0}&definition_id=${definitionId || 0}`"
          class="text-xs text-gray-600 hover:text-blue-500">
          <div class="text-sm">
            <div class="flex items-center gap-1 mb-1">
              <span class="text-xs">{{ t('components.commentItem.replyingTo') }}</span>
              #{{ processedComment.comment_num - 1 }}
            </div>
            <div class="italic prose prose-sm max-w-none [&_img]:max-h-48 [&_img]:object-contain">
              <template v-for="(part, index) in processedComment.parent_content" :key="index">
                <div v-if="['text', 'header'].includes(part.type)">
                  <LazyMathJax :content="part.data" :enable-markdown="true" />
                </div>
              </template>
            </div>
          </div>
        </RouterLink>
      </div>
    </div>

    <div v-if="showParentInThread && processedComment.parent_content" class="min-w-48 overflow-x-auto">
      <div class="pl-2 border rounded-md border-l-2 border-gray-300">
        <RouterLink
          :to="`/comments/?thread_id=${processedComment.thread_id}&comment_id=${processedComment.parent_id}&scroll_to=${processedComment.parent_id}&valsi_id=${valsiId || 0}&definition_id=${definitionId || 0}`"
          class="text-xs text-gray-600 hover:text-blue-500">
          <div class="text-sm">
            <div class="flex items-center gap-1 mb-1">
              <span class="text-xs">{{ t('components.commentItem.parentComment') }}</span>
              #{{ processedComment.comment_num - 1 }}
            </div>
            <div class="italic prose prose-sm max-w-none [&_img]:max-h-48 [&_img]:object-contain">
              <template v-for="(part, index) in processedComment.parent_content" :key="index">
                <div v-if="['text', 'header'].includes(part.type)">
                  <LazyMathJax :content="part.data" :enable-markdown="true" />
                </div>
              </template>
            </div>
          </div>
        </RouterLink>
      </div>
    </div>

    <button v-if="!flatStyle && processedComment.parent_id"
      class="text-gray-500 italic hover:text-blue-600 flex items-center text-xs mb-2"
      @click.stop="showParentInThread = !showParentInThread">
      <ArrowUp class="h-3 w-3" />
      <span>{{ t('components.commentItem.showParent') }}</span>
    </button>

    <div class="min-w-48 overflow-x-auto">
      <div class="prose prose-sm max-w-none text-gray-700 mb-3 [&_img]:max-h-48 [&_img]:object-contain">
        <template v-for="(part, index) in processedComment.plain_content" :key="index">
          <LazyMathJax :content="part.data" :enable-markdown="true" />
        </template>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex flex-wrap gap-2 justify-end">
      <div class="flex gap-2 items-center">
        <div v-if="reactions.length" class="flex flex-wrap gap-1">
          <button v-for="reaction in reactions" :key="reaction.reaction"
            class="gap-1.5 btn-reaction transition-all duration-300" :class="[
              reaction.reacted &&
              'enabled',
            ]" @click.stop="handleReactionClick(reaction.reaction)">
            <span class="inline-block text-base"
              :class="{ 'animate-emoji-rotate': reaction.reacted && reaction.isNew }">
              {{ reaction.reaction }}
            </span>
            <span v-if="reaction.count > 0" class="text-sm font-bold">
              {{ reaction.count }}
            </span>
          </button>
        </div>

        <div v-if="auth.state.isLoggedIn" class="relative">
          <button class="ml-3 btn-empty" @click.stop="showReactionPicker = !showReactionPicker">
            <ReactionPlusIcon />
            <span class="sr-only">{{ t('components.commentItem.addReaction') }}</span>
          </button>

          <Teleport to="body">
            <div v-if="showReactionPicker"
              class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
              @click="showReactionPicker = false">
              <div class="bg-white rounded-lg shadow-lg max-w-sm w-full m-4 p-2" @click.stop>
                <div class="mb-2 px-1">
                  <input v-model="customEmoji" :placeholder="t('components.commentItem.customEmojiPlaceholder')"
                    class="w-full px-3 py-2 border rounded text-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 focus:outline-none"
                    maxlength="2" @keydown.enter="addCustomEmoji" @keydown.stop @click.stop>
                </div>
                <div class="grid grid-cols-5 auto-rows-fr gap-0">
                  <button v-for="emoji in emojiList" :key="emoji.symbol"
                    class="group p-1 hover:bg-gray-100 rounded transition-colors"
                    :class="{ 'bg-blue-100': isReactionSelected(emoji.symbol) }" @click="addReaction(emoji.symbol)">
                    <div class="flex flex-col items-center justify-between">
                      <span class="text-base group-hover:scale-110 transition-transform mb-1">{{
                        emoji.symbol
                      }}</span>
                      <span class="text-xs text-gray-600 text-center break-words" :lang="locale" style="hyphens: auto;">{{ t(`components.commentItem.reactions.${emoji.meaning.toLowerCase().replace(/\s+/g, '')}`) }}</span>
                    </div>
                  </button>
                </div>
              </div>
            </div>
          </Teleport>
        </div>
      </div>

      <button v-if="auth.state.isLoggedIn" :disabled="isProcessing" class="gap-1.5 btn-empty" :class="[
        processedComment.is_bookmarked
          ? 'bg-blue-50 text-blue-600 hover:bg-blue-100 border border-blue-200'
          : '',
      ]" @click.stop="handleBookmarkClick">
        <div class="flex items-center">
          <BookmarkCheck v-if="processedComment.is_bookmarked" class="h-5 w-5"
            :class="{ 'animate-bounce-once': isBookmarkAnimating }" />
          <Bookmark v-else class="h-5 w-5" :class="{ 'animate-bounce-once': isBookmarkAnimating }" />
          <span class="hidden sm:inline ml-1">{{ processedComment.is_bookmarked ? t('components.commentItem.saved') : t('components.commentItem.save') }}</span>
        </div>
      </button>

      <button
        v-if="auth.state.isLoggedIn && auth.state.username === processedComment.username && (processedComment.total_replies ?? 0) === 0"
        class="inline-flex items-center btn-empty text-red-600 hover:text-red-800" :disabled="isProcessing"
        @click="handleDeleteClick">
        <Trash2 class="h-4 w-4" />
        <span class="sr-only">{{ t('components.commentItem.delete') }}</span>
      </button>

      <button v-if="auth.state.isLoggedIn && replyEnabled" class="inline-flex items-center btn-reply"
        @click="handleReplyClick">
        <Reply class="w-5 h-5" />
        <span v-if="processedComment.total_replies > 0" class="text-xs font-bold">{{
          processedComment.total_replies
        }}</span>
        <span>{{ t('components.commentItem.reply') }}</span>
      </button>
    </div>
  </div>
  <ToastFloat :show="showToast" :message="toastMessage" :type="toastType" />
  <DeleteConfirmationModal :show="showDeleteConfirm" :title="t('components.commentItem.deleteConfirmTitle')"
    :message="t('components.commentItem.deleteConfirmMessage')" :is-deleting="isDeleting"
    @confirm="performDeleteComment" @cancel="showDeleteConfirm = false" />
</template>

<script setup>
import { Bookmark, BookmarkCheck, Reply, User, Trash2, ArrowUp } from 'lucide-vue-next';
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';

import { toggleBookmark, toggleReaction, deleteComment } from '@/api';
import DeleteConfirmationModal from '@/components/DeleteConfirmation.vue';
import ReactionPlusIcon from '@/components/icons/ReactionPlusIcon.vue';
import LazyMathJax from '@/components/LazyMathJax.vue';
import ToastFloat from '@/components/ToastFloat.vue';
import { useAvatarStore } from '@/composables/avatarStore';
import { useAuth } from '@/composables/useAuth';

const { t, locale } = useI18n();

const props = defineProps({
  comment: {
    type: Object,
    required: true,
  },
  flatStyle: {
    type: Boolean,
    default: false,
  },
  level: {
    type: Number,
    default: 0,
  },
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
  replyEnabled: {
    type: Boolean,
    default: false,
  },
  showContext: {
    type: Boolean,
    default: false,
  },
})

const auth = useAuth()
const router = useRouter()
const isProcessing = ref(false)
const processedComment = computed(() => ({
  ...props.comment,
  plain_content: props.comment.content.filter(part => part.type === 'text'),
  subject: props.comment.content.find(part => part.type === 'header')?.data
}))
const isBookmarkAnimating = ref(false)
const showReactionPicker = ref(false)
const showToast = ref(false)
const toastMessage = ref('')
const toastType = ref('error')
const showDeleteConfirm = ref(false)
const isDeleting = ref(false)
const showParentInThread = ref(false)

const emojiList = [
  { symbol: '👍', meaning: 'Approve' },
  { symbol: '👎', meaning: 'Disapprove' },
  { symbol: '❤️', meaning: 'Love this' },
  { symbol: '😄', meaning: 'Happy' },
  { symbol: '⚠️', meaning: 'Warning' },
  { symbol: '😂', meaning: 'Funny' },
  { symbol: '🤔', meaning: 'Thinking' },
  { symbol: '😮', meaning: 'Surprised' },
  { symbol: '🙁', meaning: 'Sad' },
  { symbol: '🤦', meaning: 'Facepalm' },
  { symbol: '🙏', meaning: 'Thanks' },
  { symbol: '❓', meaning: 'Question' },
  { symbol: '✅', meaning: 'Correct' },
  { symbol: '🔥', meaning: 'Trending' },
  { symbol: '⊨', meaning: 'Therefore' },
  { symbol: '⊥', meaning: 'Contradiction' },
  { symbol: '∀', meaning: 'For all' },
  { symbol: '∃', meaning: 'Exists' },
  { symbol: '⊕', meaning: 'XOR' },
  { symbol: '∨', meaning: 'AND/OR' },
  { symbol: '¬', meaning: 'NOT' },
  { symbol: '≡', meaning: 'Equivalent' },
  { symbol: '≠', meaning: 'Not equal' },
  { symbol: '⟹', meaning: 'Implies' },
  { symbol: '⊢', meaning: 'Proves' },
];

const customEmoji = ref('');
const addCustomEmoji = () => {
  if (customEmoji.value) {
    addReaction(customEmoji.value)
    customEmoji.value = ''
  }
}

const reactions = ref(
  [...props.comment.reactions]
    .map((r) => ({
      ...r,
      isNew: false,
    }))
    .sort((a, b) => b.count - a.count)
)

const isReactionSelected = (reaction) => {
  return reactions.value.some((r) => r.reaction === reaction && r.reacted)
}

const handleReactionClick = async (reaction) => {
  if (!auth.state.isLoggedIn) return

  const reactionIndex = reactions.value.findIndex((r) => r.reaction === reaction)
  const isExistingReaction = reactionIndex !== -1
  const wasReacted = isExistingReaction && reactions.value[reactionIndex].reacted

  // Optimistic update
  if (isExistingReaction) {
    if (wasReacted) {
      // Remove reaction
      if (reactions.value[reactionIndex].count > 1) {
        reactions.value[reactionIndex].count--
        reactions.value[reactionIndex].reacted = false
      } else {
        reactions.value.splice(reactionIndex, 1)
      }
    } else {
      // Add to existing reaction
      reactions.value[reactionIndex].count++
      reactions.value[reactionIndex].reacted = true
      reactions.value[reactionIndex].isNew = true
      setTimeout(() => {
        reactions.value[reactionIndex].isNew = false
      }, 1000)
    }
  } else {
    // Add new reaction
    reactions.value.push({
      reaction,
      count: 1,
      reacted: true,
      isNew: true,
    })
    setTimeout(() => {
      const newIndex = reactions.value.findIndex((r) => r.reaction === reaction)
      if (newIndex !== -1) {
        reactions.value[newIndex].isNew = false
      }
    }, 1000)
  }

  try {
    const response = await toggleReaction(props.comment.comment_id, reaction)

    // If API call failed, revert optimistic update
    if (response.status !== 200) {
      if (isExistingReaction) {
        if (wasReacted) {
          // Re-add removed reaction
          if (reactionIndex !== -1) {
            reactions.value[reactionIndex].count++
            reactions.value[reactionIndex].reacted = true
          }
        } else {
          // Remove added reaction
          if (reactions.value[reactionIndex].count > 1) {
            reactions.value[reactionIndex].count--
            reactions.value[reactionIndex].reacted = false
          } else {
            reactions.value.splice(reactionIndex, 1)
          }
        }
      } else {
        // Remove newly added reaction
        const newIndex = reactions.value.findIndex((r) => r.reaction === reaction)
        if (newIndex !== -1) {
          reactions.value.splice(newIndex, 1)
        }
      }
    }
  } catch (error) {
    if (
      error.response?.status === 400 &&
      error.response?.data?.details.includes('reactions per comment reached') >= 0
    ) {
      toastMessage.value = error.response?.data?.details;
      showToast.value = true
      setTimeout(() => {
        showToast.value = false
      }, 5800)

      // Revert optimistic update for maximum reactions case
      if (isExistingReaction) {
        if (wasReacted) {
          // Re-add removed reaction
          if (reactionIndex !== -1) {
            reactions.value[reactionIndex].count++
            reactions.value[reactionIndex].reacted = true
          }
        } else {
          // Remove added reaction
          if (reactions.value[reactionIndex].count > 1) {
            reactions.value[reactionIndex].count--
            reactions.value[reactionIndex].reacted = false
          } else {
            reactions.value.splice(reactionIndex, 1)
          }
        }
      } else {
        // Remove newly added reaction
        const newIndex = reactions.value.findIndex((r) => r.reaction === reaction)
        if (newIndex !== -1) {
          reactions.value.splice(newIndex, 1)
        }
      }
    }
    console.error('Error toggling reaction:', error)
  }
}

const addReaction = async (reaction) => {
  await handleReactionClick(reaction)
  showReactionPicker.value = false
}

const avatarStore = useAvatarStore()
const hasProfileImage = ref(false)
const isProfileImageLoading = ref(false)

const getProfileImageUrl = avatarStore.getProfileImageUrl

const checkProfileImage = async () => {
  isProfileImageLoading.value = true
  hasProfileImage.value = await avatarStore.checkProfileImage(processedComment.value.username)
}

const handleImageLoad = () => {
  isProfileImageLoading.value = false
}

const handleKeyDown = (e) => {
  if (e.key === 'Escape') {
    showReactionPicker.value = false
  }
}

const handleOutsideClick = (e) => {
  if (!e.target.closest('.reaction-picker')) {
    showReactionPicker.value = false
  }
}

const emit = defineEmits(['reply', 'focus-textarea', 'deleted'])

onMounted(() => {
  checkProfileImage()
  window.addEventListener('keydown', handleKeyDown)
  document.addEventListener('click', handleOutsideClick)
})

const handleReplyClick = () => {
  emit('reply', processedComment.value.comment_id)
  // Emit focus event after a short delay to allow form to render
  setTimeout(() => {
    emit('focus-textarea')
  }, 50)
}

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
  document.removeEventListener('click', handleOutsideClick)
})

const handleDeleteClick = () => {
  showDeleteConfirm.value = true
}

const performDeleteComment = async () => {
  if (isProcessing.value) return

  isDeleting.value = true
  isProcessing.value = true
  try {
    const response = await deleteComment(processedComment.value.comment_id)
    if (response.status === 200) {
      showToast.value = true;
      toastMessage.value = t('components.commentItem.deleteSuccess');
      toastType.value = 'success';
      setTimeout(() => {
        showToast.value = false;
        // Redirect to parent comment or thread
        if (processedComment.value.parent_id) {
          router.push({
            path: '/comments',
            query: {
              comment_id: processedComment.value.parent_id,
              valsi_id: props.valsiId || 0,
              definition_id: props.definitionId || 0,
            },
          })
        } else {
          router.push({
            path: '/comments',
            query: {
              thread_id: processedComment.value.thread_id,
              valsi_id: props.valsiId || 0,
              definition_id: props.definitionId || 0,
            },
          })
        }
      }, 1000);
    } else {
      showToast.value = true;
      toastMessage.value = response.data?.error || t('components.commentItem.deleteFailed');
      toastType.value = 'error';
      setTimeout(() => {
        showToast.value = false;
      }, 3000);
    }
  } catch (error) {
    showToast.value = true;
    toastMessage.value = error.response?.data?.error || t('components.commentItem.deleteFailed');
    toastType.value = 'error';
    setTimeout(() => {
      showToast.value = false;
    }, 3000);
  } finally {
    isDeleting.value = false;
    isProcessing.value = false
    showDeleteConfirm.value = false
  }
}

const handleImageError = (event) => {
  isProfileImageLoading.value = false
  event.target.style.display = 'none'
  const sibling = event.target.nextElementSibling
  if (sibling) {
    sibling.style.display = 'flex'
  }
}

const handleBookmarkClick = async (e) => {
  e.stopPropagation()
  if (isProcessing.value) return

  isProcessing.value = true
  try {
    await toggleBookmark(processedComment.value.comment_id, !processedComment.value.is_bookmarked)
    processedComment.value.is_bookmarked = !processedComment.value.is_bookmarked

    isBookmarkAnimating.value = true
    setTimeout(() => {
      isBookmarkAnimating.value = false
    }, 1000)
  } catch (error) {
    console.error('Error toggling bookmark:', error)
  } finally {
    isProcessing.value = false
  }
}

const formatDate = (timestamp) => {
  return new Date(timestamp * 1000).toLocaleString(locale.value, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<style scoped>
.comment-item {
  transform-style: preserve-3d;
}

.comment-item img.profile-image {
  backface-visibility: hidden;
  transform: translateZ(0);
}

.comment-item:hover img.profile-image {
  border-color: rgb(147, 197, 253);
}

@keyframes profileImagePulse {
  0% {
    background-color: rgb(229, 231, 235);
  }

  50% {
    background-color: rgb(209, 213, 219);
  }

  100% {
    background-color: rgb(229, 231, 235);
  }
}

.profile-image-placeholder {
  animation: profileImagePulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.reaction-picker {
  max-height: 300px;
  overflow-y: auto;
  min-width: 16rem;
}

.highlight-comment {
  animation: highlight 5.8s ease-out;
}

@keyframes highlight {

  0%,
  95% {
    @apply outline outline-orange-600 outline-2 bg-orange-50 border-orange-600;
  }

  100% {
    background-color: transparent;
    box-shadow: none;
  }
}

.grid button {
  min-height: 70px;
}

.grid button:hover span {
  transform: scale(1.1);
  transition: transform 0.2s ease;
}

@keyframes fade-in-up {
  from {
    opacity: 0;
    transform: translateY(1rem);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fade-in-up {
  animation: fade-in-up 0.3s ease-out;
}

@keyframes reaction-add {
  0% {
    transform: scale(1);
  }

  50% {
    transform: scale(1.5);
  }

  100% {
    transform: scale(1);
  }
}

.animate-reaction-add {
  animation: reaction-add 1.5s ease-out;
}

@keyframes emoji-rotate {
  0% {
    transform: perspective(400px) rotateY(0);
  }

  50% {
    transform: perspective(400px) rotateY(180deg) scale(2.5);
  }

  100% {
    transform: perspective(400px) rotateY(360deg);
  }
}

.animate-emoji-rotate {
  animation: emoji-rotate 1.2s ease-in-out;
  transform-style: preserve-3d;
  backface-visibility: visible;
}

.highlight-new-comment {
  animation: highlight-pulse 2s ease-in-out;
}

@keyframes highlight-pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(59, 130, 246, 0.4);
  }

  70% {
    box-shadow: 0 0 0 10px rgba(59, 130, 246, 0);
  }

  100% {
    box-shadow: 0 0 0 0 rgba(59, 130, 246, 0);
  }
}
</style>
