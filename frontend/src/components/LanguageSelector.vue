<template>
  <ToolbarSelectDropdown :aria-label="currentLanguage.name">
    <template #label>{{ currentLanguage.name }}</template>
    <ToolbarSelectDropdownItem
      v-for="language in sortedLanguages"
      :key="language.id"
      @click="setLanguage(language.id)"
    >
      <img class="h-5 w-5 shrink-0 object-contain" :src="language.flag" alt="" />
      <span>{{ language.name }}</span>
    </ToolbarSelectDropdownItem>
  </ToolbarSelectDropdown>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { ToolbarSelectDropdown, ToolbarSelectDropdownItem } from '@packages/ui'
import { LANGUAGES } from '../composables/tiktoknu/languages'
import { useLocalization } from '../composables/tiktoknu/useLocalization'

const { currentLanguage, setLanguage } = useLocalization()

const sortedLanguages = computed(() => [...LANGUAGES].sort((a, b) => a.name.localeCompare(b.name)))
</script>
