<template>
  <div class="filters space-y-4">
    <!-- Language row: top by default; on semantic graph page (`languagesInExpandedPanel`) it lives inside the expanded panel. -->

    <div class="filters-bar-row" :class="{ 'sm:!justify-end': languagesInExpandedPanel }">
      <div
        v-if="!languagesInExpandedPanel"
        class="filters-primary-selects"
        :class="{ '!grid-cols-1 sm:!w-80 sm:!flex-none': !showCollectionFilter }"
      >
        <MultiSelectDropdown
          v-model="selectedLangs"
          :options="languages"
          :suggested-options="languageSuggestions"
          :suggested-label="t('filters.recentSuggestions')"
          :max-selected-labels="3"
          :option-value="(lang: LanguageOption) => lang.id"
          :option-label="(lang: LanguageOption) => lang.real_name"
          :search-field-keys="languageFilterSearchFieldKeys"
          :placeholder="t('filters.selectLanguages')"
          :search-placeholder="t('filters.searchLanguages')"
          :select-all-label="t('filters.selectAll')"
          :deselect-all-label="t('filters.deselectAll')"
          :empty-filter-label="t('filters.noMatches')"
          :preface="t('filters.languagesPreface')"
          full-bleed-mobile-panel
          class="w-full min-w-0"
        />
        <MultiSelectDropdown
          v-if="showCollectionFilter"
          v-model="selectedCollectionUsers"
          :options="collectionUserOptions"
          :suggested-options="collectionUserSuggestions"
          :suggested-label="t('filters.recentSuggestions')"
          :max-selected-labels="3"
          :option-value="collectionUserOptionValue"
          :option-label="collectionUserOptionLabel"
          :search-field-keys="collectionUserSearchFieldKeys"
          :placeholder="t('filters.selectCollectionsAndUsers')"
          :search-placeholder="t('filters.searchCollectionsAndUsers')"
          :select-all-label="t('filters.selectAll')"
          :deselect-all-label="t('filters.deselectAll')"
          :empty-filter-label="t('filters.noCollectionUserMatches')"
          :preface="t('filters.collectionsAndUsersPreface')"
          full-bleed-mobile-panel
          class="w-full min-w-0"
          @open="onCollectionUserDropdownOpen"
          @search="onCollectionUserDropdownSearch"
        >
          <template #option="{ option }">
            <span v-if="isCollectionPickerOption(option)" class="dropdown-option-rich">
              <b>{{ option.name }}</b>
              <i v-if="option.owner_username">{{
                t('filters.collectionByUser', { username: option.owner_username })
              }}</i>
            </span>
            <span v-else-if="isUserPickerOption(option)" class="dropdown-option-rich">
              {{ t('filters.userOptionPrefix') }}
              <b>{{ option.username }}</b>
            </span>
          </template>
        </MultiSelectDropdown>
      </div>
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
      <div class="col-span-2 flex min-w-0 flex-col">
        <Button
          type="button"
          variant="toolbar"
          class="w-full justify-start"
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
          <span class="text-sm select-none">{{ t('searchForm.modes.searchInPhrases') }}</span>
        </Button>
      </div>

      <div v-if="languagesInExpandedPanel" class="col-span-2 flex min-w-0 flex-col">
        <label class="filters-field-label">{{ t('filters.selectLanguages') }}</label>
        <div class="relative w-full min-w-0 [&>div]:block [&>div]:w-full">
          <MultiSelectDropdown
            v-model="selectedLangs"
            :options="languages"
            :suggested-options="languageSuggestions"
            :suggested-label="t('filters.recentSuggestions')"
            :max-selected-labels="3"
            :option-value="(lang: LanguageOption) => lang.id"
            :option-label="(lang: LanguageOption) => lang.real_name"
            :search-field-keys="languageFilterSearchFieldKeys"
            :placeholder="t('filters.selectLanguages')"
            :search-placeholder="t('filters.searchLanguages')"
            :select-all-label="t('filters.selectAll')"
            :deselect-all-label="t('filters.deselectAll')"
            :empty-filter-label="t('filters.noMatches')"
            :preface="t('filters.languagesPreface')"
            full-bleed-mobile-panel
            class="w-full max-w-full"
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

      <!-- Include authors: only when the merged collections+users picker is hidden. -->
      <div v-if="!showCollectionFilter" class="col-span-2 sm:col-span-1 flex min-w-0 flex-col">
        <label class="filters-field-label">{{ t('components.combinedFilters.filterByAuthors') }}</label>
        <div class="relative w-full min-w-0 [&>div]:block [&>div]:w-full">
          <MultiSelectDropdown
            v-model="selectedUsers"
            :options="userOptions"
            :suggested-options="includeAuthorSuggestions"
            :suggested-label="t('filters.recentSuggestions')"
            :max-selected-labels="3"
            :option-value="(user: UserHint) => user.username"
            :option-label="(user: UserHint) => user.username"
            :search-field-keys="userFilterSearchFieldKeys"
            :placeholder="t('components.combinedFilters.placeholderUsernames')"
            :search-placeholder="t('components.combinedFilters.searchUsers')"
            :select-all-label="t('filters.selectAll')"
            :deselect-all-label="t('filters.deselectAll')"
            :empty-filter-label="t('components.combinedFilters.noUsersFound')"
            class="w-full max-w-full"
            @open="onUserDropdownOpen"
            @search="onUserDropdownSearch"
          />
        </div>
      </div>

      <!-- Exclude authors (optional multi-select) -->
      <div
        class="flex min-w-0 flex-col"
        :class="showCollectionFilter ? 'col-span-2' : 'col-span-2 sm:col-span-1'"
      >
        <label class="filters-field-label">{{
          t('components.combinedFilters.filterByExcludeAuthors')
        }}</label>
        <div class="relative w-full min-w-0 [&>div]:block [&>div]:w-full">
          <MultiSelectDropdown
            v-model="excludedUsers"
            :options="userOptions"
            :suggested-options="excludeAuthorSuggestions"
            :suggested-label="t('filters.recentSuggestions')"
            :max-selected-labels="3"
            :option-value="(user: UserHint) => user.username"
            :option-label="(user: UserHint) => user.username"
            :search-field-keys="userFilterSearchFieldKeys"
            :placeholder="t('components.combinedFilters.placeholderExcludeUsernames')"
            :search-placeholder="t('components.combinedFilters.searchUsers')"
            :select-all-label="t('filters.selectAll')"
            :deselect-all-label="t('filters.deselectAll')"
            :empty-filter-label="t('components.combinedFilters.noUsersFound')"
            class="w-full max-w-full"
            @open="onUserDropdownOpen"
            @search="onUserDropdownSearch"
          />
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

import { fetchDefinitionsTypes, listUsers, searchCollectionsAndUsers } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useRecentSelections } from '@/composables/useRecentSelections'

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
/** Fields used for author multiselect search. */
const userFilterSearchFieldKeys: string[] = ['username', 'realname']
/** Fields used for the merged collections + users picker. */
const collectionUserSearchFieldKeys: string[] = ['name', 'owner_username', 'username', 'realname']

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

type CollectionOption = {
  collection_id: number
  name: string
  description?: string | null
  owner_username?: string
}

type CollectionPickerOption = CollectionOption & { kind: 'collection' }
type UserPickerOption = UserHint & { kind: 'user' }
type CollectionUserOption = CollectionPickerOption | UserPickerOption

function isCollectionPickerOption(option: unknown): option is CollectionPickerOption {
  return (
    typeof option === 'object' &&
    option !== null &&
    (option as CollectionUserOption).kind === 'collection'
  )
}

function isUserPickerOption(option: unknown): option is UserPickerOption {
  return (
    typeof option === 'object' &&
    option !== null &&
    (option as CollectionUserOption).kind === 'user'
  )
}

function collectionUserOptionValue(item: CollectionUserOption): string {
  return item.kind === 'collection' ? `c:${item.collection_id}` : `u:${item.username}`
}

function collectionUserOptionLabel(item: CollectionUserOption): string {
  return item.kind === 'collection' ? item.name : item.username
}

const props = defineProps({
  modelValue: {
    type: Object,
    default: () => ({
      selmaho: '',
      usernames: [],
      excludeUsernames: [],
      isExpanded: false,
      selectedLanguages: [],
      selectedCollections: [],
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
  /** Force the Reset control visible (e.g. find-similar mode with no other filters). */
  forceShowReset: {
    type: Boolean,
    default: false,
  },
  /** Upper bound for “max nodes” on the semantic graph page (server `SEMANTIC_GRAPH_MAX_LIMIT`). */
  semanticGraphMaxNodes: {
    type: Number,
    default: 120,
  },
  /** When false, hide the collection filter (collection detail, semantic graph). */
  showCollectionFilter: {
    type: Boolean,
    default: true,
  },
})

const emit = defineEmits(['update:modelValue', 'change', 'reset'])

const auth = useAuth()
const selectedLangs = ref<LanguageOption[]>([])
const selectedUsers = ref<UserHint[]>([])
const excludedUsers = ref<UserHint[]>([])
const userOptions = ref<UserHint[]>([])
const selectedCollections = ref<CollectionOption[]>([])
const collectionOptions = ref<CollectionOption[]>([])
const expanded = ref(props.modelValue.isExpanded)
const lastAutoExpandedSelmaho = ref('')
const wordTypes = ref<WordTypeOption[]>([])
const route = useRoute()

const { recent: recentLanguages, record: recordRecentLanguage } = useRecentSelections<LanguageOption>(
  'recentLanguageSelections',
  (lang) => lang.id
)
const { recent: recentCollections, record: recordRecentCollection } =
  useRecentSelections<CollectionOption>('recentCollectionSelections', (col) => col.collection_id)
const { recent: recentIncludeAuthors, record: recordRecentIncludeAuthor } =
  useRecentSelections<UserHint>('recentIncludeAuthorSelections', (u) => u.username)
const { recent: recentExcludeAuthors, record: recordRecentExcludeAuthor } =
  useRecentSelections<UserHint>('recentExcludeAuthorSelections', (u) => u.username)
const { recent: recentCollectionUsers, record: recordRecentCollectionUser } =
  useRecentSelections<CollectionUserOption>(
    'recentCollectionUserSelections',
    (item) => collectionUserOptionValue(item)
  )

const languageSuggestions = computed(() => {
  const byId = new Map(props.languages.map((l) => [l.id, l]))
  return recentLanguages.value
    .map((item) => byId.get(item.id) ?? item)
    .filter((item) => item?.real_name)
    .slice(0, 3)
})

const collectionUserOptions = computed<CollectionUserOption[]>(() => [
  ...collectionOptions.value.map((c) => ({ kind: 'collection' as const, ...c })),
  ...userOptions.value.map((u) => ({ kind: 'user' as const, ...u })),
])

const selectedCollectionUsers = computed<CollectionUserOption[]>({
  get() {
    return [
      ...selectedCollections.value.map((c) => ({ kind: 'collection' as const, ...c })),
      ...selectedUsers.value.map((u) => ({ kind: 'user' as const, ...u })),
    ]
  },
  set(items) {
    const collections: CollectionOption[] = []
    const users: UserHint[] = []
    for (const item of items) {
      if (item.kind === 'collection') {
        collections.push({
          collection_id: item.collection_id,
          name: item.name,
          description: item.description,
          owner_username: item.owner_username,
        })
      } else {
        users.push({
          user_id: item.user_id,
          username: item.username,
          realname: item.realname,
        })
      }
    }
    selectedCollections.value = collections
    selectedUsers.value = users
  },
})

const collectionUserSuggestions = computed(() => {
  const mixed = recentCollectionUsers.value.length
    ? recentCollectionUsers.value
    : [
        ...recentCollections.value.map((c) => ({ kind: 'collection' as const, ...c })),
        ...recentIncludeAuthors.value.map((u) => ({ kind: 'user' as const, ...u })),
      ]
  const byKey = new Map(collectionUserOptions.value.map((o) => [collectionUserOptionValue(o), o]))
  return mixed
    .map((item) => byKey.get(collectionUserOptionValue(item)) ?? item)
    .filter((item) => collectionUserOptionLabel(item))
    .slice(0, 3)
})

const includeAuthorSuggestions = computed(() => {
  const byName = new Map(userOptions.value.map((u) => [u.username, u]))
  return recentIncludeAuthors.value
    .map((item) => byName.get(item.username) ?? item)
    .filter((item) => item?.username)
    .slice(0, 3)
})

const excludeAuthorSuggestions = computed(() => {
  const byName = new Map(userOptions.value.map((u) => [u.username, u]))
  return recentExcludeAuthors.value
    .map((item) => byName.get(item.username) ?? item)
    .filter((item) => item?.username)
    .slice(0, 3)
})

const USER_SEARCH_DELAY_MS = 250
const USER_LIST_PAGE_SIZE = 50
let userSearchTimer: ReturnType<typeof setTimeout> | null = null
let collectionSearchTimer: ReturnType<typeof setTimeout> | null = null

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

function parseUsernameList(value: unknown): string[] {
  if (Array.isArray(value)) {
    return value.map((v) => String(v).trim()).filter(Boolean)
  }
  if (typeof value === 'string' && value.trim()) {
    return value
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)
  }
  return []
}

function usersFromNames(names: string[], known: UserHint[]): UserHint[] {
  const byName = new Map(known.map((u) => [u.username, u]))
  return names.map((username) => byName.get(username) ?? { user_id: 0, username })
}

function mergeUserOptions(users: UserHint[]) {
  const byName = new Map<string, UserHint>()
  for (const u of userOptions.value) byName.set(u.username, u)
  for (const u of users) byName.set(u.username, u)
  for (const u of selectedUsers.value) {
    if (!byName.has(u.username)) byName.set(u.username, u)
  }
  for (const u of excludedUsers.value) {
    if (!byName.has(u.username)) byName.set(u.username, u)
  }
  for (const u of recentIncludeAuthors.value) {
    if (!byName.has(u.username)) byName.set(u.username, u)
  }
  for (const u of recentExcludeAuthors.value) {
    if (!byName.has(u.username)) byName.set(u.username, u)
  }
  userOptions.value = [...byName.values()]
}

function parseCollectionIdList(value: unknown): number[] {
  if (Array.isArray(value)) {
    return value.map((v) => Number(v)).filter((n) => Number.isFinite(n) && n > 0)
  }
  if (typeof value === 'string' && value.trim()) {
    return value
      .split(',')
      .map((s) => Number(s.trim()))
      .filter((n) => Number.isFinite(n) && n > 0)
  }
  return []
}

function collectionsFromIds(ids: number[], known: CollectionOption[]): CollectionOption[] {
  const byId = new Map(known.map((c) => [c.collection_id, c]))
  return ids.map((id) => byId.get(id) ?? { collection_id: id, name: `#${id}` })
}

function mergeCollectionOptions(cols: CollectionOption[]) {
  const byId = new Map<number, CollectionOption>()
  for (const c of collectionOptions.value) byId.set(c.collection_id, c)
  for (const c of cols) byId.set(c.collection_id, c)
  for (const c of selectedCollections.value) {
    if (!byId.has(c.collection_id)) byId.set(c.collection_id, c)
  }
  for (const c of recentCollections.value) {
    if (!byId.has(c.collection_id)) byId.set(c.collection_id, c)
  }
  collectionOptions.value = [...byId.values()]
  if (selectedCollections.value.length) {
    selectedCollections.value = collectionsFromIds(
      selectedCollections.value.map((c) => c.collection_id),
      collectionOptions.value
    )
  }
}

watch(
  () => props.modelValue,
  (newVal) => {
    expanded.value = newVal.isExpanded
    filters.value = {
      selmaho: newVal.selmaho,
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

    const includeNames = parseUsernameList(newVal.usernames ?? newVal.username)
    const excludeNames = parseUsernameList(newVal.excludeUsernames)
    if (selectedUsers.value.map((u) => u.username).join(',') !== includeNames.join(',')) {
      selectedUsers.value = usersFromNames(includeNames, userOptions.value)
    }
    if (excludedUsers.value.map((u) => u.username).join(',') !== excludeNames.join(',')) {
      excludedUsers.value = usersFromNames(excludeNames, userOptions.value)
    }
    mergeUserOptions([])

    const collectionIds = parseCollectionIdList(newVal.selectedCollections)
    if (
      selectedCollections.value.map((c) => c.collection_id).join(',') !== collectionIds.join(',')
    ) {
      selectedCollections.value = collectionsFromIds(collectionIds, collectionOptions.value)
    }
    mergeCollectionOptions([])
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
  if (props.showCollectionFilter) {
    void fetchCollectionUsers('')
  }
})

watch(
  () => auth.state.isLoggedIn,
  (loggedIn) => {
    if (loggedIn && props.showCollectionFilter) {
      void fetchCollectionUsers('')
    }
  }
)

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
  clearUserSearchTimer()
  clearCollectionSearchTimer()
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

function clearUserSearchTimer() {
  if (userSearchTimer) {
    clearTimeout(userSearchTimer)
    userSearchTimer = null
  }
}

async function fetchUsers(search = '') {
  try {
    const response = await listUsers({
      search: search.trim() || undefined,
      per_page: USER_LIST_PAGE_SIZE,
      sort_by: 'username',
      sort_order: 'asc',
    })
    mergeUserOptions((response.data.users ?? []) as UserHint[])
  } catch (error) {
    console.error('Failed to search users for author filter:', error)
  }
}

function onUserDropdownOpen() {
  if (props.showCollectionFilter) {
    void fetchCollectionUsers('')
  } else {
    void fetchUsers('')
  }
}

function onUserDropdownSearch(query: string) {
  clearUserSearchTimer()
  userSearchTimer = setTimeout(() => {
    userSearchTimer = null
    void (props.showCollectionFilter ? fetchCollectionUsers(query) : fetchUsers(query))
  }, USER_SEARCH_DELAY_MS)
}

function clearCollectionSearchTimer() {
  if (collectionSearchTimer) {
    clearTimeout(collectionSearchTimer)
    collectionSearchTimer = null
  }
}

async function fetchCollectionUsers(search = '') {
  try {
    const response = await searchCollectionsAndUsers({
      search: search.trim() || undefined,
      per_kind: 20,
    })
    const items = (response.data.items ?? []) as CollectionUserOption[]
    const cols: CollectionOption[] = []
    const users: UserHint[] = []
    for (const item of items) {
      if (item.kind === 'collection') {
        cols.push({
          collection_id: item.collection_id,
          name: item.name,
          owner_username: item.owner_username,
        })
      } else if (item.kind === 'user') {
        users.push({
          user_id: item.user_id,
          username: item.username,
          realname: item.realname,
        })
      }
    }
    mergeCollectionOptions(cols)
    mergeUserOptions(users)
  } catch (error) {
    console.error('Failed to list collections and users for filter:', error)
  }
}

function onCollectionUserDropdownOpen() {
  void fetchCollectionUsers('')
}

function onCollectionUserDropdownSearch(query: string) {
  clearCollectionSearchTimer()
  collectionSearchTimer = setTimeout(() => {
    collectionSearchTimer = null
    void fetchCollectionUsers(query)
  }, USER_SEARCH_DELAY_MS)
}

const hasFilledAdvancedFilters = computed(() => {
  return Boolean(
    filters.value.selmaho ||
      (selectedUsers.value.length > 0 && !props.showCollectionFilter) ||
      excludedUsers.value.length > 0 ||
      filters.value.word_type ||
      filters.value.source_langid !== 1 ||
      !searchInPhrases.value
  )
})

const shouldPulseExpandChevron = computed(() => !expanded.value && hasFilledAdvancedFilters.value)

const hasAnyActiveFilters = computed(() => {
  return Boolean(
    props.forceShowReset ||
      selectedLangs.value.length > 0 ||
      selectedCollections.value.length > 0 ||
      selectedUsers.value.length > 0 ||
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
  }

  // Debounce the filter change - only trigger after user stops typing
  // This prevents excessive API calls while user is actively typing
  debounceTimer = setTimeout(() => {
    // Only emit if filters haven't changed (to prevent race conditions)
    if (filters.value.selmaho === currentFilters.selmaho) {
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
    usernames: selectedUsers.value.map((u) => u.username),
    excludeUsernames: excludedUsers.value.map((u) => u.username),
    isExpanded: expanded.value,
    selectedLanguages: selectedLangs.value.map((lang) => lang.id),
    selectedCollections: selectedCollections.value.map((col) => col.collection_id),
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
  filters.value[filterName] = ''
  emitUpdate()
}

const resetAllFilters = () => {
  clearUserSearchTimer()
  clearCollectionSearchTimer()
  selectedUsers.value = []
  excludedUsers.value = []
  selectedCollections.value = []

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
    usernames: [],
    excludeUsernames: [],
    isExpanded: false,
    selectedLanguages: defaultLangs.map((lang) => lang.id),
    selectedCollections: [],
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
      if (oldLangs?.length) {
        const prevIds = new Set(oldLangs.map((l) => l.id))
        for (const lang of newLangs) {
          if (!prevIds.has(lang.id)) recordRecentLanguage(lang)
        }
      }
      emitUpdate()
    }
  },
  { deep: true }
)

watch(
  selectedCollections,
  (newCols, oldCols) => {
    if (JSON.stringify(newCols) === JSON.stringify(oldCols)) return
    if (oldCols) {
      const prevIds = new Set(oldCols.map((c) => c.collection_id))
      for (const col of newCols) {
        if (!prevIds.has(col.collection_id)) {
          recordRecentCollection(col)
          recordRecentCollectionUser({ kind: 'collection', ...col })
        }
      }
      emitUpdate()
    }
  },
  { deep: true }
)

watch(
  selectedUsers,
  (next, prev) => {
    if (!prev) return
    const prevNames = new Set(prev.map((u) => u.username))
    for (const user of next) {
      if (!prevNames.has(user.username)) {
        recordRecentIncludeAuthor(user)
        recordRecentCollectionUser({ kind: 'user', ...user })
      }
    }
    emitUpdate()
  },
  { deep: true }
)

watch(
  excludedUsers,
  (next, prev) => {
    if (!prev) return
    const prevNames = new Set(prev.map((u) => u.username))
    for (const user of next) {
      if (!prevNames.has(user.username)) recordRecentExcludeAuthor(user)
    }
    emitUpdate()
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
