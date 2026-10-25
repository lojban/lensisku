<template>
  <span v-if="levels.length" class="item-level-badges">
    <RouterLink
      v-for="lv in levels"
      :key="lv.level_id"
      :to="{ path: `/collections/${collectionId}/levels`, query: { level: String(lv.level_id) } }"
      class="item-level-badge"
      :title="t('flashcardLevels.levelChipTitle', { name: lv.level_name })"
      @click.stop
    >
      <Layers class="h-3 w-3 shrink-0" aria-hidden="true" />
      <span class="truncate">{{
        t('flashcardLevels.levelChip', { n: lv.level_position + 1, name: lv.level_name })
      }}</span>
    </RouterLink>
  </span>
</template>

<script setup lang="ts">
import { Layers } from '@lucide/vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'

import type { ItemLevelEntry } from '@/api'

defineProps<{
  levels: ItemLevelEntry[]
  collectionId: number | string
}>()

const { t } = useI18n()
</script>

<style scoped>
.item-level-badges {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.25rem;
  min-width: 0;
  max-width: 100%;
}

.item-level-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  max-width: 14rem;
  height: 1.375rem;
  padding: 0 0.5rem;
  border-radius: 9999px;
  background: #ede9fe;
  color: #5b21b6;
  font-size: 0.75rem;
  font-weight: 600;
  line-height: 1;
  text-decoration: none;
  white-space: nowrap;
}

.item-level-badge:hover {
  background: #ddd6fe;
}
</style>
