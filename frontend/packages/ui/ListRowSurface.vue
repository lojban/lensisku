<template>
  <button type="button" class="surface-list-row w-full text-left" @click="$emit('click', $event)">
    <slot />
  </button>
</template>

<script setup lang="ts">
defineEmits<{ click: [e: MouseEvent] }>()
</script>
