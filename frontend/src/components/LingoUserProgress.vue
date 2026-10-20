<template>
  <div class="flex w-full items-center justify-between gap-x-2">
    <RouterLink
      to="/lingo/courses"
      class="lingo-btn lingo-btn--ghost lingo-btn--icon shrink-0"
      :aria-label="t('lingo.courses')"
    >
      <PlayingCardsFanIcon class="h-7 w-7 text-green-600" />
    </RouterLink>

    <RouterLink to="/lingo/shop" class="lingo-btn lingo-btn--ghost lingo-btn--sm text-orange-500">
      <Sparkles class="h-5 w-5 shrink-0" />
      <span>{{ points }}</span>
    </RouterLink>

    <RouterLink to="/lingo/shop" class="lingo-btn lingo-btn--ghost lingo-btn--sm text-rose-500">
      <Heart class="h-5 w-5 shrink-0 fill-rose-500 text-rose-500" />
      <InfinityIcon v-if="hasUnlimitedHearts" class="h-4 w-4 stroke-[3]" />
      <span v-else>{{ hearts }}</span>
    </RouterLink>
  </div>
</template>

<script setup lang="ts">
import { Heart, Sparkles, Infinity as InfinityIcon } from '@lucide/vue'
import { PlayingCardsFanIcon } from '@packages/ui'
import { useI18n } from 'vue-i18n'
import { useLingoProgress } from '@/composables/useLingoProgress'

const { t } = useI18n()
const { hearts, points, hasUnlimitedHearts } = useLingoProgress()
</script>
