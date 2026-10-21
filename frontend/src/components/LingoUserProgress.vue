<template>
  <div
    class="lingo-hearts-badge"
    role="status"
    :aria-label="
      hasUnlimitedHearts ? t('lingo.unlimitedHearts') : `${hearts} · ${t('lingo.heartsLeft')}`
    "
  >
    <Heart class="h-5 w-5 shrink-0 fill-rose-500 text-rose-500" aria-hidden="true" />
    <InfinityIcon
      v-if="hasUnlimitedHearts"
      class="h-4 w-4 shrink-0 stroke-[3]"
      aria-hidden="true"
    />
    <span v-else aria-hidden="true">{{ hearts }}</span>
  </div>
</template>

<script setup lang="ts">
import { Heart, Infinity as InfinityIcon } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import { useLingoProgress } from '@/composables/useLingoProgress'

const { t } = useI18n()
const { hearts, hasUnlimitedHearts } = useLingoProgress()
</script>
