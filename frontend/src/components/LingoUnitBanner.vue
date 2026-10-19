<template>
  <div class="lingo-unit-banner">
    <div class="min-w-0 space-y-2.5">
      <h3 class="text-2xl font-bold">{{ title }}</h3>
      <p v-if="description" class="text-lg opacity-95">{{ description }}</p>
    </div>
    <button
      type="button"
      class="lingo-btn lingo-btn--secondary lingo-btn--lg hidden shrink-0 border-white/40 xl:inline-flex"
      @click="$emit('continue')"
    >
      <BookOpen class="h-5 w-5 shrink-0" />
      {{ continueLabel }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { BookOpen } from '@lucide/vue'

defineProps({
  title: { type: String, default: '' },
  description: { type: String, default: '' },
  continueLabel: { type: String, default: '' },
})

defineEmits(['continue'])
</script>
