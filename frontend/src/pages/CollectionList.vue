<template>
  <div class="flex flex-col gap-4">
    <StudyStreak />

    <PageHeader title-as="h2" title-tone="secondary" margin="none" class="mt-0">
      <template #title>
        <span class="select-none">{{ t('collectionList.catalogCourses') }}</span>
      </template>
      <template #description>
        <div class="flex max-w-2xl flex-col gap-2 text-sm text-slate-600">
          <p class="m-0">{{ t('collectionList.learnDescription') }}</p>
          <p class="m-0">{{ t('collectionList.collectionDescriptionCourse') }}</p>
        </div>
      </template>
      <template #trailing>
        <RouterLink
          v-if="auth.state.isLoggedIn"
          to="/library"
          class="ui-btn--neutral inline-flex items-center gap-2"
        >
          <Star class="h-4 w-4 shrink-0" />
          <span>{{ t('nav.library') }}</span>
        </RouterLink>
        <RouterLink v-else to="/login" class="font-medium text-blue-600 underline hover:text-blue-800">
          {{ t('collectionList.loginTo') }}
        </RouterLink>
      </template>
    </PageHeader>

    <div class="card-base card-compact p-4 sm:p-5 flex flex-col gap-4 overflow-visible">
      <Input
        v-model="searchQuery"
        type="text"
        class="input-field w-full min-w-0 sm:max-w-md sm:mx-auto"
        :placeholder="
          segment === 'courses'
            ? t('collectionList.searchPlaceholder')
            : t('collectionList.searchPlaceholderCollections')
        "
      />

      <div class="flex w-full justify-center">
        <div class="btn-group-forced flex flex-nowrap" role="tablist">
          <Button
            v-for="seg in segments"
            :key="seg.value"
            variant="empty"
            type="button"
            role="tab"
            class="ui-btn--group-item"
            :class="segment === seg.value ? 'ui-btn--sort-sky' : 'ui-btn--empty'"
            :aria-selected="segment === seg.value"
            @click="setSegment(seg.value)"
          >
            {{ seg.label }}
          </Button>
        </div>
      </div>

      <div class="flex flex-row items-center gap-2 sm:block">
        <span id="collection-list-sort-legend" class="sr-only">{{ t('sort.sortByLabel') }}</span>
        <div
          class="btn-group-forced flex flex-nowrap justify-center min-w-0 overflow-visible py-2"
          role="group"
          aria-labelledby="collection-list-sort-legend"
          aria-describedby="collection-list-sort-current"
        >
          <Button
            v-for="opt in sortOptions"
            :key="opt.value"
            variant="empty"
            type="button"
            class="ui-btn--group-item relative flex h-6 shrink-0 items-center justify-center gap-1.5 px-2 sm:px-4 !cursor-pointer"
            :class="[sortBy === opt.value ? opt.aquaClass : 'ui-btn--empty']"
            :title="opt.label"
            :aria-label="opt.label"
            :aria-pressed="sortBy === opt.value"
            @click="sortBy = opt.value"
          >
            <component
              :is="opt.icon"
              class="h-4 w-4 shrink-0 transition-[opacity,filter] duration-200"
              :class="
                sortBy === opt.value
                  ? 'opacity-100 drop-shadow-[0_0_1px_rgba(30,64,175,0.9)]'
                  : 'opacity-55'
              "
              aria-hidden="true"
            /><span class="hidden sm:inline">{{ opt.label }}</span>
          </Button>
        </div>
        <span
          id="collection-list-sort-current"
          class="min-w-0 shrink-0 text-sm text-gray-700 sm:hidden"
          aria-live="polite"
          >{{ selectedSortLabel }}</span
        >
      </div>
    </div>
    <LoadingSpinner v-if="isLoading" />
    <div v-else class="collections-section">
      <div class="collections-grid">
        <CollectionCard
          v-for="collection in collections"
          :key="collection.collection_id"
          :collection="collection"
          :cover-image-url="
            collection.has_cover_image
              ? getCollectionImage(collection.collection_id, { cached: true })
              : null
          "
          :study-loading="studyLoadingId === collection.collection_id"
          :show-study="segment === 'courses'"
          :format-date="formatDate"
          :study-button-label="t('collectionList.studyButton')"
          :collection-button-label="t('collectionList.collectionButton')"
          :flashcards-button-label="t('collectionList.flashcardsButton')"
          :created-by-label="t('collectionList.createdBy')"
          :updated-label="t('collectionList.updatedAt')"
          :public-label="t('collectionList.publicStatus')"
          :private-label="t('collectionList.privateStatus')"
          :items-count-label="t('collectionList.itemsCount', { count: collection.item_count })"
          @study="startStudy(collection)"
        />
      </div>
      <PaginationComponent
        v-if="totalPages > 1"
        :current-page="currentPage"
        :total-pages="totalPages"
        :total="totalCollections"
        :per-page="perPage"
        class="w-full"
        @prev="prevPage"
        @next="nextPage"
      />
    </div>
    <EmptyStatePanel v-if="!isLoading && collections.length === 0" />
  </div>
</template>

<script setup lang="ts">
import { Star, CalendarDays, Calendar, Trophy, ArrowDown } from '@lucide/vue'
import { ref, computed, onBeforeUnmount, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { getPublicCollections, getLevels, getCollectionImage } from '@/api'
import { Button, CollectionCard, EmptyStatePanel, Input } from '@packages/ui'
import PageHeader from '@/components/layout/PageHeader.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import StudyStreak from '@/components/StudyStreak.vue'
import { useAuth } from '@/composables/useAuth'
import { useSeoHead } from '@/composables/useSeoHead'

const SEGMENT_STORAGE_KEY = 'collections-catalog-segment'

const auth = useAuth()
const router = useRouter()
const { t, locale } = useI18n()

const initialSegment = () => {
  const stored =
    typeof localStorage !== 'undefined' ? localStorage.getItem(SEGMENT_STORAGE_KEY) : null
  return stored === 'collections' ? 'collections' : 'courses'
}

const segment = ref(initialSegment())
const collections = ref([])
const isLoading = ref(true)
const hasLoadedOnce = ref(false)
let loadRequestId = 0
const sortBy = ref('active_week')
const searchQuery = ref('')
const currentPage = ref(1)
const perPage = ref(12)
const totalCollections = ref(0)
const studyLoadingId = ref(null)
let searchDebounceTimer = null

const totalPages = computed(() => Math.max(1, Math.ceil(totalCollections.value / perPage.value)))

const segments = computed(() => [
  { value: 'courses', label: t('collectionList.catalogCourses') },
  { value: 'collections', label: t('collectionList.catalogCollections') },
])

const sortOptions = computed(() => [
  {
    value: 'active_week',
    label: t('collectionList.sortActiveWeek'),
    icon: CalendarDays,
    aquaClass: 'ui-btn--sort-sky',
  },
  {
    value: 'active_month',
    label: t('collectionList.sortActiveMonth'),
    icon: Calendar,
    aquaClass: 'ui-btn--sort-blue',
  },
  {
    value: 'active_all',
    label: t('collectionList.sortActiveAll'),
    icon: Trophy,
    aquaClass: 'ui-btn--sort-amber',
  },
  {
    value: 'newest',
    label: t('collectionList.sortNewest'),
    icon: ArrowDown,
    aquaClass: 'ui-btn--sort-emerald',
  },
])

const selectedSortLabel = computed(
  () => sortOptions.value.find((o) => o.value === sortBy.value)?.label ?? ''
)

const pageTitle = computed(() => t('collectionList.catalogCourses'))
useSeoHead({ title: pageTitle, pathWithoutLocale: '/collections' })

function setSegment(next) {
  segment.value = next === 'collections' ? 'collections' : 'courses'
}

const fetchCollections = async () => {
  const requestId = ++loadRequestId
  if (!hasLoadedOnce.value) isLoading.value = true

  try {
    const response = await getPublicCollections({
      sort: sortBy.value,
      page: currentPage.value,
      per_page: perPage.value,
      has_flashcards_only: segment.value === 'courses' ? true : undefined,
      search: searchQuery.value.trim() || undefined,
    })

    if (requestId !== loadRequestId) return

    collections.value = response.data.collections || []
    totalCollections.value = Number(response.data.total || 0)

    if (collections.value.length === 0 && totalCollections.value > 0 && currentPage.value > 1) {
      currentPage.value = Math.max(1, totalPages.value)
      await fetchCollections()
    }
  } catch (error) {
    if (requestId !== loadRequestId) return
    console.error('Error fetching collections:', error)
    totalCollections.value = 0
  } finally {
    if (requestId === loadRequestId) {
      isLoading.value = false
      hasLoadedOnce.value = true
    }
  }
}

const prevPage = () => {
  if (currentPage.value <= 1) return
  currentPage.value -= 1
  fetchCollections()
}

const nextPage = () => {
  if (currentPage.value >= totalPages.value) return
  currentPage.value += 1
  fetchCollections()
}

const formatDate = (date) => {
  return new Date(date).toLocaleDateString(locale.value, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

const startStudy = async (collection) => {
  if (studyLoadingId.value) return
  studyLoadingId.value = collection.collection_id
  try {
    const response = await getLevels(collection.collection_id)
    const hasLevels = response.data?.levels?.length > 0
    if (hasLevels) {
      router.push(`/collections/${collection.collection_id}/levels`)
    } else {
      router.push(`/collections/${collection.collection_id}/flashcards/study`)
    }
  } catch (err) {
    console.error('Error loading levels:', err)
    router.push(`/collections/${collection.collection_id}/flashcards/study`)
  } finally {
    studyLoadingId.value = null
  }
}

watch(
  segment,
  (val) => {
    if (typeof localStorage !== 'undefined') localStorage.setItem(SEGMENT_STORAGE_KEY, val)
  },
  { immediate: true }
)

watch([segment, sortBy], () => {
  currentPage.value = 1
  fetchCollections()
})

watch(searchQuery, () => {
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
  searchDebounceTimer = setTimeout(() => {
    currentPage.value = 1
    fetchCollections()
  }, 300)
})

onMounted(() => {
  fetchCollections()
})

onBeforeUnmount(() => {
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
})
</script>

<style scoped>
.collections-section {
  @apply w-full;
}

.collections-grid {
  @apply grid gap-4 sm:gap-5 items-stretch grid-cols-1 sm:grid-cols-2 lg:grid-cols-3;
}
</style>
