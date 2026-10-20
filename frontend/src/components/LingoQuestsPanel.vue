<template>
  <div class="space-y-4 rounded-xl border-2 border-slate-200 bg-white p-4">
    <div class="flex w-full items-center justify-between gap-2">
      <h3 class="text-lg font-bold text-neutral-800">{{ t('lingo.quests') }}</h3>
      <RouterLink
        v-if="!hideViewAll"
        to="/lingo/quests"
        class="lingo-btn lingo-btn--primary lingo-btn--sm !normal-case !tracking-normal"
      >
        {{ t('lingo.viewAll') }}
      </RouterLink>
    </div>

    <ul class="w-full space-y-4">
      <li
        v-for="quest in visibleQuests"
        :key="quest.title"
        class="flex w-full items-center gap-x-3 pb-2"
      >
        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-orange-100">
          <Sparkles class="h-5 w-5 text-orange-500" />
        </div>
        <div class="flex min-w-0 w-full flex-col gap-y-2">
          <p class="text-sm font-bold text-neutral-700">{{ quest.title }}</p>
          <div class="h-2 w-full overflow-hidden rounded-full bg-slate-200">
            <div
              class="h-full rounded-full bg-green-500 transition-all"
              :style="{ width: `${Math.min(100, (points / quest.value) * 100)}%` }"
            />
          </div>
        </div>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Sparkles } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import { LINGO_QUESTS } from '@/config/lingoConstants'
import { useLingoProgress } from '@/composables/useLingoProgress'

const props = defineProps({
  /** Show every quest (quests page) vs first three (sidebar). */
  all: { type: Boolean, default: false },
  hideViewAll: { type: Boolean, default: false },
})

const { t } = useI18n()
const { points } = useLingoProgress()

const visibleQuests = computed(() => (props.all ? [...LINGO_QUESTS] : LINGO_QUESTS.slice(0, 3)))
</script>
