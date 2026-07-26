<template>
  <div class="search-form max-w-3xl mx-auto w-full animate-pulse" aria-hidden="true">
    <div class="relative flex w-full min-w-0">
      <!-- Search Input Skeleton -->
      <div class="flex-1">
        <div
          class="input-field h-10 w-full rounded-r-none bg-gray-200 border-gray-200 shadow-none"
        />
      </div>
      <!-- Mode Selector Skeleton -->
      <div class="shrink-0">
        <div
          class="dropdown-trigger dropdown-trigger--search-bar-trailing w-11 sm:w-auto max-w-none rounded-l-none rounded-r-full pointer-events-none bg-gray-200 border-gray-200 text-transparent"
        >
          <div class="flex items-center gap-2 min-w-0">
            <div class="h-4 w-4 shrink-0 rounded-full bg-gray-300" />
            <span class="hidden sm:inline-block h-4 w-28 bg-gray-300 rounded" />
          </div>
          <div class="h-4 w-4 shrink-0 rounded-full bg-gray-300" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// No script needed for a simple skeleton
</script>
