<template>
  <LingoLayout>
    <div class="flex w-full flex-col items-center pb-10">
      <div class="flex h-20 w-20 items-center justify-center rounded-2xl bg-green-100">
        <Trophy class="h-10 w-10 text-green-600" />
      </div>

      <h1 class="my-6 text-center text-2xl font-bold text-slate-800">
        {{ t('lingo.leaderboard') }}
      </h1>
      <p class="mb-6 text-center text-slate-600">
        {{ t('lingo.leaderboardDescription') }}
      </p>

      <div
        class="w-full max-w-md rounded-xl border-2 border-slate-200 bg-white p-4 text-center text-slate-500"
      >
        {{ t('lingo.comingSoon') }}
      </div>
    </div>
  </LingoLayout>
</template>

<script setup>
import { Trophy } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import { useSeoHead } from '@/composables/useSeoHead'

const { t, locale } = useI18n()
useSeoHead({ title: t('lingo.leaderboard') }, locale.value)
</script>
