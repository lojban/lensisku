<template>
  <!-- Mirrors ActivityThreads `.surface-activity-row` spacing and chrome. -->
  <div class="skeleton-activity-row skeleton-pulse" aria-hidden="true">
    <div class="mb-2 flex flex-wrap items-center gap-2">
      <div class="skeleton-bone h-5 w-14 rounded-full" />
      <div class="skeleton-bone h-5 w-2/5 max-w-[14rem]" />
      <div class="skeleton-bone--soft h-4 w-24" />
    </div>

    <div class="border-b pb-2">
      <div class="skeleton-bone--soft h-4 w-28" />
    </div>

    <div class="space-y-2 text-sm">
      <div class="flex items-center gap-2">
        <div class="skeleton-bone--soft h-3 w-20" />
        <div class="skeleton-bone--soft h-3 w-16" />
      </div>
      <div class="space-y-1.5 border-l-2 border-gray-200 pl-2">
        <div class="skeleton-bone h-3.5 w-full" />
        <div class="skeleton-bone h-3.5 w-5/6" />
        <div class="skeleton-bone--soft h-3.5 w-2/3" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// Layout-only placeholder; no logic.
</script>
