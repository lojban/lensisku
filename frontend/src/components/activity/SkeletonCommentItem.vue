<template>
  <!-- Mirrors CommentItem / RecentChangeItem: same chrome, padding, and my-2 margins. -->
  <div class="skeleton-comment-item skeleton-pulse" aria-hidden="true">
    <div v-if="showContext" class="mb-2 flex items-center gap-1.5">
      <div class="skeleton-bone h-5 w-16 shrink-0 rounded-full" />
      <div class="skeleton-bone h-4 w-2/5 max-w-[12rem]" />
    </div>

    <div class="comment-header mb-2">
      <div
        class="h-8 w-8 sm:h-10 sm:w-10 shrink-0 rounded-full border-2 border-white bg-gray-200 shadow-sm"
      />
      <div class="comment-header-main space-y-1.5">
        <div class="skeleton-bone h-4 w-24 sm:w-28" />
        <div class="skeleton-bone--soft h-3 w-16" />
      </div>
      <div class="skeleton-bone--soft h-4 w-8 shrink-0" />
    </div>

    <div class="mt-2 space-y-2">
      <div class="skeleton-bone h-4 w-full" />
      <div class="skeleton-bone h-4 w-11/12" />
      <div class="skeleton-bone--soft h-4 w-4/5" />
    </div>

    <div class="mt-3 flex justify-end gap-2">
      <div class="skeleton-bone h-7 w-14 rounded-full" />
      <div class="skeleton-bone h-7 w-14 rounded-full" />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps({
  /** Match ActivityComments / news comment rows that show definition/valsi context. */
  showContext: {
    type: Boolean,
    default: true,
  },
})
</script>
