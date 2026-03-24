<template>
  <button
    type="button"
    class="lingo-course-card flex h-full min-h-[217px] min-w-0 cursor-pointer flex-col items-center justify-between rounded-xl border-2 border-b-[4px] border-neutral-200 p-3 pb-6 transition hover:bg-black/5 active:border-b-2 lg:min-w-[200px]"
    :class="{ 'pointer-events-none opacity-50': disabled }"
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
      <BookOpen class="h-9 w-9 text-green-600" />
    </div>

    <p class="mt-3 text-center font-bold text-neutral-700 line-clamp-2">
      {{ collection.name }}
    </p>
  </button>
</template>

<script setup>
import { Check, BookOpen } from 'lucide-vue-next'

defineProps({
  collection: { type: Object, required: true },
  isActive: { type: Boolean, default: false },
  disabled: { type: Boolean, default: false },
})

defineEmits(['select'])
</script>
