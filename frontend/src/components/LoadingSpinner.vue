<template>

  <div class="fixed inset-0 flex items-center justify-center z-50">

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
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

