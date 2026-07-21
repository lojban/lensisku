<template>
  <div class="search-form max-w-3xl mx-auto animate-pulse" aria-hidden="true">
    <div class="flex flex-row gap-0 items-stretch">
      <!-- Mode Selector Skeleton -->
      <div class="relative z-10 w-auto shrink-0">
        <div
          class="dropdown-trigger dropdown-trigger--search-bar-leading w-16 sm:w-60 pointer-events-none bg-gray-200 border-gray-200 text-transparent"
        >
          <div class="flex items-center gap-2 min-w-0">
            <div class="h-4 w-4 shrink-0 rounded-full bg-gray-300" />
            <span class="hidden sm:inline-block h-4 w-20 bg-gray-300 rounded" />
          </div>
          <div class="h-4 w-4 shrink-0 rounded-full bg-gray-300" />
        </div>
      </div>
      <!-- Search Input Skeleton -->
      <div class="search-form-query-col">
        <div
          class="input-field h-10 w-full rounded-l-none bg-gray-200 border-gray-200 shadow-none"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// No script needed for a simple skeleton
</script>
