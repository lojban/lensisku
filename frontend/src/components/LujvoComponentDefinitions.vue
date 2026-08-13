<template>
  <details v-if="visible" open class="fold-panel">
    <summary class="fold-panel-summary">
      {{ decomposition.join(' + ') }}
    </summary>
    <div class="fold-panel-body">
      <div v-for="(group, index) in groupsWithDefinitions" :key="group.word + index" class="space-y-2">
        <DefinitionCardSimple
          v-for="def in group.definitions"
          :key="def.definitionid"
          :definition="def"
          :languages="cardLanguages"
          show-word-type
        />
      </div>
    </div>
  </details>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { fastSearchDefinitions } from '@/api'
import DefinitionCardSimple from '@/components/DefinitionCardSimple.vue'

interface ComponentDefinition {
  definitionid: number
  valsiword?: string
  word?: string
  langid?: number
}

interface ComponentGroup {
  word: string
  definitions: ComponentDefinition[]
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
  decomposition: string[]
  /** Definition language selected on the add/edit form. */
  langId?: string | number | null
  languages: unknown[]
}>()

const groups = ref<ComponentGroup[]>([])
const loaded = ref(false)
const loadToken = ref(0)

const cardLanguages = computed(() => props.languages as unknown as CardLanguage[])

const normalizedLangId = computed(() => {
  if (props.langId === null || props.langId === undefined || props.langId === '') return null
  const n = Number(props.langId)
  return Number.isFinite(n) && n > 0 ? n : null
})

const groupsWithDefinitions = computed(() =>
  groups.value.filter((group) => group.definitions.length > 0)
)

const visible = computed(
  () =>
    Boolean(normalizedLangId.value) &&
    props.decomposition.length > 0 &&
    loaded.value &&
    groupsWithDefinitions.value.length > 0
)

async function loadComponentDefinitions() {
  const words = props.decomposition.filter(Boolean)
  const lang = normalizedLangId.value
  loaded.value = false
  groups.value = []

  if (!words.length || !lang) {
    return
  }

  const token = ++loadToken.value
  const nextGroups: ComponentGroup[] = words.map((word) => ({ word, definitions: [] }))

  await Promise.all(
    words.map(async (word, i) => {
      try {
        const response = await fastSearchDefinitions({
          search: word,
          per_page: 50,
          page: 1,
          languages: String(lang),
        })
        if (token !== loadToken.value) return
        const all = (response.data?.definitions || []) as ComponentDefinition[]
        nextGroups[i].definitions = all.filter(
          (d) => (d.valsiword ?? d.word ?? '').toLowerCase() === word.toLowerCase()
        )
      } catch (e) {
        console.error('Failed to load lujvo component definitions for', word, e)
      }
    })
  )

  if (token !== loadToken.value) return
  groups.value = nextGroups
  loaded.value = true
}

watch(
  () => [props.decomposition.join('\0'), normalizedLangId.value] as const,
  () => {
    void loadComponentDefinitions()
  },
  { immediate: true }
)
</script>
