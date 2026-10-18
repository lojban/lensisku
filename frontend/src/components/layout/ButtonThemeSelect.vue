<template>
  <div class="relative min-w-0" :class="block ? 'w-full' : ''">
    <Select
      :model-value="buttonTheme"
      :options="[
        { value: 'aqua', label: t('buttonTheme.aqua') },
        { value: 'flat', label: t('buttonTheme.flat') },
      ]"
      :select-class="
        block
          ? 'input-field appearance-none !h-6 !w-full !min-w-0 !py-0 !pr-8 !text-xs'
          : 'input-field appearance-none !h-6 !py-0 !pr-8 !text-xs min-w-[8rem]'
      "
      :label="t('buttonTheme.label')"
      @update:model-value="onTheme"
    />
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

defineProps({
  block: { type: Boolean, default: false },
})

const { t } = useI18n()
const { buttonTheme, setButtonTheme } = useButtonTheme()

function onTheme(value: unknown) {
  if (value === 'aqua' || value === 'flat') setButtonTheme(value as ButtonThemeId)
}
</script>
