<script setup lang="ts">
/**
 * New wiki article (brand book §5c): the letter W plus a plus.
 * The book outline is omitted so the W stays readable beside the plus.
 */
defineOptions({ name: 'WikiCreateIcon', inheritAttrs: false })
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
    class="lucide lucide-book-w-plus-icon lucide-book-w-plus"
    aria-hidden="true"
    v-bind="$attrs"
  >
    <path d="M2 4.5 5 17 8 9.5 11 17 14 4.5" />
    <path d="M15 18h6" />
    <path d="M18 15v6" />
  </svg>
</template>
