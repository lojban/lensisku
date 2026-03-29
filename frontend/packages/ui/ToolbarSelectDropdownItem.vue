<template>
  <button
    type="button"
    class="block w-full whitespace-nowrap px-3 py-2 text-left text-sm hover:bg-gray-50"
    v-bind="$attrs"
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
defineOptions({ inheritAttrs: false })
</script>
