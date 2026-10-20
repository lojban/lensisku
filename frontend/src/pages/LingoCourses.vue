<template>
  <LingoLayout fill>
    <div class="feed-page min-h-0 flex-1">
      <FeedPageBody>
        <h1 class="text-2xl font-bold text-neutral-700">{{ t('lingo.languageCourses') }}</h1>

        <div class="mt-4">
          <Input
            v-model="searchQuery"
            type="text"
            class="input-field w-full sm:max-w-md"
            :placeholder="t('lingo.searchCourses')"
          />
        </div>

        <div v-if="isLoading" class="flex h-full w-full items-center justify-center py-12">
          <div
            class="h-10 w-10 animate-spin rounded-full border-2 border-green-500 border-t-transparent"
          />
        </div>

        <div
          v-else
          class="grid grid-cols-2 gap-4 pt-6 lg:grid-cols-[repeat(auto-fill,minmax(210px,1fr))]"
        >
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
          v-if="!isLoading && totalPages > 1"
          :current-page="currentPage"
          :total-pages="totalPages"
          :total="totalCollections"
          :per-page="perPage"
          class="w-full mt-6"
          @prev="prevPage"
          @next="nextPage"
        />
        <p v-if="!isLoading && collections.length === 0" class="py-8 text-center text-neutral-600">
          {{ t('lingo.noCourses') }}
        </p>
      </FeedPageBody>
      <div ref="paginationHost" class="contents" />
    </div>
  </LingoLayout>
</template>

<script setup lang="ts">
import { Input } from '@packages/ui'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import LingoCourseCard from '@/components/LingoCourseCard.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import FeedPageBody from '@/components/layout/FeedPageBody.vue'
import { useFeedPaginationHost } from '@/composables/useFeedPaginationHost'
import { getPublicCollections } from '@/api'
import { useSeoHead } from '@/composables/useSeoHead'
import { useLingoProgress } from '@/composables/useLingoProgress'

const router = useRouter()
const paginationHost = useFeedPaginationHost()
const { t } = useI18n()
const { activeCourseId, setActiveCourse } = useLingoProgress()

const collections = ref([])
const isLoading = ref(true)
const isSelecting = ref(false)
const searchQuery = ref('')
const currentPage = ref(1)
const perPage = ref(12)
const totalCollections = ref(0)
const totalPages = computed(() => Math.max(1, Math.ceil(totalCollections.value / perPage.value)))
let searchDebounceTimer = null

async function loadCollections() {
  isLoading.value = true
  try {
    // Match main “courses” catalog: any public collection with flashcards.
    // Requiring levels too hid most public decks (many have cards but no ladder yet).
    const response = await getPublicCollections({
      page: currentPage.value,
      per_page: perPage.value,
      search: searchQuery.value.trim() || undefined,
      has_flashcards_only: true,
    })
    collections.value = response.data.collections || []
    totalCollections.value = Number(response.data.total || 0)

    if (collections.value.length === 0 && totalCollections.value > 0 && currentPage.value > 1) {
      currentPage.value = Math.max(1, totalPages.value)
      await loadCollections()
      return
    }
  } catch (e) {
    console.error(e)
    collections.value = []
    totalCollections.value = 0
  } finally {
    isLoading.value = false
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
