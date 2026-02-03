<template>
  <!-- Header -->
  <div class="bg-white border rounded-lg p-4 sm:p-6 mb-6">
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center space-x-3">
        <RouterLink :to="`/collections/${props.collectionId}`" class="btn-history">
          <ArrowLeft class="h-5 w-5" />
        </RouterLink>
        <h2 class="text-2xl font-bold">
          <span class="ml-1">{{ t('components.flashcardCollectionView.title', { collectionName: collection?.name }) }}</span>
        </h2>
      </div>
    </div>

    <template v-if="isOwner && !isLoading && collection?.item_count > existingFlashcardIds.size">
      <div class="flex flex-wrap justify-center md:justify-start gap-2 mb-4">
        <RouterLink
          :to="`/collections/${collection.collection_id}?mode=add_flashcard`"
          class="btn-aqua-emerald"
        >
          {{ t('components.flashcardCollectionView.addFlashcardButton') }}
        </RouterLink>
        <button class="btn-aqua-red w-auto" :disabled="isImporting" @click="handleImport">
          {{ isImporting ? t('components.flashcardCollectionView.importing') : t('components.flashcardCollectionView.importAllButton') }}
        </button>
      </div>
    </template>
    <div class="flex flex-row justify-center mb-2 gap-2">
      <button class="btn-aqua-orange h-10 text-base" :disabled="!dueCount" @click="startLearningSession">
        {{ t('flashcardCollection.studyNow', { count: dueCount }) }}
      </button>
    </div>
    <div v-if="collection?.description">
      <div class="max-h-32 text-sm overflow-y-auto border rounded mt-4 p-2 bg-gray-50 read-box">
        <LazyMathJax :content="collection.description" />
      </div>
    </div>
  </div>

  <!-- Stats Overview -->
  <div class="grid grid-cols-1 sm:grid-cols-4 gap-4 mb-6">
    <div class="bg-white p-4 rounded-lg border shadow-sm">
      <h3 class="text-sm font-medium text-gray-600">
        {{ t('components.flashcardCollectionView.stats.new') }}
      </h3>
      <p class="text-2xl font-bold text-blue-600">
        {{ stats.new }}
      </p>
    </div>
    <div class="bg-white p-4 rounded-lg border shadow-sm">
      <h3 class="text-sm font-medium text-gray-600">
        {{ t('components.flashcardCollectionView.stats.learning') }}
      </h3>
      <p class="text-2xl font-bold text-yellow-600">
        {{ stats.learning }}
      </p>
    </div>
    <div class="bg-white p-4 rounded-lg border shadow-sm">
      <h3 class="text-sm font-medium text-gray-600">
        {{ t('components.flashcardCollectionView.stats.review') }}
      </h3>
      <p class="text-2xl font-bold text-green-600">
        {{ stats.review }}
      </p>
    </div>
    <div class="bg-white p-4 rounded-lg border shadow-sm">
      <h3 class="text-sm font-medium text-gray-600">
        {{ t('components.flashcardCollectionView.stats.graduated') }}
      </h3>
      <p class="text-2xl font-bold text-purple-600">
        {{ stats.graduated }}
      </p>
    </div>
  </div>

  <!-- Filters -->
  <div class="bg-white p-4 rounded-lg border shadow-sm mb-6">
    <div class="flex flex-wrap gap-4">
      <select v-model="filters.status" class="input-field">
        <option value="">
          {{ t('components.flashcardCollectionView.filters.allStatus') }}
        </option>
        <option value="new">
          {{ t('components.flashcardCollectionView.stats.new') }}
        </option>
        <option value="learning">
          {{ t('components.flashcardCollectionView.stats.learning') }}
        </option>
        <option value="review">
          {{ t('components.flashcardCollectionView.stats.review') }}
        </option>
        <option value="graduated">
          {{ t('components.flashcardCollectionView.stats.graduated') }}
        </option>
      </select>
      <label class="flex items-center gap-2">
        <input v-model="filters.onlyDue" type="checkbox" class="checkbox-toggle">
        <span class="text-sm text-gray-700">{{ t('components.flashcardCollectionView.filters.dueCardsOnly') }}</span>
      </label>
    </div>
  </div>

  <!-- Flashcard List -->
  <LoadingSpinner v-if="isLoading" class="py-12" />

  <div v-else-if="flashcards.length === 0" class="text-center py-12 bg-gray-50 rounded-lg border border-blue-100">
    <p class="text-gray-600">
      {{ t('components.flashcardCollectionView.noFlashcards') }}
    </p>
  </div>

  <div v-else class="space-y-4">
    <div v-for="(card, index) in flashcards" :key="card.flashcard.id" :class="{ 'cursor-pointer': isOwner }"
      @click="isOwner && openFlashcard(card)"
      class="bg-white p-4 rounded-lg border hover:border-blue-300 shadow hover:shadow-none transition-all duration-200 max-w-full overflow-hidden">
      <!-- Card Content -->
      <div class="flex justify-between items-start gap-4">
        <div class="min-w-0 flex-1">
          <h3 class="text-lg font-medium text-gray-800">
            {{ card.flashcard.word ?? card.flashcard.free_content_front }}
          </h3>
          <div v-if="card.flashcard.canonical_form" class="text-sm text-gray-500 font-mono mt-1">
             {{ card.flashcard.canonical_form }}
          </div>
          <div class="text-sm text-gray-600 mt-1">
            <LazyMathJax :content="card.flashcard.definition ?? card.flashcard.free_content_back" />
          </div>
          <div v-if="card.flashcard.has_front_image || card.flashcard.has_back_image" class="mt-2">
            <div v-if="card.flashcard.has_front_image" class="mb-2">
              <img :src="`/api/collections/${card.flashcard.collection_id}/items/${card.flashcard.item_id}/image/front`"
                class="max-h-40 rounded-lg object-contain bg-gray-100" alt="Front image">
            </div>
            <div v-if="card.flashcard.has_back_image">
              <img :src="`/api/collections/${card.flashcard.collection_id}/items/${card.flashcard.item_id}/image/back`"
                class="max-h-40 rounded-lg object-contain bg-gray-100" alt="Back image">
            </div>
          </div>
          <div v-if="card.flashcard.notes" class="text-sm text-gray-500 mt-1">
            Notes:
            <LazyMathJax :content="card.flashcard.notes" :enable-markdown="true" />
          </div>
        </div>

        <!-- Progress section -->
        <div class="flex flex-col items-end gap-3">
          <div v-for="progress in card.progress" :key="progress.card_side" class="w-32">
            <div class="flex items-center justify-between mb-1">
              <span class="text-xs font-medium text-gray-600">{{ progress.card_side }}</span>
              <span class="text-xs font-medium" :class="getStatusTextClass(progress.status)">
                {{ progress.status }}
              </span>
            </div>
            <div class="w-full bg-gray-200 rounded-full h-2">
              <div class="h-2 rounded-full transition-all duration-300" :class="getProgressBarClass(progress.status)"
                :style="{ width: getProgressWidth(progress) }" />
            </div>
            <div v-if="progress.next_review_at" class="text-xs text-gray-500 mt-1 text-right">
              Next: {{ formatDate(progress.next_review_at) }}
            </div>
          </div>

          <div class="flex items-center gap-2 flex-wrap">
             <button class="btn-empty flex items-center gap-1.5 hover:bg-orange-50 text-orange-600" :title="t('components.flashcardCollectionView.reviewNowAction')" @click.stop="reviewSingleCard(card.flashcard.id)">
               <Repeat1 class="h-4 w-4" />
               <span class="sr-only">{{ t('components.flashcardCollectionView.reviewNowAction') }}</span>
             </button>
            <button :disabled="index === 0 || isReordering" class="btn-empty flex items-center gap-1.5" :class="[
              index === 0 || isReordering ? 'opacity-50 cursor-not-allowed' : 'hover:bg-gray-100',
            ]" :title="t('components.flashcardCollectionView.moveUpAction')" @click.stop="moveCard(card, 'up')">
              <ArrowUp class="h-4 w-4" />
              <span class="sr-only">{{ t('components.flashcardCollectionView.moveUpAction') }}</span>
            </button>

            <button :disabled="index === flashcards.length - 1 || isReordering"
              class="btn-empty flex items-center gap-1.5" :class="[
                index === flashcards.length - 1 || isReordering
                  ? 'opacity-50 cursor-not-allowed'
                  : 'hover:bg-gray-100',
              ]" :title="t('components.flashcardCollectionView.moveDownAction')" @click="moveCard(card, 'down')">
              <ArrowDown class="h-4 w-4" />
              <span class="sr-only">{{ t('components.flashcardCollectionView.moveDownAction') }}</span>
            </button>
          </div>
        </div>
      </div>
      <div class="text-sm text-gray-600 mt-1">
        {{ t('components.flashcardCollectionView.directionLabel') }}
        <span class="font-medium">
          {{ t(`flashcardCollection.directions.${card.flashcard.direction}`) }}
        </span>
      </div>
    </div>
  </div>

  <div v-if="totalPages > 1">
    <PaginationComponent :current-page="currentPage" :total-pages="totalPages" :total="flashcards.length"
      :per-page="perPage" @prev="handlePageChange(currentPage - 1)" @next="handlePageChange(currentPage + 1)" />
  </div>
</template>

<script setup>
import { ArrowUp, ArrowDown, ArrowLeft, Repeat1 } from 'lucide-vue-next'
import { ref, computed, onMounted, watch, onBeforeUnmount } from 'vue'
import { useRouter, RouterLink, useRoute } from 'vue-router'

import {
  getFlashcards,
  listCollectionItems,
  getLanguages,
  updateCardPosition,
  importFromCollection,
  getCollection,
} from '@/api'
import LazyMathJax from '@/components/LazyMathJax.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'
import { useI18n } from 'vue-i18n'
import { SearchQueue } from '@/utils/searchQueue'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'

const props = defineProps({
  collectionId: {
    type: [String, Number],
    required: true,
    validator: (value) => !isNaN(Number(value)),
  },
})

const auth = useAuth()
const route = useRoute()
const isOwner = computed(() => {
  return auth.state.isLoggedIn && collection.value?.owner?.username === auth.state.username
})

// State
const { showError } = useError()
const flashcards = ref([])
const isLoading = ref(true)
const isLoadingDefinitions = ref(true)
const definitions = ref([])
const searchQuery = ref('')
const isImporting = ref(false)
const successMessage = ref('')

const existingFlashcardIds = ref(new Set())

const handleImport = async () => {
  if (isImporting.value) return

  isImporting.value = true
  try {
    const response = await importFromCollection({
      collection_id: parseInt(props.collectionId),
    })

    successMessage.value = t('components.flashcardCollectionView.importSuccess', { importedCount: response.data.imported_count, skippedCount: response.data.skipped_count })

    // Refresh flashcards list to show new cards
    await loadFlashcards()
    setTimeout(() => {
      successMessage.value = ''
    }, 3000)
  } catch (error) {
    console.error('Error importing collection:', error)
  } finally {
    isImporting.value = false
  }
}

const selectedDefinition = ref(null)
const newCard = ref({
  notes: '',
  direction: '',
  frontImage: null,
  backImage: null,
})

const dueCount = ref(0)
const stats = ref({
  new: 0,
  learning: 0,
  review: 0,
  graduated: 0,
})

const filters = ref({
  status: '',
  onlyDue: false,
})


const currentPage = ref(parseInt(route.query.page) || 1)
const perPage = ref(10)
const totalPages = ref(1)

const handlePageChange = async (page) => {
  if (page === currentPage.value) return

  router.push({
    query: {
      ...route.query,
      page: page > 1 ? page : undefined,
    },
  })
  window.scrollTo(0, 0)
}

const loadFlashcards = async () => {
  isLoading.value = true
  try {
    const response = await getFlashcards({
      collection_id: props.collectionId,
      status: filters.value.status || undefined,
      due: filters.value.onlyDue || undefined,
      page: currentPage.value,
      per_page: perPage.value,
    })

    flashcards.value = response.data.flashcards
    dueCount.value = response.data.due_count
    totalPages.value = Math.ceil(response.data.total / perPage.value)
    existingFlashcardIds.value = new Set(
      response.data.flashcards.map((f) => f.flashcard.definition_id)
    )
    updateStats()
  } catch (error) {
    console.error('Error loading flashcards:', error)
  } finally {
    isLoading.value = false
  }
}

// Search debouncing
let searchTimeout = null

// Debounce delay: 450ms is optimal for search inputs (400-500ms range)
// This balances responsiveness with reducing unnecessary API calls
const DEBOUNCE_DELAY = 450

function clearSearchTimeout() {
  if (searchTimeout) {
    clearTimeout(searchTimeout)
    searchTimeout = null
  }
}

// Search queue to prevent race conditions
const definitionsSearchQueue = new SearchQueue()

const debouncedSearch = () => {
  // Clear any pending timeouts to prevent stale searches
  clearSearchTimeout()
  
  // Capture current query value to check in timeout
  const currentQuery = searchQuery.value
  
  // Debounce the search - only trigger after user stops typing
  // This prevents excessive API calls while user is actively typing
  searchTimeout = setTimeout(() => {
    // Only perform search if query hasn't changed (to prevent race conditions)
    if (searchQuery.value === currentQuery) {
      currentPage.value = 1 // Reset to first page when searching
      loadDefinitions()
    }
    searchTimeout = null
  }, DEBOUNCE_DELAY)
}

const modalCurrentPage = ref(1)
const modalItemsPerPage = ref(10)

const loadDefinitions = async (page = modalCurrentPage.value) => {
  isLoadingDefinitions.value = true
  
  let requestId = null
  const request = definitionsSearchQueue.createRequest()
  requestId = request.requestId
  const { signal } = request
  
  try {
    const response = await listCollectionItems(props.collectionId, {
      page,
      per_page: modalItemsPerPage.value,
      search: normalizeSearchQuery(searchQuery.value) || undefined,
    }, signal)

    // Only process if this is still the latest request
    if (!definitionsSearchQueue.shouldProcess(requestId)) {
      return
    }

    definitions.value = response.data.items.map((item) => ({
      ...item,
      definitionid: item.definition_id,
      word: item.word,
      definition: item.definition,
    }))
  } catch (err) {
    // Ignore abort errors
    if (err.name === 'AbortError' || err.code === 'ERR_CANCELED' || err.message?.includes('canceled')) {
      return
    }
    
    // Only show errors for the latest request
    if (definitionsSearchQueue.shouldProcess(requestId)) {
      console.error('Error loading definitions:', err)
      showError('Failed to load definitions')
    }
  } finally {
    // Only update loading state if this is still the latest request
    if (requestId && definitionsSearchQueue.shouldProcess(requestId)) {
      isLoadingDefinitions.value = false
    } else if (!definitionsSearchQueue.hasActiveRequest()) {
      isLoadingDefinitions.value = false
    }
  }
}

// Update stats from flashcards data
const updateStats = () => {
  stats.value = {
    new: 0,
    learning: 0,
    review: 0,
    graduated: 0,
  }

  flashcards.value.forEach((card) => {
    stats.value[card.progress[0].status.toLowerCase()]++
  })
}

const openFlashcard = (card) => {
  router.push({
    path: `/collections/${props.collectionId}`,
    query: { 
      editItem: card.flashcard.item_id,
    }
  })
}

watch(
  filters,
  () => {
    currentPage.value = 1
    loadFlashcards()
  },
  { deep: true }
)

// Format helpers
const formatDate = (date) => {
  if (!date) return 'Not scheduled'
  return new Date(date).toLocaleDateString(locale.value, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

const getProgressWidth = (progress) => {
  const progressMap = {
    new: '0%',
    learning: '33%',
    review: '66%',
    graduated: '100%',
  }
  return progressMap[progress.status] || '0%'
}

const getStatusTextClass = (status) => {
  const classes = {
    new: 'text-blue-600',
    learning: 'text-yellow-600',
    review: 'text-green-600',
    graduated: 'text-purple-600',
  }
  return classes[status] || 'text-gray-600'
}

const getProgressBarClass = (status) => {
  const classes = {
    new: 'bg-blue-500',
    learning: 'bg-yellow-500',
    review: 'bg-green-500',
    graduated: 'bg-purple-500',
  }
  return classes[status] || 'bg-gray-400'
}

watch(searchQuery, () => {
  currentPage.value = 1 // Reset page when search changes
})

const languages = ref([])

const loadLanguages = async () => {
  try {
    const response = await getLanguages()
    languages.value = response.data
  } catch (error) {
    console.error('Error loading languages:', error)
  }
}

const collection = ref(null)

// Sync page from URL
const syncFromRoute = () => {
  currentPage.value = parseInt(route.query.page) || 1
}

watch(
  () => route.query.page,
  (newPage) => {
    const pageNum = parseInt(newPage) || 1
    if (pageNum !== currentPage.value) {
      syncFromRoute()
      loadFlashcards()
    }
  }
)

const { t, locale } = useI18n()
const pageTitle = ref('Flashcards')
useSeoHead({ title: pageTitle }, locale.value)

onMounted(async () => {
  syncFromRoute()
  try {
    const response = await getCollection(props.collectionId)
    collection.value = response.data
    pageTitle.value = collection.value.name + ' - flashcards'
  } catch (error) {
    console.error('Error fetching collection:', error)
  }

  await Promise.all([loadFlashcards(), loadLanguages()])
})

onBeforeUnmount(() => {
  // Clean up any pending search timeout
  clearSearchTimeout()
})

const router = useRouter()

const startLearningSession = () => {
  router.push(`/collections/${props.collectionId}/flashcards/study`)
}

const isReordering = ref(false)

const isShowingDueCards = computed(() => {
  return filters.value.onlyDue
})

const moveCard = async (card, direction) => {
  if (isReordering.value) return

  isReordering.value = true
  const currentIndex = flashcards.value.findIndex((c) => c.flashcard.id === card.flashcard.id)
  const newIndex = direction === 'up' ? currentIndex - 1 : currentIndex + 1

  try {
    // Get current and target positions
    const currentPosition = card.flashcard.position
    const targetPosition = direction === 'up' ? currentPosition - 1 : currentPosition + 1

    // Update position on server
    await updateCardPosition(card.flashcard.id, targetPosition)

    // Optimistically update local state
    const cards = [...flashcards.value]
    const [movedCard] = cards.splice(currentIndex, 1)
    movedCard.flashcard.position = targetPosition
    cards.splice(newIndex, 0, movedCard)

    // Update flashcards with new order
    flashcards.value = cards.sort((a, b) => a.flashcard.position - b.flashcard.position)
  } catch (error) {
    console.error('Error reordering flashcard:', error)
    // Show error notification if needed
  } finally {
    isReordering.value = false
  }
}

const reviewSingleCard = (flashcardId) => {
  router.push(`/collections/${props.collectionId}/flashcards/study?card_id=${flashcardId}`)
}
</script>

<style scoped>
.btn-previous,
.btn-next {
  @apply px-4 py-2 text-sm border rounded-md;
}

.btn-previous:disabled,
.btn-next:disabled {
  @apply opacity-50 cursor-not-allowed;
}

.btn-previous:not(:disabled),
.btn-next:not(:disabled) {
  @apply hover:bg-gray-50;
}

.progress-bar {
  transition: width 0.3s ease-in-out;
}
</style>
