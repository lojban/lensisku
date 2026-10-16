<template>
  <!-- Same bounding box as LiveChatCard while the async chunk / socket are deferred. -->
  <article
    class="bg-white border rounded-lg p-3 min-w-48"
    :aria-label="t('footer.liveChat')"
    aria-busy="true"
  >
    <div class="flex flex-wrap items-center justify-between gap-2 min-h-8">
      <span
        class="inline-block px-2 py-1 text-xs font-medium rounded-full bg-indigo-100 text-indigo-800"
      >
        {{ t('footer.liveChat') }}
      </span>
      <div class="flex items-center gap-3" aria-hidden="true">
        <div class="skeleton-bone h-6 w-6 rounded" />
        <div class="skeleton-bone h-6 w-6 rounded" />
        <div class="skeleton-bone h-6 w-6 rounded" />
      </div>
    </div>
    <div
      class="mt-2 w-full min-h-[7.5rem] max-h-[7.5rem] space-y-2 overflow-hidden rounded bg-gray-50 p-2 skeleton-pulse"
      aria-hidden="true"
    >
      <div class="skeleton-bone h-3.5 w-4/5" />
      <div class="skeleton-bone h-3.5 w-3/5" />
      <div class="skeleton-bone--soft h-3.5 w-2/3" />
      <div class="skeleton-bone h-3.5 w-1/2" />
    </div>
  </article>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>
