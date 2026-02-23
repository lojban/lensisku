<template>
  <div class="space-y-4">
    <div
      v-if="votes.length === 0"
      class="text-center py-8 bg-gray-50 rounded-lg"
    >
      <Vote class="mx-auto h-12 w-12 text-blue-400" />
      <p class="text-gray-600">
        {{ t('components.activityVotes.noVotes') }}
      </p>
    </div>
    <DefinitionCard
      v-for="vote in votes"
      v-else
      :key="`${vote.definition_id}-${vote.voted_at}`"
      :definition="voteToDefinition(vote)"
      :languages="languages"
      :disable-toolbar="true"
      :disable-discussion-button="false"
      :show-word-type="true"
      :show-audio="true"
    />
  </div>
</template>

<script setup>
import { Vote } from 'lucide-vue-next'
import { ref, onMounted } from 'vue'

import { getLanguages } from '@/api'
import DefinitionCard from '@/components/DefinitionCard.vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

defineProps({
  votes: {
    type: Array,
    required: true
  },
  formatDate: {
    type: Function,
    required: true
  }
})

const languages = ref([])

function voteToDefinition(vote) {
  return {
    definitionid: vote.definition_id,
    valsiword: vote.valsi_word,
    word: vote.valsi_word,
    definition: vote.definition,
    langid: vote.langid,
    score: vote.score ?? 0,
    user_vote: vote.vote_value,
    created_at: vote.voted_at
  }
}

onMounted(async () => {
  try {
    const response = await getLanguages()
    languages.value = response.data
  } catch (error) {
    console.error('Error fetching languages:', error)
  }
})
</script>
