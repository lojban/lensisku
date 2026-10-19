<script setup lang="ts">
/**
 * Several wiki articles (brand book §5c): the book with a W. One article is the letter W (`WikiIcon`).
 */
defineOptions({ name: 'WikisIcon', inheritAttrs: false })
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
    class="lucide lucide-books-w-icon lucide-books-w"
    aria-hidden="true"
    v-bind="$attrs"
  >
    <path
      d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H19a1 1 0 0 1 1 1v18a1 1 0 0 1-1 1H6.5a1 1 0 0 1 0-5H20"
    />
    <path d="m8 6 2 7 2-4 2 4 2-7" />
  </svg>
</template>
