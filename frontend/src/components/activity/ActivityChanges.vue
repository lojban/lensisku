<template>
  <div>
    <div v-if="groupedChanges.length">
      <div
        v-for="(group, index) in groupedChanges"
        :key="index"
        class="mb-8"
      >
        <h3 class="text-base font-semibold text-gray-700 mb-4 pt-4 border-t">
          {{ formatDate(group.date) }}
        </h3>
        <div class="space-y-3">
          <RecentChangeItem
            v-for="change in group.changes"
            :key="change.time"
            :change="change"
          />
        </div>
      </div>
    </div>
    <div
      v-else
      class="text-center py-8 bg-gray-50 rounded-lg border border-gray-200"
    >
      <p class="text-sm text-gray-600">
        {{ t('recentChanges.noChangesFound') }}
      </p>
    </div>
  </div>
</template>

<script setup>
import { useI18n } from 'vue-i18n'

import RecentChangeItem from '@/components/RecentChangeItem.vue'

const { t } = useI18n()

defineProps({
  groupedChanges: {
    type: Array,
    required: true
  },
  formatDate: {
    type: Function,
    required: true
  }
})
</script>
