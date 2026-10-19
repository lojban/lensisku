<template>
  <button
    type="button"
    class="lingo-course-card"
    :disabled="disabled"
    @click="$emit('select', collection.collection_id)"
  >
    <div class="flex min-h-[24px] w-full items-center justify-end">
      <div v-if="isActive" class="flex items-center justify-center rounded-md bg-green-600 p-1.5">
        <Check class="h-4 w-4 stroke-[3] text-white" />
      </div>
    </div>

    <div
      class="flex h-[70px] w-[93px] shrink-0 items-center justify-center rounded-lg border border-neutral-200 bg-neutral-50 drop-shadow-md"
    >
      <PlayingCardsFanIcon class="h-9 w-9 text-green-600" />
    </div>

    <p class="mt-3 text-center font-bold text-neutral-700 line-clamp-2">{{ collection.name }}</p>
  </button>
</template>

<script setup lang="ts">
import { PlayingCardsFanIcon } from '@packages/ui'
import { Check } from '@lucide/vue'

defineProps({
  collection: { type: Object, required: true },
  isActive: { type: Boolean, default: false },
  disabled: { type: Boolean, default: false },
})

defineEmits(['select'])
</script>
