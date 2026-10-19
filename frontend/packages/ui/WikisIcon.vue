<script setup lang="ts">
/**
 * Wiki pages, the wiki book, and wiki as a source (brand book §5c): stacked books with a W.
 * One article stays the letter W (`WikiIcon`). Creating a page is `WikiCreateIcon`.
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
    <path d="M5 6h9a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2z" />
    <path d="M7 6V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2h-3" />
    <path d="M6 11l1.75 6 1.75-3.5 1.75 3.5L13 11" />
  </svg>
</template>
