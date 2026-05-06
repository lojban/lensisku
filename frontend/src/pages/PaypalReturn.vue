<script setup lang="ts">
import { User } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

import { Button } from '@packages/ui'
import { useSeoHead } from '@/composables/useSeoHead'

const { t } = useI18n()
useSeoHead({ title: t('paypalReturn.title'), robots: 'noindex, nofollow' })
</script>

<template>
  <div class="rounded-lg bg-white p-6 shadow-md">
    <h1 class="text-2xl font-bold">{{ t('paypalReturn.title') }}</h1>

    <div class="mt-4 space-y-4 text-gray-700">
      <p>{{ t('paypalReturn.thankYou') }}</p>

      <p>{{ t('paypalReturn.viewDetails') }}</p>

      <div>
        <Button
          tag="router-link"
          to="/profile"
          variant="insert"
          size="lg"
          class="inline-flex items-center gap-2 rounded-md px-5 py-2.5"
        >
          <template #icon> <User class="h-5 w-5 shrink-0" /> </template>
          {{ t('paypalReturn.goToProfile') }}
        </Button>
      </div>
    </div>
  </div>
</template>
