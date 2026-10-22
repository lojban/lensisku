<template>
  <div
    role="status"
    aria-live="polite"
    aria-busy="true"
    :data-variant="variant"
    class="loading-spinner"
  >
    <div class="loading-spinner__mark" aria-hidden="true" />
    <p v-if="slots.default" class="loading-spinner__label">
      <slot />
    </p>
    <span v-else class="sr-only">{{ t('components.loadingSpinner.loadingDefault') }}</span>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useSlots } from 'vue'

withDefaults(
  defineProps<{
    /**
     * `page`: fixed full-viewport wash (auth bootstrap, etc.).
     * `content`: absolute wash over a `relative` parent (in-page filter/search refresh).
     * Path changes use App’s freeze + `.route-progress` bar instead.
     * `inline`: centered ring for modals and small regions (no veil).
     */
    variant?: 'page' | 'content' | 'inline'
  }>(),
  {
    variant: 'page',
  }
)

const { t } = useI18n()
const slots = useSlots()
</script>
