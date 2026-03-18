<template>
  <LingoLayout>
    <!-- Match archive: full-height container, 12px horizontal on mobile (-mx-4 cancels layout px-4, then px-3) -->
    <div class="mx-auto h-full max-w-[912px] -mx-4 px-3 lg:mx-auto lg:px-0">
      <h1 class="text-2xl font-bold text-neutral-700">
        {{ t('lingo.languageCourses') }}
      </h1>

      <div class="mt-4">
        <input
          v-model="searchQuery"
          type="text"
          class="input-field w-full sm:max-w-md"
          :placeholder="t('lingo.searchCourses')"
        >
      </div>

      <div v-if="isLoading" class="flex h-full w-full items-center justify-center py-12">
        <div class="h-10 w-10 animate-spin rounded-full border-2 border-green-500 border-t-transparent" />
      </div>

      <div
        v-else
        class="grid grid-cols-2 gap-4 pt-6 lg:grid-cols-[repeat(auto-fill,minmax(210px,1fr))]"
      >
        <LingoCourseCard
          v-for="collection in collections"
          :key="collection.collection_id"
          :collection="collection"
          :is-active="activeCollectionId === collection.collection_id"
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
    </div>
  </LingoLayout>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import LingoCourseCard from '@/components/LingoCourseCard.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import { getPublicCollections } from '@/api'
import { useSeoHead } from '@/composables/useSeoHead'

const LINGO_ACTIVE_COURSE_KEY = 'lingo_active_collection_id'

const router = useRouter()
const { t, locale } = useI18n()

const collections = ref([])
const isLoading = ref(true)
const isSelecting = ref(false)
const activeCollectionId = ref(null)
const searchQuery = ref('')
const currentPage = ref(1)
const perPage = ref(12)
const totalCollections = ref(0)
const totalPages = computed(() => Math.max(1, Math.ceil(totalCollections.value / perPage.value)))
let searchDebounceTimer = null

function getStoredActiveId() {
  try {
    const id = sessionStorage.getItem(LINGO_ACTIVE_COURSE_KEY)
    return id ? Number(id) : null
  } catch {
    return null
  }
}

function setStoredActiveId(id) {
  try {
    if (id != null) sessionStorage.setItem(LINGO_ACTIVE_COURSE_KEY, String(id))
    else sessionStorage.removeItem(LINGO_ACTIVE_COURSE_KEY)
  } catch {}
}

async function loadCollections() {
  isLoading.value = true
  try {
    const response = await getPublicCollections({
      page: currentPage.value,
      per_page: perPage.value,
      search: searchQuery.value.trim() || undefined,
      has_flashcards_only: true,
      has_levels_only: true,
    })
    collections.value = response.data.collections || []
    totalCollections.value = Number(response.data.total || 0)

    if (collections.value.length === 0 && totalCollections.value > 0 && currentPage.value > 1) {
      currentPage.value = Math.max(1, totalPages.value)
      await loadCollections()
      return
    }
    activeCollectionId.value = getStoredActiveId()
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
  if (id === activeCollectionId.value) {
    router.push(`/collections/${id}/lingo/levels`)
    return
  }
  isSelecting.value = true
  setStoredActiveId(id)
  activeCollectionId.value = id
  router.push(`/collections/${id}/lingo/levels`)
  isSelecting.value = false
}

useSeoHead({ title: t('lingo.courses') }, locale.value)

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
