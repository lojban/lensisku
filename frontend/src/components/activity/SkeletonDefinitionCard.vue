<template>
  <!-- Mirrors DefinitionCard: border + shadow shell and p-4 inner rhythm. -->
  <div class="skeleton-definition-card skeleton-pulse" aria-hidden="true">
    <div class="p-4">
      <div class="flex flex-col items-start gap-4 sm:flex-row sm:justify-between">
        <div class="w-full min-w-0 flex-1 space-y-3">
          <div class="flex w-full flex-col gap-2">
            <div class="flex flex-wrap items-center justify-between gap-2">
              <div class="skeleton-bone h-6 w-36 sm:h-7 sm:w-44" />
              <div class="flex shrink-0 items-center gap-2">
                <div class="skeleton-bone h-8 w-8 rounded-md" />
                <div class="skeleton-bone h-8 w-8 rounded-md" />
              </div>
            </div>

            <div class="flex flex-wrap items-center gap-2">
              <div class="skeleton-bone h-6 w-20 rounded-full" />
              <div class="skeleton-bone--soft h-5 w-24 rounded-full" />
            </div>
          </div>

          <div class="space-y-2 pt-1">
            <div class="skeleton-bone h-4 w-full" />
            <div class="skeleton-bone h-4 w-[92%]" />
            <div class="skeleton-bone--soft h-4 w-3/4" />
          </div>

          <div class="mt-2 flex flex-wrap gap-2 pt-1">
            <div class="skeleton-bone h-8 w-20 rounded-md" />
            <div class="skeleton-bone h-8 w-24 rounded-md" />
            <div class="skeleton-bone--soft h-8 w-16 rounded-md" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// Layout-only placeholder; no logic.
</script>
