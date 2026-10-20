<template>
  <div
    role="status"
    aria-live="polite"
    aria-busy="true"
    :data-variant="variant"
    class="loading-spinner"
  >
    <div class="flex flex-col items-center justify-center">
      <div class="h-8 w-8 shrink-0 animate-spin rounded-full border-b-2 border-blue-600" />
      <p class="mt-2 text-gray-600">
        <slot>{{ t('components.loadingSpinner.loadingDefault') }}</slot>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

withDefaults(
  defineProps<{
    /**
     * `page`: fixed full-viewport veil (auth bootstrap, etc.).
     * `content`: absolute veil over a `relative` parent (in-page filter/search refresh).
     * Path changes use App’s freeze + `route-freeze-spinner` instead.
     * `inline`: centered block for modals and small regions (no veil).
     */
    variant?: 'page' | 'content' | 'inline'
  }>(),
  {
    variant: 'page',
  }
)

const { t } = useI18n()
</script>
