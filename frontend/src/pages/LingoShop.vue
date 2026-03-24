<template>
  <LingoLayout>
    <div class="flex w-full flex-col items-center pb-10">
      <div class="flex h-20 w-20 items-center justify-center rounded-2xl bg-green-100">
        <ShoppingBag class="h-10 w-10 text-green-600" />
      </div>

      <h1 class="my-6 text-center text-2xl font-bold text-slate-800">
        {{ t('lingo.shop') }}
      </h1>
      <p class="mb-6 text-center text-slate-600">
        {{ t('lingo.shopDescription') }}
      </p>

      <div class="w-full max-w-md space-y-4 rounded-xl border-2 border-slate-200 bg-white p-4">
        <div
          class="flex items-center gap-4 border-t-2 border-slate-100 pt-4 first:border-t-0 first:pt-0"
        >
          <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-lg bg-rose-100">
            <Heart class="h-7 w-7 text-rose-500" />
          </div>
          <div class="min-w-0 flex-1">
            <p class="font-bold text-slate-700">{{ t('lingo.refillHearts') }}</p>
          </div>
          <span class="text-sm text-slate-500">{{ t('lingo.comingSoon') }}</span>
        </div>
        <div class="flex items-center gap-4 border-t-2 border-slate-100 pt-4">
          <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-lg bg-green-100">
            <Sparkles class="h-7 w-7 text-green-600" />
          </div>
          <div class="min-w-0 flex-1">
            <p class="font-bold text-slate-700">{{ t('lingo.unlimitedHearts') }}</p>
          </div>
          <span class="text-sm text-slate-500">{{ t('lingo.comingSoon') }}</span>
        </div>
      </div>
    </div>
  </LingoLayout>
</template>

<script setup>
import { ShoppingBag, Heart, Sparkles } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import { useSeoHead } from '@/composables/useSeoHead'

const { t, locale } = useI18n()
useSeoHead({ title: t('lingo.shop') }, locale.value)
</script>
