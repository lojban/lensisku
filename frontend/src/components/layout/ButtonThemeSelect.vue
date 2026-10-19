<template>
  <div class="nav-select relative">
    <div class="w-full min-w-0">
      <Select
        class="w-full min-w-0"
        :model-value="buttonTheme"
        :options="[
          { value: 'aqua', label: t('buttonTheme.aqua') },
          { value: 'flat', label: t('buttonTheme.flat') },
        ]"
        select-class="input-field nav-select-field nav-select-field--chevron"
        :label="t('buttonTheme.label')"
        @update:model-value="onTheme"
      />
    </div>
    <ChevronDown
      class="pointer-events-none absolute right-2 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400"
      aria-hidden="true"
    />
  </div>
</template>

<script setup lang="ts">
import { Select } from '@packages/ui'
import { ChevronDown } from '@lucide/vue'
import { useI18n } from 'vue-i18n'

import { useButtonTheme, type ButtonThemeId } from '@/composables/useButtonTheme'

const { t } = useI18n()
const { buttonTheme, setButtonTheme } = useButtonTheme()

function onTheme(value: unknown) {
  if (value === 'aqua' || value === 'flat') setButtonTheme(value as ButtonThemeId)
}
</script>
