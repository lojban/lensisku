<template>
  <Dropdown>
    <template #trigger="{ open }">
      <button
        type="button"
        class="vote-trigger flex h-6 items-center gap-1 rounded-lg px-1.5 text-gray-600 transition-colors hover:bg-gray-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 disabled:cursor-not-allowed disabled:opacity-50"
        :class="{
          'vote-trigger--up': userVote === 1,
          'vote-trigger--down': userVote === -1,
          'bg-gray-100': open,
        }"
        :disabled="!hasVotePermission || isLoading"
        :aria-label="`${t('components.voteButtons.upvoteTitle')} / ${t('components.voteButtons.downvoteTitle')}: ${score}`"
        :title="`${t('components.voteButtons.upvoteTitle')} / ${t('components.voteButtons.downvoteTitle')}`"
        aria-haspopup="menu"
        :aria-expanded="open"
        :aria-busy="isLoading"
      >
        <span class="relative h-[21px] w-5 shrink-0" aria-hidden="true">
          <ThumbsDown
            class="absolute -bottom-px right-0 h-3.5 w-3.5 fill-white"
            :stroke-width="1.6"
          />
          <ThumbsUp class="vote-thumb-front absolute left-0 -top-px h-4 w-4" :stroke-width="1.6" />
        </span>
        <span class="min-w-[1ch] text-xs font-semibold tabular-nums" aria-live="polite">{{
          score
        }}</span>
      </button>
    </template>
    <div role="menu" class="min-w-36 p-1">
      <button
        v-for="option in voteOptions"
        :key="option.value"
        type="button"
        role="menuitemradio"
        :aria-checked="userVote === option.value"
        :disabled="!hasVotePermission || isLoading || userVote === option.value"
        class="flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-left text-sm text-gray-700 transition-colors hover:bg-gray-100 focus-visible:bg-gray-100 focus-visible:outline-none disabled:cursor-default"
        :class="{
          'bg-green-50 !text-green-700': userVote === 1 && option.value === 1,
          'bg-red-50 !text-red-700': userVote === -1 && option.value === -1,
        }"
        @click="handleVote(option.value === -1)"
      >
        <component :is="option.icon" class="h-4 w-4" :stroke-width="1.6" aria-hidden="true" />
        <span class="flex-1">{{ t(option.label) }}</span>
        <Check v-if="userVote === option.value" class="h-3.5 w-3.5" aria-hidden="true" />
      </button>
    </div>
  </Dropdown>
</template>

<script setup lang="ts">
import { Dropdown } from '@packages/ui'
import { ThumbsUp, ThumbsDown, Check } from '@lucide/vue'
import { ref, computed, watch } from 'vue'
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

const voteOptions = [
  { value: 1, icon: ThumbsUp, label: 'components.voteButtons.upvoteTitle' },
  { value: -1, icon: ThumbsDown, label: 'components.voteButtons.downvoteTitle' },
]

const auth = useAuth()
const score = ref(props.initialScore)
const userVote = ref(props.initialUserVote)
const isLoading = ref(false)
const hasVotePermission = computed(() => (auth.state.authorities || []).includes('vote_definition'))

watch(
  () => props.initialScore,
  (value) => {
    if (!isLoading.value) score.value = value
  }
)

watch(
  () => props.initialUserVote,
  (value) => {
    if (!isLoading.value) userVote.value = value
  }
)

const handleVote = async (downvote = false) => {
  if (!auth.state.isLoggedIn) {
    router.push('/login')
    return
  }

  const newVote = downvote ? -1 : 1
  if (!hasVotePermission.value || isLoading.value || userVote.value === newVote) return

  // An opposite vote cancels the existing vote, matching the API.
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

<style scoped>
.vote-thumb-front {
  fill: #fff;
}

.vote-trigger--up {
  color: #15803d;
}

.vote-trigger--down {
  color: #b91c1c;
}
</style>
