<template>
  <!-- Shared page/section placeholder. Shapes mirror the real layouts so content swaps in without
       layout shift; `animate-pulse` only (no spinner) and announced once via sr-only text. -->
  <div
    class="page-skeleton"
    role="status"
    aria-busy="true"
    aria-live="polite"
    :data-layout="layout"
  >
    <span class="sr-only">{{ t('components.loadingSpinner.loadingDefault') }}</span>

    <!-- Card list (collection items, flashcards, course levels, search results) -->
    <div v-if="layout === 'list'" class="flex flex-col gap-3" aria-hidden="true">
      <div v-for="n in rows" :key="n" class="page-skeleton__card">
        <div class="flex items-center gap-3">
          <span class="skeleton-block h-5 w-2/5" />
          <span class="skeleton-block ml-auto h-7 w-7 rounded-full" />
        </div>
        <span class="skeleton-block mt-3 h-3.5 w-full" />
        <span class="skeleton-block mt-2 h-3.5 w-3/4" />
      </div>
    </div>

    <!-- Card grid (collections, library, courses) -->
    <div v-else-if="layout === 'grid'" class="collections-grid" aria-hidden="true">
      <div v-for="n in rows" :key="n" class="page-skeleton__card flex gap-3">
        <span class="skeleton-block h-16 w-16 shrink-0 rounded-lg" />
        <div class="flex min-w-0 flex-1 flex-col gap-2">
          <span class="skeleton-block h-5 w-3/5" />
          <span class="skeleton-block h-3.5 w-full" />
          <span class="skeleton-block h-3.5 w-2/3" />
        </div>
      </div>
    </div>

    <!-- User profile -->
    <div v-else-if="layout === 'profile'" class="flex flex-col gap-6" aria-hidden="true">
      <span class="skeleton-block mx-auto h-7 w-56 sm:mx-0" />
      <div class="page-skeleton__card flex flex-col items-center gap-4 sm:flex-row">
        <span class="skeleton-block h-24 w-24 shrink-0 rounded-full" />
        <div class="flex w-full flex-col gap-2">
          <span class="skeleton-block h-5 w-1/3" />
          <span class="skeleton-block h-3.5 w-full" />
          <span class="skeleton-block h-3.5 w-5/6" />
        </div>
      </div>
      <div v-for="n in 3" :key="n" class="page-skeleton__card">
        <span class="skeleton-block h-4 w-1/4" />
        <span class="skeleton-block mt-3 h-3.5 w-full" />
      </div>
    </div>

    <!-- Dictionary entry (valsi page) -->
    <div v-else-if="layout === 'entry'" class="flex flex-col gap-3" aria-hidden="true">
      <div class="flex items-center gap-3">
        <span class="skeleton-block h-9 w-40" />
        <span class="skeleton-block h-6 w-20 rounded-full" />
      </div>
      <span class="skeleton-block h-4 w-1/2" />
      <div v-for="n in rows" :key="n" class="page-skeleton__card">
        <span class="skeleton-block h-4 w-1/4" />
        <span class="skeleton-block mt-3 h-3.5 w-full" />
        <span class="skeleton-block mt-2 h-3.5 w-11/12" />
        <span class="skeleton-block mt-2 h-3.5 w-2/3" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

withDefaults(
  defineProps<{
    layout?: 'list' | 'grid' | 'profile' | 'entry'
    rows?: number
  }>(),
  { layout: 'list', rows: 4 }
)

const { t } = useI18n()
</script>
