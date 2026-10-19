<template>
  <div class="feed-page">
    <FeedPageBody>
      <div class="space-y-3 py-4">
        <SearchForm initial-mode="comments" @search="search" />
        <div
          role="group"
          :aria-label="t('home.searchResultsTitle.comments')"
          class="flex flex-wrap items-center gap-2"
        >
          <ToolbarSelectDropdown trigger-class="!w-full max-w-[min(100vw-4rem,18rem)]">
            <template #icon>
              <WaveSourceIcon :source="waveSource" />
            </template>
            <template #label>{{ waveSourceLabel }}</template>
            <ToolbarSelectDropdownItem
              v-for="source in waveSources"
              :key="source"
              :class="{ 'bg-gray-100': waveSource === source }"
              @click="setWaveSource(source)"
            >
              <WaveSourceIcon :source="source" />
              {{ t(sourceLabelKeys[source]) }}
            </ToolbarSelectDropdownItem>
          </ToolbarSelectDropdown>
          <SortFieldOrderDropdown
            :field="sortBy"
            :order="sortOrder"
            :options="sortFieldOptions"
            :aria-label="t('sort.sortByLabel')"
            :descending-label="t('sort.descending')"
            :ascending-label="t('sort.ascending')"
            @select="setSortFieldOrder"
          />
        </div>
      </div>
      <ActivityFeed />
    </FeedPageBody>
  </div>
</template>
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  SortFieldOrderDropdown,
  ToolbarSelectDropdown,
  ToolbarSelectDropdownItem,
} from '@packages/ui'
import { useRoute } from 'vue-router'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import SearchForm from '@/components/SearchForm.vue'
import ActivityFeed from '@/components/activity/ActivityFeed.vue'
import FeedPageBody from '@/components/layout/FeedPageBody.vue'
import WaveSourceIcon from '@/components/icons/WaveSourceIcon.vue'
import { useLocalePath } from '@/composables/useLocalePath'
import { useSeoHead } from '@/composables/useSeoHead'
import { queryStr } from '@/utils/routeQuery'

const waveSources = ['all', 'jbotcan', 'freeforums', 'comments', 'mail', 'wiki'] as const
type WaveSource = (typeof waveSources)[number]
const sourceLabelKeys: Record<WaveSource, string> = {
  all: 'home.waveSourceAll',
  jbotcan: 'home.waveSourceJbotcan',
  freeforums: 'home.waveSourceFreeforums',
  comments: 'home.discussionWaves',
  mail: 'home.waveSourceMail',
  wiki: 'home.waveSourceWiki',
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
const waveSourceLabel = computed(() => t(sourceLabelKeys[waveSource.value]))
const sortFieldOptions = computed(() =>
  sortOptions.map((value) => ({ value, label: t(`sort.${value}`) }))
)
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

function setSortFieldOrder({ field, order }: { field: string; order: 'asc' | 'desc' }) {
  sortBy.value = validSortBy(field)
  sortOrder.value = order
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
