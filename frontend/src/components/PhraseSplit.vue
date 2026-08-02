<template>
  <div v-if="tokens.length > 1" class="mb-4">
    <AlertComponent v-if="!expanded" type="tip" :label="t('components.phraseSplit.label')">
      <button type="button" class="text-sm text-nav-link hover:underline" @click="load">
        {{ t('components.phraseSplit.splitPhrase') }}
      </button>
    </AlertComponent>
    <div v-else class="space-y-4">
      <div v-for="(group, index) in groups" :key="group.word + index" class="space-y-2">
        <h3 class="text-base font-semibold text-gray-800">{{ group.word }}</h3>
        <div v-if="group.loading" class="text-sm text-gray-500">
          {{ t('components.phraseSplit.loading') }}
        </div>
        <div v-else-if="group.definitions.length === 0" class="text-sm text-gray-500">
          {{ t('components.phraseSplit.noDefinitions') }}
        </div>
        <DefinitionCardSimple
          v-for="def in group.definitions"
          v-else
          :key="def.definitionid"
          :definition="def"
          :languages="cardLanguages"
          show-word-type
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { fastSearchDefinitions } from '@/api'
import AlertComponent from '@/components/AlertComponent.vue'
import DefinitionCardSimple from '@/components/DefinitionCardSimple.vue'

const { t } = useI18n()

interface PhraseDefinition {
  definitionid: number
  valsiword?: string
  word?: string
}

interface PhraseGroup {
  word: string
  loading: boolean
  definitions: PhraseDefinition[]
}

type CardLanguage = {
  id?: number
  langid?: number
  real_name?: string
  realname?: string
  lojbanname?: string
  lojban_name?: string
}

const props = defineProps<{
  phrase: string
  selectedLanguages?: number[]
  sourceLangId?: number | null
  languages: unknown[]
}>()

const tokens = computed(() => {
  const trimmed = (props.phrase || '').trim()
  if (!trimmed || !/[.\s]/.test(trimmed)) return []
  return trimmed.split(/[.\s]+/).filter(Boolean)
})

const expanded = ref(false)
const groups = ref<PhraseGroup[]>([])

const cardLanguages = computed(() => props.languages as unknown as CardLanguage[])

watch(
  () => props.phrase,
  () => {
    expanded.value = false
    groups.value = []
  }
)

async function load() {
  expanded.value = true
  const words = tokens.value
  groups.value = words.map((word) => ({ word, loading: true, definitions: [] }))

  for (let i = 0; i < words.length; i++) {
    const word = words[i]
    const params: Record<string, unknown> = { search: word, per_page: 100, page: 1 }
    if (props.selectedLanguages?.length) {
      params.languages = props.selectedLanguages.join(',')
    }
    if (props.sourceLangId && props.sourceLangId !== 1) {
      params.source_langid = props.sourceLangId
    }

    try {
      const response = await fastSearchDefinitions(params)
      const all = (response.data?.definitions || []) as PhraseDefinition[]
      const exact = all.filter(
        (d) => (d.valsiword ?? d.word ?? '').toLowerCase() === word.toLowerCase()
      )
      groups.value[i].definitions = exact.length ? exact : all.slice(0, 10)
    } catch (e) {
      console.error('Failed to load split definitions for', word, e)
    } finally {
      groups.value[i].loading = false
    }
  }
}
</script>
