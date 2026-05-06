<template>
  <div class="flex items-center space-x-1">
    <!-- Upvote button -->
    <button
      :disabled="!hasVotePermission || isLoading || userVote === 1"
      :class="[
        'p-0 rounded-md transition-colors text-gray-600',
        userVote === 1
          ? 'text-green-600 bg-green-200'
          : hasVotePermission && 'hover:text-green-600 hover:bg-green-200',
        isLoading
          ? 'opacity-50 cursor-not-allowed'
          : !hasVotePermission
            ? 'cursor-not-allowed'
            : 'cursor-pointer',
      ]"
      :title="t('components.voteButtons.upvoteTitle')"
      @click="handleVote(false)"
    >
      <ThumbsUp class="w-5 h-5 m-1" :stroke-width="1.3" />
    </button>
    <!-- Score display -->
    <span class="text-sm"> {{ score }} </span>
    <!-- Downvote button -->
    <button
      :disabled="!hasVotePermission || isLoading || userVote === -1"
      :class="[
        'p-0 rounded-md transition-colors text-gray-600',
        userVote === -1
          ? 'text-red-600 bg-red-200'
          : hasVotePermission && 'hover:text-red-600 hover:bg-red-200',
        isLoading
          ? 'opacity-50 cursor-not-allowed'
          : !hasVotePermission
            ? 'cursor-not-allowed'
            : 'cursor-pointer',
      ]"
      :title="t('components.voteButtons.downvoteTitle')"
      @click="handleVote(true)"
    >
      <ThumbsDown class="w-5 h-5 m-1" :stroke-width="1.3" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { ThumbsUp, ThumbsDown } from 'lucide-vue-next'
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { voteDefinition } from '@/api'
import { useAuth } from '@/composables/useAuth'

const { t } = useI18n()
const router = useRouter()

const props = defineProps({
  definitionId: {
    type: Number,
    required: true,
  },
  initialScore: {
    type: Number,
    default: 0,
  },
  initialUserVote: {
    type: Number,
    default: null,
  },
})

defineEmits(['vote-change'])

const auth = useAuth()
const score = ref(props.initialScore)
const userVote = ref(props.initialUserVote)
const isLoading = ref(false)
const hasVotePermission = computed(() => (auth.state.authorities || []).includes('vote_definition'))

const handleVote = async (downvote = false) => {
  if (!auth.state.isLoggedIn) {
    router.push('/login')
    return
  }

  const newVote = downvote ? -1 : 1
  // If clicking same vote type, set to 0 (cancel vote)
  const shouldCancelVote =
    (downvote && userVote.value === 1) || (!downvote && userVote.value === -1)
  const finalVote = shouldCancelVote ? 0 : newVote

  const oldVote = userVote.value || 0
  const voteChange = finalVote - oldVote
  const oldScore = score.value

  userVote.value = shouldCancelVote ? null : finalVote
  score.value += voteChange

  try {
    isLoading.value = true
    const response = await voteDefinition(props.definitionId, downvote)

    if (!response.data.success) {
      userVote.value = oldVote
      score.value = oldScore
    } else {
      score.value = response.data.score
    }
  } catch (error) {
    userVote.value = oldVote
    score.value = oldScore
    console.error('Error voting:', error)
  } finally {
    isLoading.value = false
  }
}
</script>
