<template>
  <button type="button" class="ui-btn--fab inline-flex items-center justify-center !p-0">
    <slot />
  </button>
</template>

<script setup lang="ts">
defineOptions({ name: 'UiFabButton' })
</script>
