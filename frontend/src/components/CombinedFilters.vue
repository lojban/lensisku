<template>
  <div class="filters space-y-4">
    <!-- Language Filter Section -->
    <div class="flex flex-col sm:flex-row items-center sm:justify-between gap-4 p-4 bg-white rounded-lg shadow-sm">
      <MultiSelect v-model="selectedLangs" :options="languages" :max-selected-labels="3" name="id"
        :option-label="(lang) => `${lang.real_name} (${lang.english_name})`" filter
        :placeholder="t('filters.selectLanguages')" class="w-full sm:w-80 !rounded-full" />

      <div class="flex items-center gap-2 self-end md:self-center">
        <button v-if="hasAnyActiveFilters" class="btn-empty h-8" @click="resetAllFilters">
          {{ t('filters.resetAllFilters') }}
        </button>

        <button class="btn-empty h-8" @click="toggleExpanded">
          <ChevronDown class="h-5 w-5 transition-transform duration-200" :class="{ 'rotate-180': expanded }"
            :stroke-width="2" />
        </button>
      </div>
    </div>

    <div v-show="expanded" class="mt-3 space-y-6 bg-white rounded-lg shadow-sm p-4"
      :class="{ 'animate-expandSection': expanded }">
      <!-- Advanced Filters -->
      <div class="space-y-4">
        <div v-if="showWordType">
          <label class="block text-sm font-medium text-gray-700 mb-2">{{ t('filters.filterBy.wordType') }}</label>
          <div class="relative">
            <select v-model="filters.word_type" class="input-field w-full" @change="handleWordTypeChange">
              <option value="" disabled selected>
                {{ t('filters.selectWordType') }}
              </option>
              <option v-for="type in wordTypes" :key="type.type_id" :value="type">
                {{ type.descriptor }}
              </option>
            </select>
          </div>
        </div>

        <!-- Unified input fields with clear buttons -->
        <div v-for="field in ['selmaho', 'username']" :key="field" class="relative">
          <label class="block text-sm font-medium text-gray-700 mb-1">
            {{ field === 'selmaho' ? t('components.combinedFilters.filterBySelmao') : t('components.combinedFilters.filterByAuthor') }}
          </label>
          <div class="relative">
            <input v-model="filters[field]" type="text" :placeholder="t(`components.combinedFilters.placeholder${field.charAt(0).toUpperCase() + field.slice(1)}`)" class="input-field w-full"
              @input="debouncedFilterChange">
            <button v-if="filters[field]"
              class="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded-full text-gray-400 hover:text-gray-600 hover:bg-gray-100 transition-colors duration-200 [&>svg]:hover:text-current"
              @click="clearFilter(field)">
              <X class="h-4 w-4" />
            </button>
          </div>
        </div>
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">{{ t('filters.filterBy.sourceLanguage', 'Entry Language') }}</label>
        <div class="relative">
          <select v-model="filters.source_langid" class="input-field w-full" @change="emitUpdate">
            <option :value="1">{{ t('filters.defaultSourceLanguage', 'Default (Lojban)') }}</option>
            <option v-for="lang in languages.filter(l => l.id !== 1)" :key="lang.id" :value="lang.id">
              {{ lang.real_name }} ({{ lang.english_name }})
            </option>
          </select>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup>
import { ChevronDown, X } from 'lucide-vue-next'
import MultiSelect from 'primevue/multiselect'
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'

import { fetchDefinitionsTypes } from '@/api'

import { defaultFilterLanguageTags } from '@/config/locales';
import { useI18n } from 'vue-i18n'
const { t } = useI18n()

const props = defineProps({
  components: {
    MultiSelect,
    ChevronDown,
  },
  modelValue: {
    type: Object,
    required: true,
    default: () => ({
      selmaho: '',
      username: '',
      isExpanded: false,
      selectedLanguages: [],
      word_type: null,
      source_langid: 1, // Default to Lojban
    }),
  },
  languages: {
    type: Array,
    required: true,
  },
})

const emit = defineEmits(['update:modelValue', 'change', 'reset'])

const selectedLangs = ref([])
const expanded = ref(props.modelValue.isExpanded)
const wordTypes = ref([])
const filters = ref({
  selmaho: props.modelValue.selmaho,
  username: props.modelValue.username,
  word_type: null,
  source_langid: props.modelValue.source_langid || 1, // Initialize from prop or default
})

const showWordType = computed(() => !filters.value.selmaho)

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
      source_langid: newVal.source_langid || 1, // Sync source_langid
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

const fetchWordTypes = async () => {
  try {
    const response = await fetchDefinitionsTypes()
    wordTypes.value = response.data.types
  } catch (error) {
    console.error('Error fetching word types:', error)
  }
}

onMounted(fetchWordTypes)

onBeforeUnmount(() => {
  // Clean up any pending debounce timer
  clearDebounceTimer()
})

const getDefaultLanguages = () => {
  return props.languages.filter((lang) => defaultFilterLanguageTags.includes(lang.tag))
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
    filters.value.source_langid !== 1 || // Check if source_langid is not default
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
    if (filters.value.selmaho === currentFilters.selmaho && 
        filters.value.username === currentFilters.username) {
      emitUpdate()
    }
    debounceTimer = null
  }, DEBOUNCE_DELAY)
}

const handleWordTypeChange = (event) => {
  const selectedType = wordTypes.value.find(
    (type) => type.type_id === event.target.value?.type_id
  )
  if (selectedType) {
    filters.value.word_type = selectedType
  }
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

  const resetValue = {
    selmaho: '',
    username: '',
    isExpanded: false,
    selectedLanguages: defaultLangs.map((lang) => lang.id),
    word_type: null,
    source_langid: 1, // Reset source_langid to default
  }

  // Single emit for both reset and update
  emit('reset')
  emit('update:modelValue', resetValue)
}

const toggleExpanded = () => {
  expanded.value = !expanded.value
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
          selectedLanguages: selectedLangs.value.map(lang => lang.id)
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
