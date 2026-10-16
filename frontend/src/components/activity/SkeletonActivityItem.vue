<template>
  <!-- Mirror CommentItem / RecentChangeItem chrome so feed CLS stays low. -->
  <div
    class="comment-item bg-white border rounded-lg p-3 my-2 skeleton-pulse min-w-0 max-w-full"
    aria-hidden="true"
  >
    <div class="mb-2 flex items-center gap-1.5">
      <div class="skeleton-bone h-5 w-16 rounded-full" />
      <div class="skeleton-bone--soft h-4 w-1/3 max-w-[12rem]" />
    </div>
    <div class="flex flex-col mb-2 min-w-0">
      <div class="comment-header">
        <div
          class="w-8 h-8 sm:w-10 sm:h-10 shrink-0 rounded-full bg-gray-200 border-2 border-white shadow-sm"
        />
        <div class="comment-header-main space-y-1">
          <div class="skeleton-bone h-4 w-24 max-w-full" />
          <div class="skeleton-bone--soft h-3 w-16" />
        </div>
        <div class="skeleton-bone--soft h-4 w-8 shrink-0" />
      </div>
      <div class="mt-2 space-y-2">
        <div class="skeleton-bone h-4 w-2/3 max-w-md" />
        <div class="skeleton-bone h-4 w-full" />
        <div class="skeleton-bone--soft h-4 w-5/6" />
      </div>
    </div>
    <div class="flex justify-end gap-2 pt-1">
      <div class="skeleton-bone h-6 w-14 rounded-full" />
      <div class="skeleton-bone h-6 w-14 rounded-full" />
    </div>
  </div>
</template>

<script setup lang="ts">
// Shared activity / comment row skeleton (home feed, Me, contributions).
</script>
