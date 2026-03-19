<template>
  <div class="text-center py-20">
    <h1 class="text-4xl font-bold mb-4">{{ $t('notFoundPage.pageTitle') }}</h1>
    <p class="text-gray-600 mb-8">{{ $t('notFoundPage.description') }}</p>
    <RouterLink
      to="/"
      class="btn-aqua-blue px-6 py-2 text-lg"
    >
      {{ $t('notFoundPage.returnHome') }}
    </RouterLink>
  </div>
</template>

<script setup>
import { RouterLink } from 'vue-router'
import { useSeoHead } from '@/composables/useSeoHead'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
useSeoHead({ title: t('notFoundPage.pageTitle') })
</script>
