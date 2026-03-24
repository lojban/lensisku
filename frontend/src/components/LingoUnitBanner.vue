<template>
  <div
    class="lingo-unit-banner flex w-full items-center justify-between rounded-xl bg-green-500 p-5 text-white"
  >
    <div class="space-y-2.5">
      <h3 class="text-2xl font-bold">{{ title }}</h3>
      <p v-if="description" class="text-lg opacity-95">{{ description }}</p>
    </div>
    <button
      type="button"
      class="btn-aqua-white hidden border-2 border-b-4 border-white/30 active:border-b-2 xl:inline-flex"
      @click="$emit('continue')"
    >
      <BookOpen class="mr-2 h-5 w-5" />
      {{ continueLabel }}
    </button>
  </div>
</template>

<script setup>
import { BookOpen } from 'lucide-vue-next'

defineProps({
  title: { type: String, default: '' },
  description: { type: String, default: '' },
  continueLabel: { type: String, default: '' },
})

defineEmits(['continue'])
</script>
