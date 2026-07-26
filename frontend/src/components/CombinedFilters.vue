<template>
  <div class="filters space-y-4">
    <!-- Language row: top by default; on semantic graph page (`languagesInExpandedPanel`) it lives inside the expanded panel. -->

    <div class="filters-bar-row" :class="{ 'sm:!justify-end': languagesInExpandedPanel }">
      <MultiSelectDropdown
        v-if="!languagesInExpandedPanel"
        v-model="selectedLangs"
        :options="languages"
        :max-selected-labels="3"
        :option-value="(lang: LanguageOption) => lang.id"
        :option-label="(lang: LanguageOption) => lang.real_name"
        :search-field-keys="languageFilterSearchFieldKeys"
        :placeholder="t('filters.selectLanguages')"
        :search-placeholder="t('filters.searchLanguages')"
        :select-all-label="t('filters.selectAll')"
        :empty-filter-label="t('filters.noMatches')"
        class="w-full sm:w-80"
      />
      <div class="flex items-center gap-2 self-end md:self-center">
        <div class="btn-group-forced flex items-center flex-row" role="group">
          <Button
            type="button"
            variant="toolbar"
            class="ui-btn--group-item"
            :title="t('searchForm.modes.semantic')"
            @click="toggleSemanticSearch"
          >
            <template #icon>
              <Checkbox
                class="checkmark-aqua pointer-events-none shrink-0"
                :model-value="isSemantic"
                tabindex="-1"
                aria-hidden="true"
              />
            </template>
            <span class="text-xs select-none whitespace-nowrap">{{
              t('searchForm.modes.semantic')
            }}</span>
          </Button>
          <Button
            type="button"
            variant="toolbar"
            class="ui-btn--group-item"
            :disabled="!!filters.word_type"
            :title="t('searchForm.modes.searchInPhrases')"
            @click="toggleSearchInPhrases"
          >
            <template #icon>
              <Checkbox
                class="checkmark-aqua pointer-events-none shrink-0"
                :model-value="searchInPhrases && !filters.word_type"
                :disabled="!!filters.word_type"
                tabindex="-1"
                aria-hidden="true"
              />
            </template>
            <span class="text-xs select-none whitespace-nowrap">
              {{ t('searchForm.modes.searchInPhrases') }}
            </span>
          </Button>
        </div>

        <div class="btn-group-forced flex items-center flex-row" role="group">
          <Button
            v-if="hasAnyActiveFilters"
            type="button"
            variant="toolbar"
            class="ui-btn--group-item"
            :title="t('filters.resetAllFilters')"
            @click="resetAllFilters"
          >
            <span class="text-xs">{{ t('filters.resetAllFilters') }}</span>
          </Button>
          <Button
            type="button"
            variant="toolbar"
            class="ui-btn--group-item"
            :title="expanded ? t('filters.collapse') : t('filters.expand')"
            @click="toggleExpanded"
          >
            <template #icon>
              <ChevronDown
                class="h-5 w-5 shrink-0 transition-transform duration-200"
                :class="{ 'rotate-180': expanded }"
                :stroke-width="2"
                aria-hidden="true"
              />
            </template>
          </Button>
        </div>
      </div>
    </div>

    <div
      v-show="expanded"
      class="mt-3 grid grid-cols-2 gap-x-3 gap-y-4 bg-white rounded-lg shadow-sm p-4"
      :class="{ 'animate-expandSection': expanded }"
    >
      <div v-if="languagesInExpandedPanel" class="col-span-2 flex min-w-0 flex-col">
        <label class="filters-field-label">{{ t('filters.selectLanguages') }}</label>
        <div class="relative w-full min-w-0 [&>div]:block [&>div]:w-full">
          <MultiSelectDropdown
            v-model="selectedLangs"
            :options="languages"
            :max-selected-labels="3"
            :option-value="(lang: LanguageOption) => lang.id"
            :option-label="(lang: LanguageOption) => lang.real_name"
            :search-field-keys="languageFilterSearchFieldKeys"
            :placeholder="t('filters.selectLanguages')"
            :search-placeholder="t('filters.searchLanguages')"
            :select-all-label="t('filters.selectAll')"
            :empty-filter-label="t('filters.noMatches')"
            class="w-full max-w-full sm:w-80"
          />
        </div>
      </div>

      <div v-if="showWordType" class="flex min-w-0 flex-col">
        <label class="filters-field-label">{{ t('filters.filterBy.wordType') }}</label>
        <div class="relative w-full [&>div]:block [&>div]:w-full">
          <ToolbarSelectDropdown
            id="cf-word-type"
            :aria-label="t('filters.filterBy.wordType')"
            trigger-class="w-full cursor-pointer"
            truncate-label
          >
            <template #label>
              {{ selectedWordTypeLabel }}
            </template>
            <ToolbarSelectDropdownItem
              v-for="type in wordTypeOptions"
              :key="type.type_id ?? 'all'"
              :class="{
                'bg-gray-100': filters.word_type
                  ? filters.word_type.type_id === type.type_id
                  : type.type_id === null,
              }"
              @click="selectWordType(type)"
            >
              {{ type.descriptor }}
            </ToolbarSelectDropdownItem>
          </ToolbarSelectDropdown>
        </div>
      </div>
      <!-- Unified input fields with clear buttons -->
      <div v-for="field in ['selmaho', 'username']" :key="field" class="flex min-w-0 flex-col">
        <label class="filters-field-label">
          {{
            field === 'selmaho'
              ? t('components.combinedFilters.filterBySelmao')
              : t('components.combinedFilters.filterByAuthor')
          }}
        </label>
        <div class="relative">
          <Input
            v-model="filters[field]"
            type="text"
            :placeholder="
              t(
                `components.combinedFilters.placeholder${field.charAt(0).toUpperCase() + field.slice(1)}`
              )
            "
            class="input-field w-full h-8"
            @input="debouncedFilterChange"
          />
          <IconButtonGhost
            v-if="filters[field]"
            class="absolute right-2 top-1/2 -translate-y-1/2"
            :aria-label="t('components.combinedFilters.clearFilter')"
            @click="clearFilter(field)"
          >
            <X class="h-5 w-5" />
          </IconButtonGhost>
        </div>
      </div>

      <div class="flex min-w-0 flex-col" :class="{ 'col-span-2': !showWordType }">
        <label class="filters-field-label">{{ t('filters.filterBy.sourceLanguage') }}</label>
        <div class="relative w-full [&>div]:block [&>div]:w-full">
          <ToolbarSelectDropdown
            id="cf-source-language"
            :aria-label="t('filters.filterBy.sourceLanguage')"
            trigger-class="w-full cursor-pointer"
            truncate-label
          >
            <template #label>
              {{ sourceLanguageLabel }}
            </template>
            <ToolbarSelectDropdownItem
              :class="{ 'bg-gray-100': filters.source_langid === 1 }"
              @click="selectSourceLang(1)"
            >
              {{ t('filters.defaultSourceLanguage') }}
            </ToolbarSelectDropdownItem>
            <ToolbarSelectDropdownItem
              v-for="lang in languages.filter((l) => l.id !== 1)"
              :key="lang.id"
              :class="{ 'bg-gray-100': filters.source_langid === lang.id }"
              @click="selectSourceLang(lang.id)"
            >
              {{ lang.real_name }}
            </ToolbarSelectDropdownItem>
          </ToolbarSelectDropdown>
        </div>
      </div>
    </div>

    <div
      v-if="graphBuildParams != null && !hideSemanticGraphMetrics"
      class="grid grid-cols-2 gap-3 rounded-lg bg-white p-3 shadow-sm sm:grid-cols-3 md:grid-cols-5"
    >
      <div class="flex min-w-0 flex-col">
        <label class="filters-field-label" for="cf-sg-min-vote">{{
          t('semanticGraph.minVote')
        }}</label>
        <Input
          id="cf-sg-min-vote"
          v-model.number="graphBuildParams.minVote"
          type="number"
          class="input-field h-8 w-full"
          min="-999"
          step="1"
        />
      </div>

      <div class="flex min-w-0 flex-col">
        <label class="filters-field-label" for="cf-sg-limit">{{
          t('semanticGraph.nodeLimit')
        }}</label>
        <Input
          id="cf-sg-limit"
          v-model.number="graphBuildParams.graphLimit"
          type="number"
          class="input-field h-8 w-full"
          min="1"
          :max="semanticGraphMaxNodes"
          step="1"
        />
      </div>

      <div class="flex min-w-0 flex-col">
        <label class="filters-field-label" for="cf-sg-k">{{ t('semanticGraph.kNeighbors') }}</label>
        <Input
          id="cf-sg-k"
          v-model.number="graphBuildParams.kNeighbors"
          type="number"
          class="input-field h-8 w-full"
          min="1"
          max="30"
          step="1"
        />
      </div>

      <div class="flex min-w-0 flex-col">
        <label class="filters-field-label" for="cf-sg-min-sim">{{
          t('semanticGraph.minPairwiseSim')
        }}</label>
        <Input
          id="cf-sg-min-sim"
          v-model.number="graphBuildParams.minPairwiseSim"
          type="number"
          class="input-field h-8 w-full"
          min="0"
          max="1"
          step="0.05"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ChevronDown, X } from 'lucide-vue-next'
import {
  Button,
  Checkbox,
  IconButtonGhost,
  Input,
  MultiSelectDropdown,
  ToolbarSelectDropdown,
  ToolbarSelectDropdownItem,
} from '@packages/ui'
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { useRoute } from 'vue-router'

import { fetchDefinitionsTypes } from '@/api'

import { defaultFilterLanguageTags } from '@/config/locales'
import { useI18n } from 'vue-i18n'
import type { PropType } from 'vue'
const { t } = useI18n()

export type SemanticGraphBuildParams = {
  minVote: number
  graphLimit: number
  kNeighbors: number
  minPairwiseSim: number
}

const graphBuildParams = defineModel<SemanticGraphBuildParams | undefined>('graphBuildParams', {
  required: false,
})

/** Fields used for language multiselect search (values only; case-insensitive substring). */
const languageFilterSearchFieldKeys: string[] = ['tag', 'english_name', 'lojban_name', 'real_name']

type LanguageOption = {
  id: number
  real_name: string
  english_name: string
  tag: string
  lojban_name?: string
}

type WordTypeOption = {
  type_id: number | null
  descriptor: string
}

const props = defineProps({
  modelValue: {
    type: Object,
    default: () => ({
      selmaho: '',
      username: '',
      isExpanded: false,
      selectedLanguages: [],
      word_type: null,
      source_langid: 1, // Default to Lojban
      isSemantic: true,
      searchInPhrases: true,
    }),
  },
  languages: {
    type: Array as PropType<LanguageOption[]>,
    required: true,
  },
  /** When true, language multiselect is only inside the expandable advanced panel (e.g. semantic graph page). */
  languagesInExpandedPanel: {
    type: Boolean,
    default: false,
  },
  /** Hide the semantic graph metrics grid (min vote, limit, …) when rendered elsewhere (e.g. below the graph). */
  hideSemanticGraphMetrics: {
    type: Boolean,
    default: false,
  },
  /** Upper bound for “max nodes” on the semantic graph page (server `SEMANTIC_GRAPH_MAX_LIMIT`). */
  semanticGraphMaxNodes: {
    type: Number,
    default: 120,
  },
})

const emit = defineEmits(['update:modelValue', 'change', 'reset'])

const selectedLangs = ref([])
const expanded = ref(props.modelValue.isExpanded)
const wordTypes = ref<WordTypeOption[]>([])
const route = useRoute()

function getInitialIsSemantic(): boolean {
  const mode = route.query.mode
  if (mode !== undefined) {
    return mode === 'semantic'
  }
  if (typeof window !== 'undefined') {
    const stored = localStorage.getItem('searchMode')
    if (stored !== null) {
      return stored === 'semantic'
    }
  }
  return props.modelValue.isSemantic !== false
}

function getInitialSearchInPhrases(): boolean {
  const urlVal = route.query.searchInPhrases
  if (urlVal !== undefined) {
    return urlVal !== 'false'
  }
  if (typeof window !== 'undefined') {
    const stored = localStorage.getItem('searchInPhrases')
    if (stored !== null) {
      return stored === 'true'
    }
  }
  return props.modelValue.searchInPhrases !== false
}

const isSemantic = ref<boolean>(getInitialIsSemantic())
const searchInPhrases = ref<boolean>(getInitialSearchInPhrases())

const filters = ref({
  selmaho: props.modelValue.selmaho,
  username: props.modelValue.username,
  word_type: null as WordTypeOption | null,
  source_langid: props.modelValue.source_langid || 1,
})

const showWordType = computed(() => !filters.value.selmaho)

const sourceLanguageLabel = computed(() => {
  if (filters.value.source_langid === 1) {
    return t('filters.defaultSourceLanguage')
  }
  const lang = props.languages.find((l) => l.id === filters.value.source_langid)
  return lang ? lang.real_name : t('filters.defaultSourceLanguage')
})

const selectedWordTypeLabel = computed(() => {
  return filters.value.word_type?.descriptor ?? t('filters.allWordTypes')
})

const wordTypeOptions = computed<WordTypeOption[]>(() => [
  { type_id: null, descriptor: t('filters.allWordTypes') },
  ...wordTypes.value,
])

const getLanguagesFromIds = (ids) => {
  return props.languages.filter((lang) => ids.includes(lang.id))
}

watch(
  () => props.modelValue,
  (newVal) => {
    expanded.value = newVal.isExpanded
    filters.value = {
      selmaho: newVal.selmaho,
      username: newVal.username,
      word_type: null,
      source_langid: newVal.source_langid || 1,
    }

    if (newVal.word_type && wordTypes.value.length > 0) {
      const selectedType = wordTypes.value.find((t) => t.type_id === newVal.word_type)
      if (selectedType) {
        filters.value.word_type = selectedType
      }
    }

    if (newVal.selectedLanguages?.length > 0) {
      selectedLangs.value = getLanguagesFromIds(newVal.selectedLanguages)
    }
  },
  { deep: true, immediate: true }
)

watch(
  () => ({
    isSemantic: props.modelValue.isSemantic,
    searchInPhrases: props.modelValue.searchInPhrases,
  }),
  (newVal) => {
    if (newVal.isSemantic !== undefined) {
      isSemantic.value = newVal.isSemantic !== false
    }
    if (newVal.searchInPhrases !== undefined && newVal.searchInPhrases !== null) {
      searchInPhrases.value = newVal.searchInPhrases !== false
    }
  }
)

const fetchWordTypes = async () => {
  try {
    const response = await fetchDefinitionsTypes()
    wordTypes.value = response.data.types
  } catch (error) {
    console.error('Error fetching word types:', error)
  }
}

onMounted(() => {
  fetchWordTypes()
  syncTogglesWithModel()
})

function syncTogglesWithModel() {
  const semanticChanged =
    props.modelValue.isSemantic !== undefined && props.modelValue.isSemantic !== isSemantic.value
  const searchInPhrasesChanged =
    props.modelValue.searchInPhrases !== undefined &&
    props.modelValue.searchInPhrases !== null &&
    props.modelValue.searchInPhrases !== searchInPhrases.value

  if (semanticChanged || searchInPhrasesChanged) {
    emitUpdate()
  }
}

onBeforeUnmount(() => {
  // Clean up any pending debounce timer
  clearDebounceTimer()
})

const getDefaultLanguages = () => {
  return props.languages.filter((lang) =>
    (defaultFilterLanguageTags as readonly string[]).includes(lang.tag)
  )
}

// Debounce delay: 450ms is optimal for search inputs (400-500ms range)
// This balances responsiveness with reducing unnecessary API calls
const DEBOUNCE_DELAY = 450

// Debounce timer
let debounceTimer = null

function clearDebounceTimer() {
  if (debounceTimer) {
    clearTimeout(debounceTimer)
    debounceTimer = null
  }
}

const hasAnyActiveFilters = computed(() => {
  return Boolean(
    selectedLangs.value.length > 0 ||
    filters.value.selmaho ||
    filters.value.username ||
    filters.value.word_type ||
    filters.value.source_langid !== 1 ||
    !isSemantic.value ||
    !searchInPhrases.value ||
    expanded.value
  )
})

const debouncedFilterChange = () => {
  // Clear any pending timeouts to prevent stale filter updates
  clearDebounceTimer()

  // Capture current filter values to check in timeout
  const currentFilters = {
    selmaho: filters.value.selmaho,
    username: filters.value.username,
  }

  // Debounce the filter change - only trigger after user stops typing
  // This prevents excessive API calls while user is actively typing
  debounceTimer = setTimeout(() => {
    // Only emit if filters haven't changed (to prevent race conditions)
    if (
      filters.value.selmaho === currentFilters.selmaho &&
      filters.value.username === currentFilters.username
    ) {
      emitUpdate()
    }
    debounceTimer = null
  }, DEBOUNCE_DELAY)
}

function selectWordType(type: WordTypeOption) {
  filters.value.word_type = type.type_id === null ? null : type
  emitUpdate()
}

function selectSourceLang(id: number) {
  filters.value.source_langid = id
  emitUpdate()
}

const emitUpdate = () => {
  const updatedValue = {
    selmaho: filters.value.selmaho,
    username: filters.value.username,
    isExpanded: expanded.value,
    selectedLanguages: selectedLangs.value.map((lang) => lang.id),
    word_type: filters.value.word_type?.type_id || null,
    source_langid: filters.value.source_langid || 1, // Include source_langid
    isSemantic: isSemantic.value,
    searchInPhrases: filters.value.word_type ? null : searchInPhrases.value,
  }
  emit('update:modelValue', updatedValue)
  emit('change', updatedValue)
}

const clearFilter = (filterName) => {
  // Clear any pending timeouts first to prevent them from firing after clearing
  clearDebounceTimer()
  filters.value[filterName] = ''
  emitUpdate()
}

const resetAllFilters = () => {
  const defaultLangs = getDefaultLanguages()
  selectedLangs.value = defaultLangs

  isSemantic.value = true
  searchInPhrases.value = true
  if (typeof window !== 'undefined') {
    localStorage.setItem('searchMode', 'semantic')
    localStorage.setItem('searchInPhrases', 'true')
  }

  const resetValue = {
    selmaho: '',
    username: '',
    isExpanded: false,
    selectedLanguages: defaultLangs.map((lang) => lang.id),
    word_type: null,
    source_langid: 1,
    isSemantic: true,
    searchInPhrases: true,
  }

  emit('reset')
  emit('update:modelValue', resetValue)
}

const toggleExpanded = () => {
  expanded.value = !expanded.value
  emitUpdate()
}

const toggleSearchInPhrases = () => {
  if (filters.value.word_type) return
  searchInPhrases.value = !searchInPhrases.value
  if (typeof window !== 'undefined') {
    localStorage.setItem('searchInPhrases', String(searchInPhrases.value))
  }
  emitUpdate()
}

const toggleSemanticSearch = () => {
  isSemantic.value = !isSemantic.value
  if (typeof window !== 'undefined') {
    localStorage.setItem('searchMode', isSemantic.value ? 'semantic' : 'dictionary')
  }
  emitUpdate()
}

watch(
  selectedLangs,
  (newLangs, oldLangs) => {
    // Only emit if the actual values changed, not just reference
    if (JSON.stringify(newLangs) !== JSON.stringify(oldLangs)) {
      emitUpdate()
    }
  },
  { deep: true }
)

watch(
  () => wordTypes.value,
  (newTypes) => {
    if (newTypes.length > 0 && props.modelValue.word_type) {
      const selectedType = newTypes.find((t) => t.type_id === props.modelValue.word_type)
      if (selectedType) {
        filters.value.word_type = selectedType
      }
    }
  },
  { immediate: true }
)

watch(
  () => filters.value.selmaho,
  (newVal) => {
    if (newVal) {
      filters.value.word_type = null
    }
  }
)

// Initialize selected languages
watch(
  () => props.languages,
  (newLanguages) => {
    if (newLanguages.length > 0) {
      if (props.modelValue.selectedLanguages?.length > 0) {
        selectedLangs.value = getLanguagesFromIds(props.modelValue.selectedLanguages)
      } else {
        // Set default languages but don't emit update to prevent double fetching
        selectedLangs.value = getDefaultLanguages()
        // Update the modelValue without emitting change event
        const updatedValue = {
          ...props.modelValue,
          selectedLanguages: selectedLangs.value.map((lang) => lang.id),
        }
        emit('update:modelValue', updatedValue)
      }
    }
  },
  { immediate: true }
)
</script>

<style scoped>
.animate-expandSection {
  animation: expandSection 0.2s ease-out;
}

@keyframes expandSection {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
