<template>
  <div class="search-form max-w-3xl mx-auto w-full">
    <div class="relative flex w-full min-w-0">
      <SearchInput
        ref="searchInput"
        class="flex-1"
        :model-value="query"
        :placeholder="getPlaceholder"
        :is-loading="isSearching"
        joined-right
        @update:model-value="handleQueryUpdate"
        @search="handleSearch"
        @clear="handleClear"
      />
      <button
        type="button"
        class="dropdown-trigger dropdown-trigger--search-bar-trailing w-auto max-w-none rounded-l-none rounded-r-full shrink-0"
        :aria-label="t('flashcardStudy.wavesButton')"
        :aria-pressed="isWavesMode"
        :class="{
          'ring-1 ring-blue-500 border-blue-500 z-10': isWavesMode,
        }"
        @click="toggleMode"
      >
        {{ t('flashcardStudy.wavesButton') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { BookOpen, MessageSquare } from 'lucide-vue-next'
import SearchInput from '@/components/SearchInput.vue'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'

const { t } = useI18n()

const modes = ref([
  { label: t('searchForm.modes.dictionary'), value: 'dictionary', icon: BookOpen },
  { label: t('searchForm.modes.comments'), value: 'comments', icon: MessageSquare },
])

const props = defineProps({
  initialQuery: { type: String, default: '' },
  initialMode: { type: String, default: 'semantic' },
})

const emit = defineEmits(['search'])

const searchInput = ref<InstanceType<typeof SearchInput> | null>(null)
const query = ref(normalizeSearchQuery(props.initialQuery))
const modeValue = ref(props.initialMode === 'semantic' ? 'dictionary' : props.initialMode)
const isSearching = ref(false)
let searchTimeout: number | null = null

const DEBOUNCE_DELAY = 450

const isWavesMode = computed(() => modeValue.value === 'comments')

const getPlaceholder = computed(() => {
  switch (modeValue.value) {
    case 'dictionary':
      return t('searchForm.placeholder.dictionary')
    case 'comments':
      return t('searchForm.placeholder.comments')
    default:
      return t('searchForm.placeholder.default')
  }
})

function clearSearchTimeout() {
  if (searchTimeout !== null) {
    window.clearTimeout(searchTimeout)
    searchTimeout = null
  }
  isSearching.value = false
}

function emitSearch() {
  emit('search', { query: normalizeSearchQuery(query.value), mode: modeValue.value })
}

function handleQueryUpdate(value: string) {
  query.value = value
  clearSearchTimeout()

  if (!value.trim()) {
    emitSearch()
    return
  }

  const currentQuery = value

  searchTimeout = window.setTimeout(() => {
    if (query.value === currentQuery) {
      if (query.value.trim()) {
        isSearching.value = true
      }
      emitSearch()
    }
    searchTimeout = null
  }, DEBOUNCE_DELAY)
}

function handleSearch() {
  clearSearchTimeout()
  isSearching.value = true
  emitSearch()
}

function handleClear() {
  searchInput.value?.focus()
}

function toggleMode() {
  modeValue.value = modeValue.value === 'dictionary' ? 'comments' : 'dictionary'
  clearSearchTimeout()
  emitSearch()
  searchInput.value?.focus()
}

watch(
  () => props.initialQuery,
  (newValue) => {
    clearSearchTimeout()
    query.value = normalizeSearchQuery(newValue)
  }
)

watch(
  () => props.initialMode,
  (newValue) => {
    const target = newValue === 'semantic' ? 'dictionary' : newValue
    if (modes.value.some((m) => m.value === target)) {
      clearSearchTimeout()
      modeValue.value = target
    }
  }
)

onBeforeUnmount(() => {
  clearSearchTimeout()
})

defineExpose({
  focusInput: () => searchInput.value?.focus(),
})
</script>
