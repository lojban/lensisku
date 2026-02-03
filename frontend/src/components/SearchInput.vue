<script setup>
import { Loader2, X } from 'lucide-vue-next'
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
  }
})

defineEmits(['update:modelValue', 'clear'])
</script>

<template>
  <div class="relative flex-1">
    <input
      :value="modelValue"
      type="text"
      :placeholder="placeholder"
      class="input-field w-full min-w-[200px]"
      :class="{ 'pr-10': modelValue.length > 0 }"
      @input="$emit('update:modelValue', normalizeSearchQuery($event.target.value))"
    >
    <div class="absolute right-3 top-1/2 transform -translate-y-1/2 flex items-center">
      <Loader2
        v-if="isLoading"
        class="h-4 w-4 text-gray-500 animate-spin"
      />
      <button
        v-else-if="modelValue"
        class="text-gray-400 hover:text-gray-600"
        @click="$emit('clear')"
      >
        <X class="h-5 w-5" />
      </button>
    </div>
  </div>
</template>
