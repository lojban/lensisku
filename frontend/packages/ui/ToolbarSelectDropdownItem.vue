<template>
  <button
    type="button"
    class="flex items-center gap-2 w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
    v-bind="$attrs"
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
defineOptions({ inheritAttrs: false })
</script>
