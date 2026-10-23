<template>
  <LingoLayout fill>
    <!-- Wait for the first catalog payload before painting the feed (avoids header/grid CLS). -->
    <div
      v-if="isLoading && !hasLoadedOnce"
      class="flex min-h-0 flex-1 items-center justify-center py-12"
    >
      <div
        class="h-10 w-10 animate-spin rounded-full border-2 border-green-500 border-t-transparent"
        role="status"
        :aria-label="t('components.loadingSpinner.loadingDefault')"
      />
    </div>

    <LingoFeedLayout
      v-else
      class="min-h-0 flex-1"
      :show-aside="displayedSegment === 'courses'"
      :show-header-aside="false"
      :aria-busy="isLoading || undefined"
    >
      <template #header>
        <div class="flex h-10 w-full min-w-0 items-center justify-between gap-2">
          <div
            class="lingo-catalog-tabs w-fit"
            role="tablist"
            :aria-label="t('collectionList.learnDescription')"
          >
            <button
              v-for="seg in segments"
              :key="seg.value"
              type="button"
              role="tab"
              class="lingo-catalog-tabs__tab"
              :class="{ 'lingo-catalog-tabs__tab--active': segment === seg.value }"
              :aria-selected="segment === seg.value"
              @click="setSegment(seg.value)"
            >
              {{ seg.label }}
            </button>
          </div>

          <!-- Fixed 40×40 slot: unmounting the sort control on Collections caused header CLS. -->
          <div class="flex h-10 w-10 shrink-0 items-center justify-center">
            <ToolbarSelectDropdown
              v-if="segment === 'courses'"
              :aria-label="selectedSortLabel"
              trigger-icon="ellipsis"
            >
              <template #trigger="{ open }">
                <button
                  type="button"
                  class="lingo-icon-btn lingo-icon-btn--muted"
                  :aria-label="selectedSortLabel"
                  aria-haspopup="menu"
                  :aria-expanded="open"
                >
                  <ArrowUpDown class="h-5 w-5" aria-hidden="true" />
                </button>
              </template>
              <ToolbarSelectDropdownItem
                v-for="opt in sortOptions"
                :key="opt.value"
                :class="{ 'bg-green-50 text-green-700': sortBy === opt.value }"
                @click="sortBy = opt.value"
              >
                {{ opt.label }}
              </ToolbarSelectDropdownItem>
            </ToolbarSelectDropdown>
          </div>
        </div>
      </template>

      <template #toolbar>
        <Input
          v-model="searchQuery"
          type="text"
          class="input-field w-full sm:max-w-md"
          :placeholder="
            segment === 'courses'
              ? t('lingo.searchCourses')
              : t('collectionList.searchPlaceholderCollections')
          "
        />
      </template>

      <!-- Keep the previous grid until the next payload lands (brandbook §8.2). -->
      <div class="grid grid-cols-2 gap-4 lg:grid-cols-[repeat(auto-fill,minmax(210px,1fr))]">
        <LingoCourseCard
          v-for="collection in collections"
          :key="collection.collection_id"
          :collection="collection"
          :is-active="activeCourseId === collection.collection_id"
          :disabled="isSelecting"
          @select="onSelectCourse"
        />
      </div>
      <PaginationComponent
        v-if="totalPages > 1"
        :current-page="currentPage"
        :total-pages="totalPages"
        :total="totalCollections"
        :per-page="perPage"
        class="mt-6 w-full"
        @prev="prevPage"
        @next="nextPage"
      />
      <p v-if="collections.length === 0" class="py-8 text-center text-neutral-600">
        {{ t('lingo.noCourses') }}
      </p>
    </LingoFeedLayout>
  </LingoLayout>
</template>

<script setup lang="ts">
import { Input, ToolbarSelectDropdown, ToolbarSelectDropdownItem } from '@packages/ui'
import { ArrowUpDown } from '@lucide/vue'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import LingoFeedLayout from '@/components/LingoFeedLayout.vue'
import LingoCourseCard from '@/components/LingoCourseCard.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import { getPublicCollections } from '@/api'
import { useSeoHead } from '@/composables/useSeoHead'
import { useLingoProgress } from '@/composables/useLingoProgress'

const router = useRouter()
const { t } = useI18n()
const { activeCourseId, setActiveCourse } = useLingoProgress()

const SEGMENT_STORAGE_KEY = 'lingo-catalog-segment'

const segment = ref(
  typeof localStorage !== 'undefined' && localStorage.getItem(SEGMENT_STORAGE_KEY) === 'collections'
    ? 'collections'
    : 'courses'
)
const collections = ref([])
const isLoading = ref(true)
const hasLoadedOnce = ref(false)
/** Aside/chrome that must swap with the grid, not ahead of it (§8.2). */
const displayedSegment = ref(segment.value)
const isSelecting = ref(false)
const sortBy = ref('recently_studied')
const searchQuery = ref('')
const currentPage = ref(1)
const perPage = ref(12)
const totalCollections = ref(0)
const totalPages = computed(() => Math.max(1, Math.ceil(totalCollections.value / perPage.value)))
const segments = computed(() => [
  { value: 'courses', label: t('collectionList.catalogCourses') },
  { value: 'collections', label: t('collectionList.catalogCollections') },
])
const sortOptions = computed(() => [
  { value: 'recently_studied', label: t('collectionList.sortRecentlyStudied') },
  { value: 'active_week', label: t('collectionList.sortActiveWeek') },
  { value: 'active_month', label: t('collectionList.sortActiveMonth') },
  { value: 'active_all', label: t('collectionList.sortActiveAll') },
  { value: 'newest', label: t('collectionList.sortNewest') },
])
const selectedSortLabel = computed(() =>
  `${t('sort.sortByLabel')} ${
    sortOptions.value.find((o) => o.value === sortBy.value)?.label ?? ''
  }`.trim()
)
let searchDebounceTimer = null
let loadRequestId = 0

function setSegment(next) {
  segment.value = next === 'collections' ? 'collections' : 'courses'
}

async function loadCollections() {
  const requestId = ++loadRequestId
  // Always mark busy for a11y; the grid stays until replace (brandbook §8.2).
  isLoading.value = true
  const nextSegment = segment.value
  try {
    // Courses: public collections with flashcards. Collections: full public catalog.
    // Search always applies to the active tab (name/description ILIKE + flashcards filter).
    const response = await getPublicCollections({
      sort: sortBy.value,
      page: currentPage.value,
      per_page: perPage.value,
      search: searchQuery.value.trim() || undefined,
      has_flashcards_only: nextSegment === 'courses' ? true : undefined,
    })
    if (requestId !== loadRequestId) return
    collections.value = response.data.collections || []
    totalCollections.value = Number(response.data.total || 0)
    displayedSegment.value = nextSegment

    if (collections.value.length === 0 && totalCollections.value > 0 && currentPage.value > 1) {
      currentPage.value = Math.max(1, totalPages.value)
      await loadCollections()
      return
    }
  } catch (e) {
    if (requestId !== loadRequestId) return
    console.error(e)
    collections.value = []
    totalCollections.value = 0
    displayedSegment.value = nextSegment
  } finally {
    if (requestId === loadRequestId) {
      isLoading.value = false
      hasLoadedOnce.value = true
    }
  }
}

function prevPage() {
  if (currentPage.value <= 1) return
  currentPage.value -= 1
  loadCollections()
}

function nextPage() {
  if (currentPage.value >= totalPages.value) return
  currentPage.value += 1
  loadCollections()
}

function onSelectCourse(id) {
  if (isSelecting.value) return
  const collection = collections.value.find((c) => c.collection_id === id)
  isSelecting.value = true
  setActiveCourse(id, collection?.name || '')
  router.push(`/collections/${id}/lingo/levels`)
  isSelecting.value = false
}

useSeoHead({ title: t('lingo.courses') })

watch(segment, (val) => {
  if (typeof localStorage !== 'undefined') localStorage.setItem(SEGMENT_STORAGE_KEY, val)
})

watch([segment, sortBy], () => {
  currentPage.value = 1
  loadCollections()
})

watch(searchQuery, () => {
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
  searchDebounceTimer = setTimeout(() => {
    currentPage.value = 1
    loadCollections()
  }, 300)
})

onMounted(loadCollections)

onBeforeUnmount(() => {
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
})
</script>

<style>
/* Same rules as tailwind.lingo.mjs. The running dev server may not rebuild that file. */
.lingo-catalog-tabs {
  display: inline-flex;
  width: fit-content;
  max-width: 100%;
  height: 2.5rem;
  box-sizing: border-box;
  align-self: center;
  align-items: stretch;
  overflow: hidden;
  border-radius: 0.75rem;
  border: 2px solid rgb(226 232 240);
  border-bottom-width: 4px;
  background: #fff;
}

.lingo-catalog-tabs__tab {
  cursor: pointer;
  border: 0;
  background: transparent;
  padding: 0 1rem;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: rgb(100 116 139);
}

.lingo-catalog-tabs__tab:hover {
  background: rgb(248 250 252);
}

.lingo-catalog-tabs__tab + .lingo-catalog-tabs__tab {
  border-left: 2px solid rgb(226 232 240);
}

.lingo-catalog-tabs__tab--active,
.lingo-catalog-tabs__tab--active:hover {
  background: rgb(34 197 94);
  color: #fff;
}
</style>
