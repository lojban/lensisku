<template>
  <div v-if="!isPro" class="space-y-4 rounded-xl border-2 border-slate-200 bg-white p-4">
    <div class="space-y-2">
      <div class="flex items-center gap-x-2">
        <Sparkles class="h-6 w-6 text-indigo-500" />
        <h3 class="text-lg font-bold text-neutral-800">{{ t('lingo.upgradeToPro') }}</h3>
      </div>
      <p class="text-sm text-slate-500">{{ t('lingo.upgradeDescription') }}</p>
    </div>
    <RouterLink
      to="/lingo/shop"
      class="lingo-btn lingo-btn--primary lingo-btn--lg w-full !normal-case !tracking-normal"
    >
      {{ t('lingo.upgradeToday') }}
    </RouterLink>
  </div>
</template>

<script setup lang="ts">
import { Sparkles } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import { useLingoProgress } from '@/composables/useLingoProgress'

const { t } = useI18n()
const { isPro } = useLingoProgress()
</script>
