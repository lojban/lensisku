<script setup lang="ts">
/**
 * New discussion / wave / thread (brand book §5c): a plus above two wave lines.
 * A new comment stays Lucide `MessageSquarePlus`.
 */
defineOptions({ name: 'WaveCreateIcon', inheritAttrs: false })
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
    class="lucide lucide-wave-plus-icon lucide-wave-plus"
    aria-hidden="true"
    v-bind="$attrs"
  >
    <path d="M12 2v6" />
    <path d="M9 5h6" />
    <path d="M2 14c1.7-1.7 3.3-1.7 5 0s3.3 1.7 5 0 3.3-1.7 5 0 3.3 1.7 5 0" />
    <path d="M2 20c1.7-1.7 3.3-1.7 5 0s3.3 1.7 5 0 3.3-1.7 5 0 3.3 1.7 5 0" />
  </svg>
</template>
