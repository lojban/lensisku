<script setup>
import { useI18n } from 'vue-i18n'
import { useSeoHead } from '@/composables/useSeoHead'

const { t } = useI18n()
useSeoHead({ title: t('paypalReturn.title') })
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-8">
    <div class="bg-white p-6 rounded-lg shadow-md">
      <h1 class="text-2xl font-bold mb-4">
        {{ t('paypalReturn.title') }}
      </h1>

      <div class="space-y-4 text-gray-700">
        <p>
          {{ t('paypalReturn.thankYou') }}
        </p>

        <p>{{ t('paypalReturn.viewDetails') }}</p>

        <div class="mt-6">
          <RouterLink
            to="/profile"
            class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5 mr-2"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fill-rule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-11a1 1 0 10-2 0v3.586L7.707 9.293a1 1 0 00-1.414 1.414l3 3a1 1 0 001.414 0l3-3a1 1 0 00-1.414-1.414L11 10.586V7z"
                clip-rule="evenodd"
              />
            </svg>
            {{ t('paypalReturn.goToProfile') }}
          </RouterLink>
        </div>
      </div>
    </div>
  </div>
</template>
