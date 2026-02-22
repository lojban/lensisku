<script setup>
import { Loader2, Search, X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'

const { t } = useI18n()

defineProps({
  modelValue: {
    type: String,
    default: ''
  },
  isLoading: {
    type: Boolean,
    default: false
  },
  placeholder: {
    type: String,
    default: '',
  },
  showSearchIcon: {
    type: Boolean,
    default: false
  }
})

defineEmits(['update:modelValue', 'clear'])
</script>

<template>
  <div class="relative flex-1 min-w-0">
    <Search
      v-if="showSearchIcon"
      class="absolute left-3.5 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400 pointer-events-none"
      aria-hidden="true"
    />
    <input
      :value="modelValue"
      type="text"
      :placeholder="placeholder"
      class="input-field w-full min-w-[200px] transition-colors"
      :class="{
        'pl-10': showSearchIcon,
        'pr-10': modelValue.length > 0
      }"
      @input="$emit('update:modelValue', normalizeSearchQuery($event.target.value))"
    >
    <div class="absolute right-3 top-1/2 -translate-y-1/2 flex items-center gap-1">
      <Loader2
        v-if="isLoading"
        class="h-4 w-4 text-blue-500 animate-spin shrink-0"
        aria-hidden="true"
      />
      <button
        v-else-if="modelValue"
        type="button"
        class="p-0.5 rounded-full text-gray-400 hover:text-gray-600 hover:bg-gray-100 transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-1"
        :aria-label="t('components.searchInput.clearAria')"
        @click="$emit('clear')"
      >
        <X class="h-4 w-4" />
      </button>
    </div>
  </div>
</template>
