<template>
  <!-- Streak Stats -->
  <div v-if="auth.state.isLoggedIn" class="mb-4 h-24 bg-white px-3 py-2.5 rounded-lg border">
    <template v-if="!isLoadingStreak && streakData">
      <div class="flex items-center justify-between mb-1.5">
        <h3 class="text-sm font-semibold text-gray-800">
          {{ t('collectionList.studyStreak') }}
        </h3>
        <div class="text-xs text-gray-600">
          {{ t('collectionList.currentStreak') }}: <span class="font-semibold">{{ t('collectionList.days', {
            count:
              streakData.current_streak
          }) }}</span>
        </div>
      </div>
      <div class="grid grid-cols-7 gap-1">
        <div v-for="day in streakData.daily_progress.slice(0, 7).reverse()" :key="day.date"
          class="flex flex-col items-center">
          <div class="text-[10px] text-gray-500 leading-tight">
            {{ new Date(day.date).toLocaleDateString(locale, { weekday: 'short' }) }}
          </div>
          <div class="w-6 h-6 rounded-full flex items-center justify-center text-xs"
            :class="day.reviews_count > 0 ? 'bg-blue-100 text-blue-700' : 'bg-gray-100 text-gray-400'">
            {{ day.reviews_count }}
          </div>
          <div class="text-[10px] text-gray-500 leading-tight">
            {{ t('collectionList.points', { count: day.points }) }}
          </div>
        </div>
      </div>
    </template>
    <div v-else class="animate-pulse h-24">
      <div class="flex items-center justify-between mb-2">
        <div class="h-4 bg-gray-200 rounded w-1/3" />
        <div class="h-3 bg-gray-100 rounded w-1/4" />
      </div>
      <div class="grid grid-cols-7 gap-1">
        <div v-for="i in 7" :key="i" class="flex flex-col items-center">
          <div class="h-3 bg-gray-100 rounded w-full max-w-[32px] mb-0.5" />
          <div class="w-6 h-6 rounded-full bg-gray-100" />
          <div class="h-2.5 bg-gray-100 rounded w-full max-w-[24px] mt-0.5" />
        </div>
      </div>
    </div>
  </div>

  <!-- Header -->
  <div class="flex flex-col sm:flex-row justify-between items-center gap-2 space-x-2 mb-4">
    <h2 class="text-xl sm:text-2xl font-bold text-gray-800">
      {{ viewMode !== 'my' ? t('collectionList.publicCollections') : t('collectionList.myCollections') }}
    </h2>
    <div class="flex flex-col sm:flex-row justify-end flex-grow gap-2 sm:gap-0 mt-2 sm:mt-0">
      <input ref="importFileInput" type="file" accept=".json" class="hidden" @change="handleImportFile">
      <div v-if="auth.state.isLoggedIn" class="flex flex-row gap-2 items-center">
        <!-- When in 'my' mode: show an IconButton to switch back to public view -->
        <IconButton
          v-if="viewMode === 'my'"
          :label="t('collectionList.publicCollectionsLabel')"
          button-classes="btn-aqua-white"
          @click="setViewMode('public')"
        >
          <template #icon>
            <ArrowBigRight class="h-4 w-4" />
          </template>
        </IconButton>
        <Dropdown :trigger-label="t('collectionList.addActions')">
          <button
            type="button"
            class="w-full px-4 py-2 text-left text-sm text-orange-600 hover:bg-orange-50 flex items-center gap-2"
            @click="setViewMode('my')"
          >
            <BookOpen class="h-4 w-4 shrink-0" />
            {{ t('collectionList.myCollectionsLabel') }}
          </button>
          <button
            type="button"
            class="w-full px-4 py-2 text-left text-sm text-cyan-600 hover:bg-cyan-50 flex items-center gap-2"
            :disabled="isImporting"
            @click="triggerImport"
          >
            <Upload class="h-4 w-4 shrink-0" />
            {{ t('collectionList.importCollection') }}
          </button>
          <button
            type="button"
            class="w-full px-4 py-2 text-left text-sm text-emerald-600 hover:bg-emerald-50 flex items-center gap-2"
            @click="showCreateModal = true"
          >
            <CirclePlus class="h-4 w-4 shrink-0" />
            {{ t('collectionList.createCollection') }}
          </button>
        </Dropdown>
      </div>
    </div>
  </div>

  <!-- Sort Controls -->
  <div class="flex flex-wrap items-center justify-center gap-2 mb-4">
    <div class="flex flex-wrap justify-center gap-1" role="group">
      <button
        v-for="opt in sortOptions"
        :key="opt.value"
        type="button"
        :class="[
          'px-3 py-1 rounded-full text-xs font-medium border transition-colors',
          sortBy === opt.value
            ? 'bg-blue-600 text-white border-blue-600'
            : 'bg-white text-gray-600 border-gray-300 hover:border-blue-400 hover:text-blue-600',
        ]"
        @click="sortBy = opt.value"
      >
        {{ opt.label }}
      </button>
    </div>
  </div>

  <!-- Loading State -->
  <div v-if="isLoading" class="flex justify-center py-8">
    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
  </div>

  <!-- Collections Card Grid -->
  <div v-else class="collections-section">
    <p class="text-slate-600 text-sm mb-6 max-w-2xl">
      <template v-if="auth.state.isLoggedIn">{{ t('collectionList.collectionDescription') }}</template>
      <i18n-t v-else keypath="collectionList.collectionDescriptionLoggedOut" tag="span">
        <RouterLink to="/login" class="text-blue-600 hover:text-blue-800 underline font-medium">{{ t('collectionList.loginTo') }}</RouterLink>
      </i18n-t>
    </p>
    <div class="collections-grid">
      <CollectionCard
        v-for="collection in collections"
        :key="collection.collection_id"
        :collection="collection"
        :study-loading="studyLoadingId === collection.collection_id"
        :format-date="formatDate"
        :study-button-label="t('collectionList.studyButton')"
        :studying-label="t('collectionList.studying')"
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
  </div>

  <!-- Empty State -->
  <div v-if="!isLoading && collections.length === 0"
    class="flex flex-col items-center justify-center text-center py-12 bg-gray-50 rounded-lg border border-blue-100">
    <button v-if="viewMode === 'my' && auth.state.isLoggedIn" class="mt-4 btn-aqua-emerald"
      @click="showCreateModal = true">
      <CirclePlus class="h-4 w-4" />
      <span>{{ t('collectionList.createFirstCollection') }}</span>
    </button>
  </div>

  <!-- Create Collection ModalComponent -->
  <div v-if="showCreateModal"
    class="z-[1000] fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4">
    <div class="bg-white rounded-lg max-w-md w-full p-6">
      <h3 class="text-lg font-semibold mb-4">
        {{ t('collectionList.createModalTitle') }}
      </h3>
      <form @submit.prevent="performCreateCollection">
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionList.nameLabel') }}</label>
            <input v-model="newCollection.name" type="text" required class="w-full input-field">
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('collectionList.descriptionLabel')
            }}</label>
            <textarea v-model="newCollection.description" rows="3" class="textarea-field" />
          </div>
          <div class="flex items-center gap-2">
            <input id="is_public" v-model="newCollection.is_public" type="checkbox" class="checkbox-toggle">
            <label for="is_public" class="text-sm text-gray-700">
              {{ t('collectionList.makePublicLabel') }}
            </label>
          </div>
        </div>

        <div class="mt-6 flex justify-end gap-3">
          <button type="button" class="btn-cancel" @click="showCreateModal = false">
            {{ t('collectionList.cancelButton') }}
          </button>
          <button type="submit" :disabled="isSubmitting" class="btn-create">
            {{ isSubmitting ? t('collectionList.creatingButton') : t('collectionList.createButton') }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup>
import { CirclePlus, Upload, BookOpen, ArrowBigRight } from 'lucide-vue-next';
import { ref, computed, onMounted, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';

import {
  getCollections,
  getPublicCollections,
  createCollection,
  importCollectionFull,
  getStreak,
  getLevels,
} from '@/api'
import { CollectionCard, Dropdown, IconButton } from '@packages/ui'
import { useAuth } from '@/composables/useAuth'
import { useSeoHead } from '@/composables/useSeoHead'

const auth = useAuth()
const router = useRouter()
const route = useRoute()

const VIEW_PARAM = 'view'
const VIEW_STORAGE_KEY = 'collections-view'
const validView = (v) => (v === 'my' || v === 'public' ? v : null)

const { t, locale } = useI18n()

// State: viewMode from URL param; when no param, from localStorage if set; else default by login
const initialView = () => {
  const fromUrl = validView(route.query[VIEW_PARAM])
  if (fromUrl && (fromUrl === 'public' || auth.state.isLoggedIn)) return fromUrl
  const fromStorage = validView(typeof localStorage !== 'undefined' ? localStorage.getItem(VIEW_STORAGE_KEY) : null)
  if (fromStorage != null && (fromStorage === 'public' || auth.state.isLoggedIn)) return fromStorage
  return auth.state.isLoggedIn ? 'my' : 'public'
}
const viewMode = ref(initialView())
const collections = ref([])
const isLoading = ref(true)
const sortBy = ref('active_week')
const showCreateModal = ref(false)
const isSubmitting = ref(false)
const importFileInput = ref(null)
const isImporting = ref(false)
const streakData = ref(null)
const isLoadingStreak = ref(false)
const studyLoadingId = ref(null)

const sortOptions = computed(() => [
  { value: 'active_week',  label: t('collectionList.sortActiveWeek') },
  { value: 'active_month', label: t('collectionList.sortActiveMonth') },
  { value: 'active_all',   label: t('collectionList.sortActiveAll') },
  { value: 'newest',       label: t('collectionList.sortNewest') },
])

const newCollection = ref({
  name: '',
  description: '',
  is_public: true,
}, locale.value)

const pageTitle = ref(t(viewMode.value === 'my' ? "collectionList.myCollections": "collectionList.publicCollections"))
useSeoHead({ title: pageTitle, locale: locale.value })

const fetchStreakData = async () => {
  if (!auth.state.isLoggedIn) return

  isLoadingStreak.value = true
  try {
    const response = await getStreak(7) // Get last 7 days
    streakData.value = response.data
  } catch (error) {
    console.error('Error fetching streak data:', error)
  } finally {
    isLoadingStreak.value = false
  }
}

const fetchCollections = async () => {
  isLoading.value = true
  try {
    // Only allow 'my' view mode when logged in
    if (!auth.state.isLoggedIn) {
      viewMode.value = 'public'
    }

    const params = { sort: sortBy.value }

    const response = await (viewMode.value === 'my' && auth.state.isLoggedIn
      ? getCollections(params)
      : getPublicCollections(params))
    collections.value = response.data.collections
  } catch (error) {
    console.error('Error fetching collections:', error)
  } finally {
    isLoading.value = false
  }
}

// Create new collection
const performCreateCollection = async () => {
  if (isSubmitting.value) return
  isSubmitting.value = true

  try {
    const response = await createCollection(newCollection.value)
    collections.value.unshift(response.data)
    showCreateModal.value = false
    newCollection.value = { name: '', description: '', is_public: true }
    router.push(`/collections/${response.data.collection_id}`)
  } catch (error) {
    console.error('Error creating collection:', error)
  } finally {
    isSubmitting.value = false
  }
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

const triggerImport = () => {
  importFileInput.value?.click()
}

const handleImportFile = async (event) => {
  const file = event.target.files?.[0]
  if (!file) return

  isImporting.value = true
  try {
    const fileContent = await file.text()
    const jsonData = JSON.parse(fileContent)

    if (!jsonData.collection || !Array.isArray(jsonData.items)) {
      alert(t('collectionList.importFullFormatError', 'Use a full collection export file (JSON with collection, items, and optionally levels).'))
      return
    }

    const payload = {
      collection: {
        name: jsonData.collection.name,
        description: jsonData.collection.description ?? null,
        is_public: jsonData.collection.is_public ?? true,
      },
      items: jsonData.items,
      levels: Array.isArray(jsonData.levels) ? jsonData.levels : [],
    }

    const response = await importCollectionFull(payload)
    const { collection: newCollection } = response.data
    collections.value.unshift(newCollection)
    router.push(`/collections/${newCollection.collection_id}`)
  } catch (error) {
    console.error('Import failed:', error)
    alert(error.response?.data?.error || t('collectionList.importError', 'Import failed'))
  } finally {
    isImporting.value = false
    event.target.value = ''
  }
}

function setViewMode(mode) {
  const next = mode === 'my' || mode === 'public' ? mode : viewMode.value
  if (!auth.state.isLoggedIn && next === 'my') return
  viewMode.value = next
  router.replace({ path: route.path, query: { ...route.query, [VIEW_PARAM]: next } })
}

// Persist viewMode to localStorage whenever it changes
watch(viewMode, (val) => {
  if (typeof localStorage !== 'undefined') localStorage.setItem(VIEW_STORAGE_KEY, val)
}, { immediate: true })

// Sync viewMode from URL when route query changes (e.g. back/forward)
watch(() => route.query[VIEW_PARAM], (qView) => {
  const next = validView(qView) && (qView === 'public' || auth.state.isLoggedIn) ? qView : (auth.state.isLoggedIn ? 'my' : 'public')
  if (viewMode.value !== next) viewMode.value = next
})

// Watch for view mode or sort changes
watch([viewMode, sortBy, () => auth.state.isLoggedIn], () => {
  // Force public view when logged out
  if (!auth.state.isLoggedIn) {
    viewMode.value = 'public'
  }
  fetchCollections()
})

// Update title when view mode changes
watch(viewMode, (newMode) => {
  pageTitle.value = t(newMode === 'my' ? "collectionList.myCollections": "collectionList.publicCollections")
})

onMounted(() => {
  // Persist initial view to URL so refresh and back/forward stay in sync
  if (route.query[VIEW_PARAM] !== viewMode.value) {
    router.replace({ path: route.path, query: { ...route.query, [VIEW_PARAM]: viewMode.value } })
  }
  fetchCollections()
  fetchStreakData()
})
</script>

<style scoped>
.collections-section {
  @apply w-full;
}
.collections-grid {
  @apply grid gap-4 sm:gap-5 items-stretch grid-cols-1 sm:grid-cols-2 lg:grid-cols-3;
}

.animate-fade-in-up {
  animation: fadeInUp 0.3s ease-out;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(1rem);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
