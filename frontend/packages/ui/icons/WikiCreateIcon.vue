<script setup lang="ts">
/**
 * New wiki page and “into wiki” (brand book §5c): a book, a plus, and a W.
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
    <path d="M12 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-7" />
    <path d="M18 3v6" />
    <path d="M15 6h6" />
    <path d="M6 10l2 7 2-4 2 4 2-7" />
  </svg>
</template>
