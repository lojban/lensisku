<template>
  <div class="flex w-auto shrink-0 items-center justify-end gap-x-2">
    <RouterLink to="/lingo/shop" class="lingo-btn lingo-btn--ghost lingo-btn--sm text-rose-500">
      <Heart class="h-5 w-5 shrink-0 fill-rose-500 text-rose-500" />
      <InfinityIcon v-if="hasUnlimitedHearts" class="h-4 w-4 stroke-[3]" />
      <span v-else>{{ hearts }}</span>
    </RouterLink>
  </div>
</template>

<script setup lang="ts">
import { Heart, Infinity as InfinityIcon } from '@lucide/vue'
import { useLingoProgress } from '@/composables/useLingoProgress'

const { hearts, hasUnlimitedHearts } = useLingoProgress()
</script>
