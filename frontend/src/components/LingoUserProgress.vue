<template>
  <div class="flex w-auto shrink-0 items-center justify-end gap-x-2">
    <span class="inline-flex items-center gap-x-1 text-sm font-bold text-rose-500">
      <Heart class="h-5 w-5 shrink-0 fill-rose-500 text-rose-500" aria-hidden="true" />
      <InfinityIcon v-if="hasUnlimitedHearts" class="h-4 w-4 stroke-[3]" aria-hidden="true" />
      <span v-else>{{ hearts }}</span>
    </span>
  </div>
</template>

<script setup lang="ts">
import { Heart, Infinity as InfinityIcon } from '@lucide/vue'
import { useLingoProgress } from '@/composables/useLingoProgress'

const { hearts, hasUnlimitedHearts } = useLingoProgress()
</script>
