<template>
  <!-- Mirror CombinedFilters.vue collapsed bar (`filters-bar-row`) for matching bounding box. -->
  <div class="filters space-y-4" aria-hidden="true">
    <div class="filters-bar-row">
      <div
        class="filters-primary-selects"
        :class="{ '!grid-cols-1 sm:!w-80 sm:!flex-none': !showCollectionFilter }"
      >
        <div
          class="dropdown-trigger pointer-events-none bg-gray-100 border-gray-200 w-full min-w-0"
        >
          <span class="min-w-0 flex-1 truncate invisible select-none">{{
            t('filters.selectLanguages')
          }}</span>
          <span class="h-4 w-4 shrink-0 rounded bg-gray-300" />
        </div>
        <div
          v-if="showCollectionFilter"
          class="dropdown-trigger pointer-events-none bg-gray-100 border-gray-200 w-full min-w-0"
        >
          <span class="min-w-0 flex-1 truncate invisible select-none">{{
            t('filters.selectCollectionsAndUsers')
          }}</span>
          <span class="h-4 w-4 shrink-0 rounded bg-gray-300" />
        </div>
      </div>
      <div class="flex items-center gap-2 self-end md:self-center">
        <div class="btn-group-forced flex items-center flex-row" role="presentation">
          <div
            class="inline-flex h-6 items-center gap-1.5 rounded-full border border-gray-200 bg-gray-100 px-3 pointer-events-none"
          >
            <span class="h-4 w-4 shrink-0 rounded bg-gray-300" />
            <span class="invisible select-none text-xs whitespace-nowrap">{{
              t('searchForm.modes.semantic')
            }}</span>
          </div>
        </div>
        <div class="btn-group-forced flex items-center flex-row" role="presentation">
          <div
            class="inline-flex h-6 items-center rounded-full border border-gray-200 bg-gray-100 px-3 pointer-events-none"
          >
            <span class="invisible select-none text-xs whitespace-nowrap">{{
              t('filters.resetAllFilters')
            }}</span>
          </div>
          <div
            class="inline-flex h-6 w-8 items-center justify-center rounded-full border border-gray-200 bg-gray-100 pointer-events-none -ml-px"
          >
            <span class="h-5 w-5 shrink-0 rounded bg-gray-300" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

withDefaults(
  defineProps<{
    /** Match CombinedFilters `showCollectionFilter` (collection detail uses false). */
    showCollectionFilter?: boolean
  }>(),
  { showCollectionFilter: true }
)

const { t } = useI18n()
</script>
