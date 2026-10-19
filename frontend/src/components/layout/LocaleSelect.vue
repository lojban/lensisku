<template>
  <div class="nav-select">
    <Select
      searchable
      class="w-full min-w-0"
      :search-placeholder="t('filters.searchLanguages')"
      :empty-filter-label="t('filters.noMatches')"
      :model-value="locale"
      :label="t('nav.languages')"
      select-class="input-field nav-select-field"
      :options="availableLocales.map((loc) => ({ value: loc, label: localeNativeName(loc) }))"
      @change="onSelectChange"
    />
  </div>
</template>

<script setup lang="ts">
import { Select } from '@packages/ui'
import { useI18n } from 'vue-i18n'

import { localeNativeName } from '@/config/locales'
import { useSwitchLocale } from '@/composables/useSwitchLocale'

const { t } = useI18n()
const { locale, availableLocales, onSelectChange } = useSwitchLocale()
</script>
