<template>
  <div class="nav-select">
    <Select
      searchable
      class="w-full min-w-0"
      :search-placeholder="t('filters.searchOptions')"
      :empty-filter-label="t('filters.noMatches')"
      :model-value="buttonTheme"
      :options="[
        { value: 'aqua', label: t('buttonTheme.aqua') },
        { value: 'flat', label: t('buttonTheme.flat') },
      ]"
      select-class="input-field nav-select-field"
      :label="t('buttonTheme.label')"
      @update:model-value="onTheme"
    />
  </div>
</template>

<script setup lang="ts">
import { Select } from '@packages/ui'
import { useI18n } from 'vue-i18n'

import { useButtonTheme, type ButtonThemeId } from '@/composables/useButtonTheme'

const { t } = useI18n()
const { buttonTheme, setButtonTheme } = useButtonTheme()

function onTheme(value: unknown) {
  if (value === 'aqua' || value === 'flat') setButtonTheme(value as ButtonThemeId)
}
</script>
