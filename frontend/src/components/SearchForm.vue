<template>

  <div class="search-form max-w-3xl mx-auto">

    <div class="flex flex-col sm:flex-row gap-2 sm:gap-0">

      <div
        class="relative z-10 w-full sm:w-56 shrink-0 [&>div]:block [&>div]:w-full"
      >
         <Dropdown
          > <template #trigger
            > <button
              type="button"
              class="dropdown-trigger dropdown-trigger--search-bar-leading"
            >

              <div v-if="mode" class="flex items-center gap-2 min-w-0">
                 <component :is="mode.icon" class="h-4 w-4 shrink-0" :class="mode.color" /> <span
                  class="truncate"
                  >{{ mode.name }}</span
                >
              </div>
               <span v-else class="text-gray-500">{{ $t('searchForm.selectSearchMode') }}</span
              > <ChevronDown class="h-4 w-4 shrink-0 text-gray-500" /> </button
            > </template
          > <button
            v-for="m in modes"
            :key="m.value"
            type="button"
            class="w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 flex items-center gap-2"
            @click="selectMode(m)"
          >
             <component :is="m.icon" class="h-4 w-4 shrink-0" :class="m.color" /> {{ m.name }} </button
          > </Dropdown
        >
      </div>

      <div class="search-form-query-col">
         <input
          ref="searchInput"
          v-model="query"
          :placeholder="getPlaceholder"
          :class="`input-field w-full text-base h-10 sm:rounded-l-none ${query ? 'pr-10' : ''}`"
          @input="handleInput"
        />
        <div class="absolute right-3 top-1/2 transform -translate-y-1/2 flex items-center">

          <div
            v-if="isSearching"
            class="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-500"
          />
           <button
            v-else-if="query"
            class="text-gray-400 hover:text-gray-600 hover:bg-gray-100 transition-colors duration-200 p-1 rounded-full"
            @click="clearInput"
          >
             <X class="h-5 w-5" /> </button
          >
        </div>

      </div>

    </div>

  </div>

</template>

<script setup lang="ts">
import { Book, ChevronDown, Waves, X } from 'lucide-vue-next'
import { Dropdown } from '@packages/ui'
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'

const { t } = useI18n()

const modes = ref([
  {
    name: t('searchForm.modes.dictionary'),
    value: 'dictionary',
    icon: Book,
    color: 'text-blue-500',
  },
  {
    name: t('searchForm.modes.comments'),
    value: 'comments',
    icon: Waves,
    color: 'text-purple-500',
  },
])

const props = defineProps({
  initialQuery: {
    type: String,
    default: '',
  },
  initialMode: {
    type: String,
    default: 'semantic',
  },
})

const emit = defineEmits(['search'])

const searchInput = ref(null)
const query = ref(normalizeSearchQuery(props.initialQuery))
const mode = ref(
  modes.value.find(
    (m) => m.value === (props.initialMode === 'semantic' ? 'dictionary' : props.initialMode)
  ) || modes.value[0]
)
const isSearching = ref(false)
let searchTimeout = null

// Debounce delay: 450ms is optimal for search inputs (400-500ms range)
// This balances responsiveness with reducing unnecessary API calls
const DEBOUNCE_DELAY = 450

const getPlaceholder = computed(() => {
  switch (mode.value?.value) {
    case 'dictionary':
      return t('searchForm.placeholder.dictionary')
    case 'comments':
      return t('searchForm.placeholder.comments')
    default:
      return t('searchForm.placeholder.default')
  }
})

function clearSearchTimeout() {
  if (searchTimeout) {
    window.clearTimeout(searchTimeout)
    searchTimeout = null
  }
  isSearching.value = false
}

function handleInput() {
  query.value = normalizeSearchQuery(query.value)
  // Clear any pending timeouts to prevent stale searches
  clearSearchTimeout()

  // Capture current query value to check in timeout
  const currentQuery = query.value

  // Debounce the search - only trigger after user stops typing
  // This prevents excessive API calls while user is actively typing
  searchTimeout = window.setTimeout(() => {
    // Only emit if query hasn't changed (to prevent race conditions)
    if (query.value === currentQuery) {
      // Show loading spinner when search actually starts
      if (query.value.trim()) {
        isSearching.value = true
      }
      emitSearch()
      // Note: isSearching will be cleared by parent component when search completes
      // or by next input/clear action
    }
    searchTimeout = null
  }, DEBOUNCE_DELAY)
}

function emitSearch() {
  let effectiveMode = mode.value.value
  emit('search', { query: normalizeSearchQuery(query.value), mode: effectiveMode })
}

function clearInput() {
  // Clear any pending timeouts first to prevent them from firing after clearing
  clearSearchTimeout()
  query.value = ''
  emitSearch()
  focusInput()
}

function onModeChange() {
  // Clear any pending timeouts when mode changes to prevent stale searches
  clearSearchTimeout()
  emitSearch()
}

function selectMode(m) {
  mode.value = m
  onModeChange()
}

watch(
  () => props.initialQuery,
  (newValue) => {
    // Clear any pending timeouts when query changes externally
    clearSearchTimeout()
    query.value = normalizeSearchQuery(newValue)
  }
)

watch(
  () => props.initialMode,
  (newValue) => {
    const targetModeValue = newValue === 'semantic' ? 'dictionary' : newValue
    const newMode = modes.value.find((m) => m.value === targetModeValue)
    if (newMode) {
      // Clear any pending timeouts when mode changes externally
      clearSearchTimeout()
      mode.value = newMode
    }
  }
)

// Clean up timeout on component unmount
onBeforeUnmount(() => {
  clearSearchTimeout()
})

function focusInput() {
  searchInput.value?.focus()
}

defineExpose({
  focusInput,
})
</script>

