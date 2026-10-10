<template>
  <div class="feed-page">
    <div class="feed-page__body">
      <div class="space-y-3 py-4">
        <SearchForm initial-mode="comments" @search="search" />
        <div
          role="group"
          :aria-label="t('home.searchResultsTitle.comments')"
          class="flex flex-wrap items-center gap-2"
        >
          <ToolbarSelectDropdown
            trigger-class="!w-full max-w-[min(100vw-4rem,18rem)]"
            truncate-label
          >
            <template #label>{{ waveSourceLabel }}</template>
            <ToolbarSelectDropdownItem
              v-for="source in waveSources"
              :key="source"
              :class="{ 'bg-gray-100': waveSource === source }"
              @click="setWaveSource(source)"
            >
              {{ t(`home.waveSource${sourceLabels[source]}`) }}
            </ToolbarSelectDropdownItem>
          </ToolbarSelectDropdown>
          <ToolbarSelectDropdown
            trigger-class="!w-full max-w-[min(100vw-4rem,18rem)]"
            truncate-label
          >
            <template #label>{{ sortByLabel }}</template>
            <ToolbarSelectDropdownItem
              v-for="sort in availableSorts"
              :key="sort"
              :class="{ 'bg-gray-100': sortBy === sort }"
              @click="setSortBy(sort)"
            >
              {{ t(`sort.${sort}`) }}
            </ToolbarSelectDropdownItem>
          </ToolbarSelectDropdown>
          <Button
            variant="empty"
            type="button"
            class="inline-flex h-8 min-w-0 w-auto items-center gap-1.5 whitespace-nowrap px-3 text-sm"
            :title="sortOrder === 'asc' ? t('sort.ascending') : t('sort.descending')"
            @click="toggleSortOrder"
          >
            <ChevronUp v-if="sortOrder === 'asc'" class="h-4 w-4 shrink-0 opacity-60" />
            <ChevronDown v-else class="h-4 w-4 shrink-0 opacity-60" />
            <span>{{ t(sortOrder === 'asc' ? 'sort.asc' : 'sort.desc') }}</span>
          </Button>
        </div>
      </div>
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
import { computed, ref, watch } from 'vue'
import { ChevronDown, ChevronUp } from '@lucide/vue'
import { Button, ToolbarSelectDropdown, ToolbarSelectDropdownItem } from '@packages/ui'
import { useRoute } from 'vue-router'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import SearchForm from '@/components/SearchForm.vue'
import ActivityFeed from '@/components/activity/ActivityFeed.vue'
import LiveChatDock from '@/components/LiveChatDock.vue'
import { useLocalePath } from '@/composables/useLocalePath'
import { useSeoHead } from '@/composables/useSeoHead'
import { queryStr } from '@/utils/routeQuery'

const waveSources = ['all', 'jbotcan', 'freeforums', 'comments', 'mail', 'wiki'] as const
type WaveSource = (typeof waveSources)[number]
const sourceLabels: Record<WaveSource, string> = {
  all: 'All',
  jbotcan: 'Jbotcan',
  freeforums: 'Freeforums',
  comments: 'Comments',
  mail: 'Mail',
  wiki: 'Wiki',
}
const sortOptions = ['relevance', 'time', 'reactions', 'replies'] as const
type SortBy = (typeof sortOptions)[number]
const router = useRouter()
const route = useRoute()
const localePath = useLocalePath()
const { t } = useI18n()
const waveSource = ref<WaveSource>(validWaveSource(queryStr(route.query.wave_source)))
const sortBy = ref<SortBy>(validSortBy(queryStr(route.query.sort_by)))
const sortOrder = ref(queryStr(route.query.sort_order) === 'asc' ? 'asc' : 'desc')
const availableSorts = computed(() =>
  sortOptions.filter((sort) => sort !== 'relevance' || queryStr(route.query.q).trim())
)
const waveSourceLabel = computed(() => t(`home.waveSource${sourceLabels[waveSource.value]}`))
const sortByLabel = computed(() => t(`sort.${sortBy.value}`))
useSeoHead({ title: () => t('activityFeed.title') })

function validWaveSource(value: string): WaveSource {
  return waveSources.includes(value as WaveSource) ? (value as WaveSource) : 'all'
}

function validSortBy(value: string): SortBy {
  return sortOptions.includes(value as SortBy) ? (value as SortBy) : 'relevance'
}

watch(
  () => [route.query.wave_source, route.query.sort_by, route.query.sort_order],
  () => {
    waveSource.value = validWaveSource(queryStr(route.query.wave_source))
    sortBy.value = validSortBy(queryStr(route.query.sort_by))
    sortOrder.value = queryStr(route.query.sort_order) === 'asc' ? 'asc' : 'desc'
  }
)

function updateSearchOptions() {
  void router.replace({
    query: {
      ...route.query,
      wave_source: waveSource.value === 'all' ? undefined : waveSource.value,
      sort_by: sortBy.value === 'relevance' ? undefined : sortBy.value,
      sort_order: sortOrder.value === 'desc' ? undefined : sortOrder.value,
    },
  })
}

function setWaveSource(source: WaveSource) {
  waveSource.value = source
  updateSearchOptions()
}

function setSortBy(sort: SortBy) {
  sortBy.value = sort
  updateSearchOptions()
}

function toggleSortOrder() {
  sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  updateSearchOptions()
}

function search({ query, mode }: { query: string; mode: string }) {
  void router.push({
    path: localePath(),
    query: {
      q: query || undefined,
      mode,
      wave_source: waveSource.value === 'all' ? undefined : waveSource.value,
      sort_by: sortBy.value === 'relevance' ? undefined : sortBy.value,
      sort_order: sortOrder.value === 'desc' ? undefined : sortOrder.value,
    },
  })
}
</script>
