<template>
  <article class="card-base" :class="[elevated && 'card-elevated', compact && 'card-compact']">
    <div v-if="$slots.media" class="overflow-hidden flex-shrink-0"><slot name="media" /></div>

    <div class="card-body">
      <div v-if="$slots.header" class="card-header-wrap"><slot name="header" /></div>

      <div class="card-footer-spacer" />

      <div v-if="$slots.footer" class="card-footer"><slot name="footer" /></div>
    </div>
  </article>
</template>

<script setup lang="ts">
defineProps({
  elevated: { type: Boolean, default: true },
  compact: { type: Boolean, default: false },
})

defineOptions({ name: 'UiCard' })
</script>
