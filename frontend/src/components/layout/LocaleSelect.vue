<template>
  <div class="flex min-w-0 items-center" :class="block ? 'w-full' : ''">
    <Select
      searchable
      :class="block ? 'w-full min-w-0' : 'w-auto'"
      :search-placeholder="t('filters.searchLanguages')"
      :empty-filter-label="t('filters.noMatches')"
      :model-value="locale"
      :label="t('nav.languages')"
      :select-class="
        block
          ? 'input-field appearance-none !h-6 !w-full !min-w-0 !py-0 !pr-3 !text-xs'
          : 'input-field appearance-none !h-6 !py-0 !pr-3 min-w-[8rem] !text-xs'
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
  block: { type: Boolean, default: false },
})

const { t } = useI18n()
const { locale, availableLocales, onSelectChange } = useSwitchLocale()
</script>
