<script setup lang="ts">
/**
 * One wiki article (brand book §5c): the letter W.
 * A set of articles is the book-W (`WikisIcon`). Creating one is `WikiCreateIcon`.
 */
defineOptions({ name: 'WikiIcon', inheritAttrs: false })
</script>

<template>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    class="lucide lucide-w-icon lucide-w"
    aria-hidden="true"
    v-bind="$attrs"
  >
    <path d="M3 3.5 7.5 20.5 12 8.5 16.5 20.5 21 3.5" />
  </svg>
</template>
