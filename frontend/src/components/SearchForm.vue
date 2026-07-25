<template>
  <div class="search-form max-w-3xl mx-auto w-full">
    <div class="relative flex w-full min-w-0">
      <div class="shrink-0">
        <ToolbarSelectDropdown
          id="search-mode"
          aria-label="Search mode"
          trigger-class="h-10 rounded-l-full rounded-r-none bg-gray-50"
        >
          <template #label>
            <component :is="selectedMode.icon" class="h-4 w-4 shrink-0" aria-hidden="true" />
            <span class="hidden sm:inline">{{ selectedMode.label }}</span>
          </template>
          <ToolbarSelectDropdownItem
            v-for="m in modes"
            :key="m.value"
            @click="handleModeUpdate(m.value)"
          >
            <div class="inline-flex items-center gap-2">
              <component :is="m.icon" class="h-4 w-4 shrink-0" aria-hidden="true" />
              <span>{{ m.label }}</span>
            </div>
          </ToolbarSelectDropdownItem>
        </ToolbarSelectDropdown>
      </div>
      <SearchInput
        ref="searchInput"
        class="flex-1"
        :model-value="query"
        :mode-value="modeValue"
        :placeholder="getPlaceholder"
        :is-loading="isSearching"
        @update:model-value="handleQueryUpdate"
        @search="handleSearch"
        @clear="handleClear"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { BookOpen, MessageSquare } from 'lucide-vue-next'
import { ToolbarSelectDropdown, ToolbarSelectDropdownItem } from '@packages/ui'
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

const selectedMode = computed(
  () => modes.value.find((m) => m.value === modeValue.value) ?? modes.value[0]
)

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
}

function emitSearch() {
  emit('search', { query: normalizeSearchQuery(query.value), mode: modeValue.value })
}

function handleQueryUpdate(value: string) {
  query.value = value
  clearSearchTimeout()

  if (!value.trim()) {
    isSearching.value = false
    emitSearch()
    return
  }

  isSearching.value = true
  const currentQuery = value

  searchTimeout = window.setTimeout(() => {
    if (query.value === currentQuery) {
      emitSearch()
    } else {
      isSearching.value = false
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
  clearSearchTimeout()
  query.value = ''
  isSearching.value = false
  emitSearch()
  searchInput.value?.focus()
}

function handleModeUpdate(value: string) {
  modeValue.value = value
  clearSearchTimeout()
  emitSearch()
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
