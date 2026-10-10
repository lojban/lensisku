<template>
  <div class="feed-page">
    <div class="feed-page__body">
      <div class="py-4"><SearchForm initial-mode="comments" @search="search" /></div>
      <ActivityFeed />
    </div>
    <div v-if="queryStr(route.query.feed) === 'news'" class="feed-page__footer">
      <div class="feed-page__footer-inner">
        <LiveChatDock />
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import SearchForm from '@/components/SearchForm.vue'
import ActivityFeed from '@/components/activity/ActivityFeed.vue'
import LiveChatDock from '@/components/LiveChatDock.vue'
import { useLocalePath } from '@/composables/useLocalePath'
import { useSeoHead } from '@/composables/useSeoHead'
import { queryStr } from '@/utils/routeQuery'
const router = useRouter()
const route = useRoute()
const localePath = useLocalePath()
const { t } = useI18n()
useSeoHead({ title: () => t('activityFeed.title') })
function search({ query, mode }: { query: string; mode: string }) {
  void router.push({ path: localePath(), query: { q: query || undefined, mode } })
}
</script>
