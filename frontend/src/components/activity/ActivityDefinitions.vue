<template>
  <div class="space-y-4">
    <div v-if="definitions.length === 0" class="text-center py-8 bg-gray-50 rounded-lg">
      <Book class="mx-auto h-12 w-12 text-blue-400" />
      <p class="text-gray-600">{{ emptyMessage }}</p>
    </div>
    <DictionaryEntries
      v-else
      :definitions="enrichedDefinitions"
      :languages="languages"
      :show-vote-buttons="auth.state.isLoggedIn"
      :collections="collections"
      @collection-updated="collections = $event"
    />
  </div>
</template>

<script setup lang="ts">
import { Book } from 'lucide-vue-next'
import { computed, ref, watch, onMounted, type PropType } from 'vue'
import { useI18n } from 'vue-i18n'

import { getBulkVotes, getCollections, getLanguages } from '@/api'
import DictionaryEntries from '@/components/DictionaryEntries.vue'
import { useAuth } from '@/composables/useAuth'

const { t } = useI18n()
const auth = useAuth()

type ActivityDefinition = {
  definitionid: number
  definition?: string
  content?: string
  word?: string
  valsiword?: string
  score?: number
  user_vote?: number | null
} & Record<string, unknown>

const props = defineProps({
  definitions: {
    type: Array as PropType<ActivityDefinition[]>,
    required: true,
  },
  formatDate: {
    type: Function,
    required: true,
  },
  noItemsMessage: {
    type: String,
    default: '',
  },
})

const languages = ref<Array<{ id: number; real_name: string; english_name?: string }>>([])
const collections = ref([])
const userVotes = ref<Record<string, number | null>>({})

const emptyMessage = computed(() => props.noItemsMessage || t('activity.noDefinitions'))

function normalizeDefinition(def: ActivityDefinition) {
  const definitionText =
    typeof def.definition === 'string'
      ? def.definition
      : typeof def.content === 'string'
        ? def.content
        : ''
  const word = (def.valsiword ?? def.word ?? '') as string
  return {
    ...def,
    definition: definitionText,
    valsiword: word,
    word,
    score: typeof def.score === 'number' ? def.score : 0,
    user_vote: userVotes.value[String(def.definitionid)] ?? def.user_vote ?? null,
  }
}

const enrichedDefinitions = computed(() => props.definitions.map(normalizeDefinition))

async function loadUserVotes(defs: ActivityDefinition[]) {
  if (!auth.state.isLoggedIn || defs.length === 0) {
    userVotes.value = {}
    return
  }

  const definitionIds = defs.map((d) => d.definitionid).filter((id) => typeof id === 'number')
  if (definitionIds.length === 0) {
    userVotes.value = {}
    return
  }

  try {
    const votesResponse = await getBulkVotes({ definition_ids: definitionIds })
    userVotes.value = votesResponse.data.votes || {}
  } catch (error) {
    console.error('Error fetching votes:', error)
    userVotes.value = {}
  }
}

watch(
  () => props.definitions,
  (defs) => {
    loadUserVotes(defs)
  },
  { immediate: true, deep: true }
)

onMounted(async () => {
  try {
    const [languagesResponse, collectionsResponse] = await Promise.all([
      getLanguages(),
      auth.state.isLoggedIn ? getCollections().catch(() => null) : Promise.resolve(null),
    ])
    languages.value = languagesResponse.data
    if (collectionsResponse?.data?.collections) {
      collections.value = collectionsResponse.data.collections
    }
  } catch (error) {
    console.error('Error fetching languages:', error)
  }
})
</script>
