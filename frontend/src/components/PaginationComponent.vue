<template>
  <div
    class="mt-6 flex flex-wrap gap-2 justify-center sm:justify-between items-center px-4 py-3 bg-white border rounded-lg"
  >
    <div class="flex items-center justify-center sm:justify-start flex-1">
      <p
        v-if="paginationStart && paginationEnd && total"
        class="text-sm text-gray-700 whitespace-nowrap"
      >
        {{ t('pagination.showing') }} <span class="font-medium">{{ paginationStart }}</span>
        {{ t('pagination.to') }} <span class="font-medium">{{ paginationEnd }}</span>
        {{ t('pagination.of') }} <span class="font-medium">{{ total }}</span>
        {{ t('pagination.items') }}
      </p>
    </div>

    <div class="flex items-center space-x-2 flex-1 justify-center sm:justify-end">
      <button
        :disabled="currentPage === 1"
        class="ui-btn--back"
        :class="
          currentPage === 1
            ? 'text-gray-400 border-gray-200'
            : 'text-gray-700 border-gray-300 hover:bg-gray-50'
        "
        @click="$emit('prev')"
      >
        {{ t('pagination.previous') }}
      </button>
      <span class="text-sm text-gray-600 whitespace-nowrap">
        {{ t('pagination.page', { currentPage: currentPage, totalPages: totalPages || 1 }) }}
      </span>
      <button
        :disabled="currentPage >= totalPages"
        class="ui-btn--forward"
        :class="
          currentPage >= totalPages
            ? 'text-gray-400 border-gray-200'
            : 'text-gray-700 border-gray-300 hover:bg-gray-50'
        "
        @click="$emit('next')"
      >
        {{ t('pagination.next') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps({
  currentPage: {
    type: Number,
    required: true,
  },
  totalPages: {
    type: Number,
    required: true,
  },
  total: {
    type: Number,
    required: true,
  },
  perPage: {
    type: Number,
    required: true,
  },
})

const paginationStart = computed(() => {
  return (props.currentPage - 1) * props.perPage + 1
})

const paginationEnd = computed(() => {
  return Math.min(props.currentPage * props.perPage, props.total)
})
</script>
