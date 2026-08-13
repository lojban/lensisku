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
                checkbox-class="pointer-events-none"
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
                checkbox-class="pointer-events-none"
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
            :class="{ 'filters-expand-attention': shouldPulseExpandChevron }"
            :title="expanded ? t('filters.collapse') : t('filters.expand')"
            @click="toggleExpanded"
          >
            <template #icon>
              <span
                class="inline-flex shrink-0"
                :class="{ 'filters-expand-chevron-pulse': shouldPulseExpandChevron }"
              >
                <ChevronDown
                  class="h-5 w-5 shrink-0 transition-transform duration-200"
                  :class="{ 'rotate-180': expanded }"
                  :stroke-width="2"
                  aria-hidden="true"
                />
              </span>
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
      <!-- Selma'o filter -->
      <div class="flex min-w-0 flex-col">
        <label class="filters-field-label" for="cf-selmaho">{{
          t('components.combinedFilters.filterBySelmao')
        }}</label>
        <div class="relative">
          <Input
            id="cf-selmaho"
            v-model="filters.selmaho"
            type="text"
            :placeholder="t('components.combinedFilters.placeholderSelmaho')"
            class="input-field w-full h-8"
            @input="debouncedFilterChange"
          />
          <IconButtonGhost
            v-if="filters.selmaho"
            class="absolute right-2 top-1/2 -translate-y-1/2"
            :aria-label="t('components.combinedFilters.clearFilter')"
            @click="clearFilter('selmaho')"
          >
            <X class="h-5 w-5" />
          </IconButtonGhost>
        </div>
      </div>

      <!-- Author / username filter with user hints -->
      <div class="flex min-w-0 flex-col">
        <label class="filters-field-label" for="cf-username">{{
          t('components.combinedFilters.filterByAuthor')
        }}</label>
        <div ref="usernameFieldRef" class="relative">
          <Input
            id="cf-username"
            v-model="filters.username"
            type="text"
            autocomplete="off"
            role="combobox"
            :aria-expanded="showUsernameHints"
            aria-autocomplete="list"
            aria-controls="cf-username-hints"
            :placeholder="t('components.combinedFilters.placeholderUsername')"
            class="input-field w-full h-8"
            @input="onUsernameInput"
            @focus="onUsernameFocus"
            @keydown="onUsernameKeydown"
          />
          <IconButtonGhost
            v-if="filters.username"
            class="absolute right-2 top-1/2 -translate-y-1/2"
            :aria-label="t('components.combinedFilters.clearFilter')"
            @click="clearUsernameFilter"
          >
            <X class="h-5 w-5" />
          </IconButtonGhost>
        </div>
        <Teleport to="body">
          <div
            v-if="showUsernameHints"
            id="cf-username-hints"
            ref="usernameHintsRef"
            role="listbox"
            class="dropdown-menu-panel !w-auto min-w-[12rem] max-h-60 py-1"
            :style="usernameHintsStyle"
          >
            <p
              v-if="isSearchingUsers"
              class="px-3 py-2 text-sm text-gray-500"
              role="status"
            >
              {{ t('components.combinedFilters.searchingUsers') }}
            </p>
            <template v-else-if="usernameHints.length > 0">
              <Button
                v-for="(user, index) in usernameHints"
                :id="`cf-username-hint-${index}`"
                :key="user.user_id"
                variant="neutral"
                type="button"
                role="option"
                class="assistant-session-row flex w-full items-center gap-3 rounded-none px-3 py-2"
                :class="
                  index === usernameHintIndex
                    ? 'assistant-session-row--active'
                    : 'assistant-session-row--idle'
                "
                :aria-selected="index === usernameHintIndex"
                @mousedown.prevent="selectUsernameHint(user)"
              >
                <div class="avatar-placeholder-sm !h-7 !w-7 shrink-0 text-xs">
                  {{ user.username[0]?.toUpperCase() }}
                </div>
                <div class="min-w-0 flex-1 text-left">
                  <p class="truncate text-sm font-medium text-gray-900">{{ user.username }}</p>
                  <p v-if="user.realname" class="truncate text-xs text-gray-500">
                    {{ user.realname }}
                  </p>
                </div>
              </Button>
            </template>
            <p
              v-else-if="filters.username.trim()"
              class="px-3 py-2 text-sm text-gray-500"
              role="status"
            >
              {{ t('components.combinedFilters.noUsersFound') }}
            </p>
          </div>
        </Teleport>
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
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { useRoute } from 'vue-router'

import { fetchDefinitionsTypes, listUsers } from '@/api'

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

type UserHint = {
  user_id: number
  username: string
  realname?: string | null
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
const lastAutoExpandedSelmaho = ref('')
const wordTypes = ref<WordTypeOption[]>([])
const route = useRoute()

const usernameFieldRef = ref<HTMLElement | null>(null)
const usernameHintsRef = ref<HTMLElement | null>(null)
const usernameHints = ref<UserHint[]>([])
const usernameHintsOpen = ref(false)
const usernameHintIndex = ref(-1)
const isSearchingUsers = ref(false)
const usernameHintsStyle = ref<Record<string, string>>({})

const USERNAME_HINTS_GAP_PX = 4
const USERNAME_SEARCH_DELAY_MS = 250
let usernameSearchTimer: ReturnType<typeof setTimeout> | null = null

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

const showUsernameHints = computed(
  () => usernameHintsOpen.value && Boolean(filters.value.username?.trim())
)

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
  maybeAutoExpandForSelmaho(props.modelValue.selmaho ?? '', '')
  document.addEventListener('mousedown', handleUsernameHintsOutsideClick)
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
  clearUsernameSearchTimer()
  document.removeEventListener('mousedown', handleUsernameHintsOutsideClick)
  window.removeEventListener('resize', updateUsernameHintsPosition)
  window.removeEventListener('scroll', updateUsernameHintsPosition, true)
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

function clearUsernameSearchTimer() {
  if (usernameSearchTimer) {
    clearTimeout(usernameSearchTimer)
    usernameSearchTimer = null
  }
}

function updateUsernameHintsPosition() {
  const field = usernameFieldRef.value
  if (!field) return

  const rect = field.getBoundingClientRect()
  usernameHintsStyle.value = {
    top: `${rect.bottom + USERNAME_HINTS_GAP_PX}px`,
    left: `${rect.left}px`,
    width: `${rect.width}px`,
  }
}

async function openUsernameHints() {
  const wasOpen = usernameHintsOpen.value
  usernameHintsOpen.value = true
  await nextTick()
  updateUsernameHintsPosition()
  if (!wasOpen) {
    window.addEventListener('resize', updateUsernameHintsPosition)
    window.addEventListener('scroll', updateUsernameHintsPosition, true)
  }
}

function closeUsernameHints() {
  usernameHintsOpen.value = false
  usernameHintIndex.value = -1
  window.removeEventListener('resize', updateUsernameHintsPosition)
  window.removeEventListener('scroll', updateUsernameHintsPosition, true)
}

function handleUsernameHintsOutsideClick(event: MouseEvent) {
  if (!usernameHintsOpen.value) return
  const target = event.target
  if (!(target instanceof Node)) return
  if (usernameFieldRef.value?.contains(target) || usernameHintsRef.value?.contains(target)) {
    return
  }
  closeUsernameHints()
}

async function searchUsernameHints(query: string) {
  const trimmed = query.trim()
  if (!trimmed) {
    usernameHints.value = []
    isSearchingUsers.value = false
    closeUsernameHints()
    return
  }

  isSearchingUsers.value = true
  await openUsernameHints()
  try {
    const response = await listUsers({ search: trimmed, per_page: 12 })
    const users = (response.data.users ?? []) as UserHint[]
    usernameHints.value = users
    usernameHintIndex.value = users.length > 0 ? 0 : -1
    await nextTick()
    updateUsernameHintsPosition()
  } catch (error) {
    console.error('Failed to search users for author filter:', error)
    usernameHints.value = []
    usernameHintIndex.value = -1
  } finally {
    isSearchingUsers.value = false
  }
}

function scheduleUsernameSearch() {
  clearUsernameSearchTimer()
  const query = filters.value.username ?? ''
  if (!query.trim()) {
    usernameHints.value = []
    isSearchingUsers.value = false
    closeUsernameHints()
    return
  }
  isSearchingUsers.value = true
  void openUsernameHints()
  usernameSearchTimer = setTimeout(() => {
    usernameSearchTimer = null
    void searchUsernameHints(query)
  }, USERNAME_SEARCH_DELAY_MS)
}

function onUsernameInput() {
  debouncedFilterChange()
  scheduleUsernameSearch()
}

function onUsernameFocus() {
  if (filters.value.username?.trim()) {
    scheduleUsernameSearch()
  }
}

function selectUsernameHint(user: UserHint) {
  clearUsernameSearchTimer()
  clearDebounceTimer()
  filters.value.username = user.username
  closeUsernameHints()
  usernameHints.value = []
  emitUpdate()
}

function clearUsernameFilter() {
  clearUsernameSearchTimer()
  usernameHints.value = []
  closeUsernameHints()
  clearFilter('username')
}

function onUsernameKeydown(event: KeyboardEvent) {
  if (!showUsernameHints.value) {
    if (event.key === 'ArrowDown' && filters.value.username?.trim()) {
      scheduleUsernameSearch()
    }
    return
  }

  if (event.key === 'Escape') {
    event.preventDefault()
    closeUsernameHints()
    return
  }

  if (event.key === 'ArrowDown') {
    event.preventDefault()
    if (usernameHints.value.length === 0) return
    usernameHintIndex.value = (usernameHintIndex.value + 1) % usernameHints.value.length
    return
  }

  if (event.key === 'ArrowUp') {
    event.preventDefault()
    if (usernameHints.value.length === 0) return
    usernameHintIndex.value =
      (usernameHintIndex.value - 1 + usernameHints.value.length) % usernameHints.value.length
    return
  }

  if (event.key === 'Enter' && usernameHintIndex.value >= 0) {
    const user = usernameHints.value[usernameHintIndex.value]
    if (user) {
      event.preventDefault()
      selectUsernameHint(user)
    }
  }
}

const hasFilledAdvancedFilters = computed(() => {
  return Boolean(
    filters.value.selmaho ||
    filters.value.username ||
    filters.value.word_type ||
    filters.value.source_langid !== 1
  )
})

const shouldPulseExpandChevron = computed(() => !expanded.value && hasFilledAdvancedFilters.value)

const hasAnyActiveFilters = computed(() => {
  return Boolean(
    selectedLangs.value.length > 0 ||
    hasFilledAdvancedFilters.value ||
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

function maybeAutoExpandForSelmaho(newSelmaho: string, prevSelmaho: string) {
  if (newSelmaho && newSelmaho !== prevSelmaho && newSelmaho !== lastAutoExpandedSelmaho.value) {
    lastAutoExpandedSelmaho.value = newSelmaho
    if (!expanded.value) {
      expanded.value = true
      emitUpdate()
    }
  }
  if (!newSelmaho) {
    lastAutoExpandedSelmaho.value = ''
  }
}

watch(
  () => props.modelValue.selmaho,
  (newSelmaho, oldSelmaho) => {
    maybeAutoExpandForSelmaho(newSelmaho ?? '', oldSelmaho ?? '')
  }
)

const clearFilter = (filterName) => {
  // Clear any pending timeouts first to prevent them from firing after clearing
  clearDebounceTimer()
  if (filterName === 'username') {
    clearUsernameSearchTimer()
    usernameHints.value = []
    closeUsernameHints()
  }
  filters.value[filterName] = ''
  emitUpdate()
}

const resetAllFilters = () => {
  clearUsernameSearchTimer()
  usernameHints.value = []
  closeUsernameHints()

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
  if (!expanded.value) {
    closeUsernameHints()
  }
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

.filters-expand-attention {
  border-width: 2px;
  animation: filtersExpandBorderPulse 1.4s ease-in-out infinite;
}

.filters-expand-chevron-pulse {
  color: rgb(8 145 178);
  animation: filtersExpandChevronPulse 1.2s ease-in-out infinite;
}

.filters-expand-chevron-pulse svg {
  color: inherit;
}

@keyframes filtersExpandBorderPulse {
  0%,
  100% {
    border-color: rgb(156 163 175);
    background-color: transparent;
    box-shadow: none;
  }

  50% {
    border-color: rgb(6 182 212);
    background-color: rgb(207 250 254 / 0.7);
    box-shadow: 0 0 0 3px rgb(6 182 212 / 0.35);
  }
}

@keyframes filtersExpandChevronPulse {
  0%,
  100% {
    opacity: 0.45;
    transform: scale(1);
  }

  50% {
    opacity: 1;
    transform: scale(1.28);
  }
}
</style>
