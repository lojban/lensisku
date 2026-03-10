<template>
  <LingoLayout>
    <!-- Match archive: full-height container, 12px horizontal on mobile (-mx-4 cancels layout px-4, then px-3) -->
    <div class="mx-auto h-full max-w-[912px] -mx-4 px-3 lg:mx-auto lg:px-0">
      <h1 class="text-2xl font-bold text-neutral-700">
        {{ t('lingo.languageCourses', 'Language Courses') }}
      </h1>

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

      <p v-if="!isLoading && collections.length === 0" class="py-8 text-center text-neutral-600">
        {{ t('lingo.noCourses', 'No courses yet. Create a collection from the main app.') }}
      </p>
    </div>
  </LingoLayout>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import LingoCourseCard from '@/components/LingoCourseCard.vue'
import { getPublicCollections } from '@/api'
import { useSeoHead } from '@/composables/useSeoHead'

const LINGO_ACTIVE_COURSE_KEY = 'lingo_active_collection_id'

const router = useRouter()
const { t, locale } = useI18n()

const collections = ref([])
const isLoading = ref(true)
const isSelecting = ref(false)
const activeCollectionId = ref(null)

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
    const response = await getPublicCollections({})
    const raw = response.data.collections || []
    // Prefer backend-computed has_flashcards, fallback to item_count for compatibility.
    collections.value = raw.filter((c) => c.has_flashcards || (c.item_count ?? 0) > 0)
    activeCollectionId.value = getStoredActiveId()
  } catch (e) {
    console.error(e)
    collections.value = []
  } finally {
    isLoading.value = false
  }
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

useSeoHead({ title: t('lingo.courses', 'Courses') }, locale.value)

onMounted(loadCollections)
</script>
