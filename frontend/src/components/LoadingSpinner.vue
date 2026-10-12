<template>
  <div :class="wrapperClass" role="status" aria-live="polite" aria-busy="true">
    <div class="flex flex-col items-center justify-center">
      <div class="h-8 w-8 shrink-0 animate-spin rounded-full border-b-2 border-blue-600" />
      <p class="mt-2 text-gray-600">
        <slot>{{ t('components.loadingSpinner.loadingDefault') }}</slot>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(
  defineProps<{
    /**
     * `page`: fixed full-viewport veil (auth bootstrap, etc.).
     * Route transitions use App’s freeze + `route-freeze-spinner` instead.
     * `inline`: centered block for modals and small regions (no veil).
     */
    variant?: 'page' | 'inline'
  }>(),
  {
    variant: 'page',
  }
)

const { t } = useI18n()

const wrapperClass = computed(() => {
  if (props.variant === 'page') return 'page-loading-overlay'
  return 'flex flex-col items-center justify-center'
})
</script>
