<template>
  <div class="pagination-bar" :class="{ 'pagination-bar--end': !showSummary }">
    <p v-if="showSummary" class="pagination-bar__summary">
      {{ t('pagination.showing') }} <span class="font-medium">{{ paginationStart }}</span>
      {{ t('pagination.to') }} <span class="font-medium">{{ paginationEnd }}</span>
      {{ t('pagination.of') }} <span class="font-medium">{{ total }}</span>
      {{ t('pagination.items') }}
    </p>

    <div class="pagination-bar__controls">
      <Button
        variant="neutral"
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
      </Button>
      <span class="text-sm text-gray-600 whitespace-nowrap">
        {{ t('pagination.page', { currentPage: currentPage, totalPages: totalPages || 1 }) }}
      </span>
      <Button
        variant="neutral"
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
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

defineEmits(['prev', 'next'])

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

const showSummary = computed(
  () => !!(paginationStart.value && paginationEnd.value && props.total)
)
</script>
