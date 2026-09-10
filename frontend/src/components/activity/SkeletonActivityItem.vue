<template>
  <div class="skeleton-activity-row skeleton-pulse" aria-hidden="true">
    <div class="flex items-start space-x-3">
      <div class="h-10 w-10 flex-shrink-0 rounded-full bg-gray-200" />

      <div class="flex-1 space-y-3">
        <div class="flex items-center justify-between">
          <div class="skeleton-bone h-4 w-1/4" />
          <div class="skeleton-bone--soft h-3 w-1/6" />
        </div>
        <div class="space-y-2">
          <div class="skeleton-bone h-4 w-full" />
          <div class="skeleton-bone h-4 w-5/6" />
          <div class="skeleton-bone--soft h-4 w-3/4" />
        </div>
        <div class="flex justify-end space-x-2 pt-2">
          <div class="skeleton-bone h-6 w-16 rounded-full" />
          <div class="skeleton-bone h-6 w-16 rounded-full" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// Shared fallback activity skeleton (Me / contributions pages).
</script>
