<template>
  <!-- Streak Stats -->
  <div v-if="auth.state.isLoggedIn" class="mb-6 h-36 bg-white p-4 rounded-lg border">
    <template v-if="!isLoadingStreak && streakData">
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-lg font-semibold text-gray-800">
          {{ t('collectionList.studyStreak') }}
        </h3>
        <div class="text-sm text-gray-600">
          {{ t('collectionList.currentStreak') }}: <span class="font-semibold">{{ t('collectionList.days', {
            count:
              streakData.current_streak
          }) }}</span>
        </div>
      </div>
      <div class="grid grid-cols-7 gap-2">
        <div v-for="day in streakData.daily_progress.slice(0, 7).reverse()" :key="day.date"
          class="flex flex-col items-center">
          <div class="text-xs text-gray-500 mb-1">
            {{ new Date(day.date).toLocaleDateString(locale, { weekday: 'short' }) }}
          </div>
          <div class="w-8 h-8 rounded-full flex items-center justify-center"
            :class="day.reviews_count > 0 ? 'bg-blue-100 text-blue-700' : 'bg-gray-100 text-gray-400'">
            {{ day.reviews_count }}
          </div>
          <div class="text-xs text-gray-500 mt-1">
            {{ t('collectionList.points', { count: day.points }) }}
          </div>
        </div>
      </div>
    </template>
    <div v-else class="animate-pulse h-36">
      <div class="flex items-center justify-between mb-4">
        <div class="h-6 bg-gray-200 rounded w-1/3" />
        <div class="h-4 bg-gray-100 rounded w-1/4" />
      </div>
      <div class="grid grid-cols-7 gap-2">
        <div v-for="i in 7" :key="i" class="flex flex-col items-center space-y-2">
          <div class="h-4 bg-gray-100 rounded w-full max-w-[40px]" />
          <div class="w-8 h-8 rounded-full bg-gray-100" />
          <div class="h-3 bg-gray-100 rounded w-full max-w-[30px]" />
        </div>
      </div>
    </div>
  </div>

  <!-- Header -->
  <div class="flex flex-col sm:flex-row justify-between items-center gap-2 space-x-2 mb-6">
    <h2 class="text-xl sm:text-2xl font-bold text-gray-800">
      {{ viewMode !== 'my' ? t('collectionList.publicCollections') : t('collectionList.myCollections') }}
    </h2>
    <div class="flex flex-row gap-2 justify-end flex-grow flex-wrap">
      <label v-if="auth.state.isLoggedIn" :class="[viewMode === 'my' ? ' btn-aqua-slate' : 'btn-aqua-white']">
        <input type="checkbox" class="checkmark-aqua" :checked="viewMode === 'my'"
          @click="viewMode = viewMode === 'my' ? 'public' : 'my'">
        <span> {{ t('collectionList.myCollectionsLabel') }} </span>
      </label>
      <input ref="importFileInput" type="file" accept=".json" class="hidden" @change="handleImportFile">
      <IconButton v-if="auth.state.isLoggedIn" :label="t('collectionList.importCollection')"
        button-classes="btn-aqua-sky flex-grow" :disabled="isImporting" @click="triggerImport" />
      <IconButton v-if="auth.state.isLoggedIn" :label="t('collectionList.createCollection')"
        button-classes="btn-aqua-emerald flex-grow" @click="showCreateModal = true" />
    </div>
  </div>
  <!-- Loading State -->
  <div v-if="isLoading" class="flex justify-center py-8">
    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
  </div>

  <!-- Collections Grid -->
  <div v-else class="grid gap-4">
    <p class="text-gray-600 text-sm mb-2">
      {{ auth.state.isLoggedIn ? t('collectionList.collectionDescription') :
        t('collectionList.collectionDescriptionLoggedOut') }}
    </p>
    <div v-for="collection in collections" :key="collection.collection_id"
      class="bg-white p-3 sm:p-4 border rounded-lg hover:border-blue-300 transition-colors">
      <!-- Main Content -->
      <div class="flex flex-col gap-3">
        <!-- Title and Description -->
        <div class="min-w-0">
          <RouterLink :to="`/collections/${collection.collection_id}`"
            class="block text-base font-semibold text-blue-600 hover:text-blue-800 hover:underline line-clamp-2 mb-1">
            {{ collection.name }}
          </RouterLink>
          <p v-if="collection.description" class="text-gray-600 text-sm line-clamp-2">
            {{ collection.description }}
          </p>
        </div>
      </div>
      <div class="flex flex-wrap items-center gap-4 text-sm justify-end">
        <!-- Action Buttons -->
          <button
            type="button"
            class="btn-aqua-sky"
            :disabled="studyLoadingId === collection.collection_id"
            @click="startStudy(collection)"
          >
            <GraduationCap v-if="studyLoadingId !== collection.collection_id" class="w-4 h-4" />
            <span v-else class="animate-spin inline-block w-4 h-4 border-2 border-current border-t-transparent rounded-full" />
            {{ studyLoadingId === collection.collection_id ? t('collectionList.studying') : t('collectionList.studyButton') }}
          </button>
          <RouterLink :to="`/collections/${collection.collection_id}/flashcards`" class="btn-aqua-orange">
            <GalleryHorizontalIcon class="w-4 h-4" />
            {{ t('collectionList.flashcardsButton') }}
          </RouterLink>
          <RouterLink :to="`/collections/${collection.collection_id}`" class="btn-aqua-purple">
            <List class="w-4 h-4" />
            {{ t('collectionList.collectionButton') }}
          </RouterLink>
      </div>

      <!-- Footer -->
      <div class="flex flex-wrap items-center justify-between text-xs sm:text-sm text-gray-500 mt-3 gap-2">
        <div class="flex items-center gap-2">
          <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs"
            :class="collection.is_public ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-700'">
            {{ collection.is_public ? t('collectionList.publicStatus') : t('collectionList.privateStatus') }}
          </span>
          <span class="text-xs text-gray-500"> {{ t('collectionList.itemsCount', { count: collection.item_count }) }}
          </span>
        </div>
        <div class="italic">
          {{ t('collectionList.createdBy') }}
          <RouterLink :to="`/user/${collection.owner.user_id}`"
            class="text-blue-600 hover:text-blue-800 hover:underline">
            {{ collection.owner.username }}
          </RouterLink>
        </div>
        <div>{{ t('collectionList.updated') }} {{ formatDate(collection.updated_at) }}</div>
      </div>
    </div>
  </div>

  <!-- Empty State -->
  <div v-if="!isLoading && collections.length === 0"
    class="text-center py-12 bg-gray-50 rounded-lg border border-blue-100">
    <button v-if="viewMode === 'my' && auth.state.isLoggedIn" class="mt-4 inline-flex items-center btn-aqua-emerald"
      @click="showCreateModal = true">
      <CirclePlus class="h-4 w-4" />
      {{ t('collectionList.createFirstCollection') }}
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
import { GalleryHorizontalIcon, List, CirclePlus, GraduationCap } from 'lucide-vue-next';
import { ref, onMounted, watch } from 'vue';
import { RouterLink, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';

import {
  getCollections,
  getPublicCollections,
  createCollection,
  importCollectionFull,
  getStreak,
  getLevels,
} from '@/api'
import IconButton from '@/components/icons/IconButton.vue'
import { useAuth } from '@/composables/useAuth'
import { useSeoHead } from '@/composables/useSeoHead'

const auth = useAuth()
const router = useRouter()

const { t, locale } = useI18n()

// State
const collections = ref([])
const isLoading = ref(true)
const viewMode = ref(auth.state.isLoggedIn ? 'my' : 'public')
const showCreateModal = ref(false)
const isSubmitting = ref(false)
const importFileInput = ref(null)
const isImporting = ref(false)
const streakData = ref(null)
const isLoadingStreak = ref(false)
const studyLoadingId = ref(null)

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

    const response = await (viewMode.value === 'my' && auth.state.isLoggedIn
      ? getCollections()
      : getPublicCollections())
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

// Watch for view mode changes
watch([viewMode, () => auth.state.isLoggedIn], () => {
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
  fetchCollections()
  fetchStreakData()
})
</script>

<style scoped>
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
