<template>
  <section ref="feedElement" class="space-y-4 pb-20" :aria-label="t('activityFeed.title')">
    <DiscussionComposer @posted="posted" />
    <h2 class="text-lg font-bold text-gray-800">{{ t('activityFeed.title') }}</h2>
    <TabbedPageHeader
      :tabs="tabs"
      :active-tab="filter"
      :page-title="t('activityFeed.title')"
      :show-title="false"
      @tab-click="selectFilter"
    />
    <template v-if="filter === 'news'">
      <NewsActivityFeed />
    </template>
    <template v-else>
      <button
        v-if="newActivity"
        type="button"
        class="w-full rounded-lg bg-blue-50 text-blue-700 py-3 text-sm"
        @click="refresh"
      >
        {{ t('activityFeed.newActivity') }}
      </button>
      <div
        v-if="loading && !rows.length"
        class="space-y-3"
        role="status"
        :aria-label="t('activityFeed.loading')"
      >
        <div v-for="n in 3" :key="n" class="h-36 rounded-xl bg-gray-100 animate-pulse" />
      </div>
      <div class="space-y-3">
        <ActivityCard v-for="row in rows" :key="row.event_id" :row="row" />
      </div>
      <div
        v-if="error"
        class="rounded-lg border border-red-100 bg-red-50 p-4 text-sm text-red-800"
        role="alert"
      >
        <p>{{ t('activityFeed.error') }}</p>
        <button type="button" class="mt-2 underline" @click="retry()">
          {{ t('activityFeed.retry') }}
        </button>
      </div>
      <p v-else-if="!loading && !rows.length" class="text-center text-gray-500 py-10">
        {{ t('activityFeed.empty') }}
      </p>
      <div v-if="rows.length && !error" class="text-center py-3">
        <Button v-if="cursor" variant="neutral" :disabled="loading" @click="load(true)">{{
          loading ? t('activityFeed.loading') : t('activityFeed.loadMore')
        }}</Button>
        <p v-else class="text-sm text-gray-500">{{ t('activityFeed.end') }}</p>
      </div>
    </template>
  </section>
</template>
<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Button } from '@packages/ui'
import { Bell, BookOpen, FileText, List, MessageSquare } from '@lucide/vue'
import DiscussionComposer from './DiscussionComposer.vue'
import ActivityCard from './ActivityCard.vue'
import NewsActivityFeed from './NewsActivityFeed.vue'
import TabbedPageHeader from '@/components/TabbedPageHeader.vue'
import { useActivityFeed } from '@/composables/useActivityFeed'
import { queryStr } from '@/utils/routeQuery'
const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const filter = computed(() =>
  ['all', 'discussions', 'definitions', 'wiki', 'news'].includes(queryStr(route.query.feed))
    ? queryStr(route.query.feed)
    : 'all'
)
const location = computed(() => route.path)
const { rows, cursor, loading, error, newActivity, restoredScroll, load, retry, save, setVisible } =
  useActivityFeed(filter, location)
const tabs = computed(() => [
  { key: 'all', label: t('activityFeed.all'), icon: List },
  { key: 'discussions', label: t('activityFeed.discussions'), icon: MessageSquare },
  { key: 'definitions', label: t('activityFeed.definitions'), icon: BookOpen },
  { key: 'wiki', label: t('activityFeed.wiki'), icon: FileText },
  { key: 'news', label: t('recentChanges.news'), icon: Bell },
])
const feedElement = ref<HTMLElement | null>(null)
let body: HTMLElement | null = null
let observer: IntersectionObserver | undefined
let visible = false
const visibilityChanged = () => setVisible(visible && document.visibilityState === 'visible')
const saveScroll = () => save(body?.scrollTop || 0)
function selectFilter(feed: string) {
  saveScroll()
  void router.push({
    query: { ...route.query, feed: feed === 'all' ? undefined : feed, source: undefined },
  })
}
async function refresh() {
  await load()
  body?.scrollTo({ top: 0, behavior: 'instant' })
}
async function posted() {
  await router.replace({ query: { ...route.query, feed: undefined, source: undefined } })
  await refresh()
}
watch(
  restoredScroll,
  async (scroll) => {
    await nextTick()
    if (body) body.scrollTop = scroll
  },
  { flush: 'post' }
)
onMounted(async () => {
  if (route.query.source) {
    await router.replace({ query: { ...route.query, source: undefined } })
  }
  body = feedElement.value?.closest('.feed-page__body') as HTMLElement | null
  await nextTick()
  if (body) body.scrollTop = restoredScroll.value
  body?.addEventListener('scroll', saveScroll, { passive: true })
  document.addEventListener('visibilitychange', visibilityChanged)
  observer = new IntersectionObserver(([entry]) => {
    visible = entry.isIntersecting
    visibilityChanged()
  })
  if (feedElement.value) observer.observe(feedElement.value)
})
onBeforeUnmount(() => {
  saveScroll()
  body?.removeEventListener('scroll', saveScroll)
  document.removeEventListener('visibilitychange', visibilityChanged)
  observer?.disconnect()
})
</script>
