<template>

  <div :class="wrapperClass">
     <div class="flex flex-col items-center justify-center">

      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />

      <p class="text-gray-600 mt-2">
         <slot>{{ t('components.loadingSpinner.loadingDefault') }}</slot
        >
      </p>

    </div>

  </div>

</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(
  defineProps<{
    /** `page`: fixed full-viewport overlay with backdrop blur. `inline`: centered block for modals and small regions. */
    variant?: 'page' | 'inline'
  }>(),
  {
    variant: 'page',
  }
)

const { t } = useI18n()

const wrapperClass = computed(() =>
  props.variant === 'page' ? 'page-loading-overlay' : 'flex flex-col items-center justify-center'
)
</script>
