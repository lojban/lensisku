<template>
  <div
    class="nav-select"
    :class="inline ? 'nav-select-inline w-max max-w-full shrink-0' : undefined"
  >
    <Select
      searchable
      :class="inline ? 'w-max max-w-full' : 'w-full min-w-0'"
      :search-placeholder="t('filters.searchLanguages')"
      :empty-filter-label="t('filters.noMatches')"
      :model-value="locale"
      :label="t('nav.languages')"
      :select-class="
        inline ? 'input-field nav-select-field !w-max max-w-full' : 'input-field nav-select-field'
      "
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

defineProps({
  /** Hug the label. The default fills a nav cell. */
  inline: { type: Boolean, default: false },
})

const { t } = useI18n()
const { locale, availableLocales, onSelectChange } = useSwitchLocale()
</script>
