<template>
  <LingoLayout>
    <LingoFeedLayout>
      <template #aside>
        <LingoPromo />
      </template>

      <div class="flex w-full flex-col items-center">
        <div class="flex h-20 w-20 items-center justify-center rounded-2xl bg-green-100">
          <Target class="h-10 w-10 text-green-600" />
        </div>

        <h1 class="my-6 text-center text-2xl font-bold text-neutral-800">
          {{ t('lingo.quests') }}
        </h1>
        <p class="mb-6 text-center text-lg text-slate-500">{{ t('lingo.questsDescription') }}</p>

        <ul class="w-full max-w-lg">
          <li
            v-for="quest in LINGO_QUESTS"
            :key="quest.title"
            class="flex w-full items-center gap-x-4 border-t-2 border-slate-100 p-4"
          >
            <div
              class="flex h-14 w-14 shrink-0 items-center justify-center rounded-lg bg-orange-100"
            >
              <Sparkles class="h-7 w-7 text-orange-500" />
            </div>
            <div class="flex min-w-0 w-full flex-col gap-y-2">
              <p class="text-xl font-bold text-neutral-700">{{ quest.title }}</p>
              <div class="h-3 w-full overflow-hidden rounded-full bg-slate-200">
                <div
                  class="h-full rounded-full bg-green-500 transition-all"
                  :style="{ width: `${Math.min(100, (points / quest.value) * 100)}%` }"
                />
              </div>
              <p class="text-xs text-slate-500">
                {{ Math.min(points, quest.value) }} / {{ quest.value }} XP
              </p>
            </div>
          </li>
        </ul>
      </div>
    </LingoFeedLayout>
  </LingoLayout>
</template>

<script setup lang="ts">
import { Target, Sparkles } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import LingoFeedLayout from '@/components/LingoFeedLayout.vue'
import LingoPromo from '@/components/LingoPromo.vue'
import { LINGO_QUESTS } from '@/config/lingoConstants'
import { useLingoProgress } from '@/composables/useLingoProgress'
import { useSeoHead } from '@/composables/useSeoHead'

const { t } = useI18n()
const { points } = useLingoProgress()

useSeoHead({ title: t('lingo.quests') })
</script>
