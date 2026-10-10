<template>
  <article class="min-w-0">
    <RecentChangeItem v-if="row.change_type === 'comment'" :change="row" compact />
    <RecentChangeItem v-else :change="row" />
  </article>
</template>
<script setup lang="ts">
import RecentChangeItem from '@/components/RecentChangeItem.vue'
import type { ActivityRow } from '@/composables/useActivityFeed'
defineProps<{ row: ActivityRow }>()
</script>
