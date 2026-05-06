<template>
  <LingoLayout>
    <div class="flex w-full flex-col items-center pb-10">
      <div class="flex h-20 w-20 items-center justify-center rounded-2xl bg-green-100">
        <Target class="h-10 w-10 text-green-600" />
      </div>

      <h1 class="my-6 text-center text-2xl font-bold text-slate-800">{{ t('lingo.quests') }}</h1>

      <p class="mb-6 text-center text-slate-600">{{ t('lingo.questsDescription') }}</p>

      <ul class="w-full max-w-lg space-y-0 border-t-2 border-slate-200">
        <li
          v-for="quest in LINGO_QUESTS"
          :key="quest.title"
          class="flex w-full items-center gap-4 border-t-2 border-slate-100 p-4 first:border-t-0"
        >
          <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-amber-100">
            <Sparkles class="h-6 w-6 text-amber-600" />
          </div>

          <div class="min-w-0 flex-1">
            <p class="font-bold text-slate-700">{{ quest.title }}</p>

            <div class="mt-2 h-2 w-full overflow-hidden rounded-full bg-slate-200">
              <div
                class="h-full rounded-full bg-green-500 transition-all"
                :style="{ width: `${Math.min(100, (lingoPoints / quest.value) * 100)}%` }"
              />
            </div>
          </div>
        </li>
      </ul>
    </div>
  </LingoLayout>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Target, Sparkles } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import { LINGO_QUESTS } from '@/config/lingoConstants'
import { useSeoHead } from '@/composables/useSeoHead'

const { t } = useI18n()

const lingoPoints = ref(0)

onMounted(() => {
  try {
    const stored = parseInt(sessionStorage.getItem('lingo_points') || '0', 10)
    lingoPoints.value = Number.isNaN(stored) ? 0 : stored
  } catch {}
})

useSeoHead({ title: t('lingo.quests') })
</script>
