<script setup lang="ts">
/**
 * Create-definition glyph: Lucide `TextQuote` plus a plus (brand book §5c).
 * The plus uses the same bottom-right corner as Lucide `StarPlus` and does not
 * meet the quote strokes.
 */
defineOptions({ name: 'DefinitionCreateIcon', inheritAttrs: false })
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
    class="lucide lucide-text-quote-plus-icon lucide-text-quote-plus"
    aria-hidden="true"
    v-bind="$attrs"
  >
    <path d="M14 2H3" />
    <path d="M14 6H8" />
    <path d="M14 10H8" />
    <path d="M3 6v5" />
    <path d="M15 18h6" />
    <path d="M18 15v6" />
  </svg>
</template>
