<template>

  <div
    v-if="!hide"
    class="lingo-banner fixed left-0 right-0 top-0 z-50 flex items-center justify-center bg-slate-100 px-10 py-2.5 text-sm shadow-md"
  >

    <p class="text-center text-slate-700"> <slot /> </p>
     <button
      type="button"
      class="absolute right-2 top-1/2 -translate-y-1/2 rounded-full border border-slate-300 bg-white p-1 opacity-80 hover:opacity-100"
      aria-label="Close banner"
      @click="$emit('close')"
    >
       <X class="h-3 w-3" /> </button
    >
  </div>

</template>

<script setup lang="ts">
import { X } from 'lucide-vue-next'

defineProps({
  hide: { type: Boolean, default: false },
})

defineEmits(['close'])
</script>

