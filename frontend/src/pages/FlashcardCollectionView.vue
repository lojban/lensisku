<template>
   <!-- Header: row order matches CollectionDetail.vue (Title → Description → meta) -->
  <div class="bg-white border rounded-lg p-4 sm:p-6 mb-6 flex flex-col gap-2">
     <!-- Skeleton: same layout shape to avoid CLS while loading --> <template
      v-if="isHeaderLoading"
      >
      <div class="h-4 w-20 bg-gray-200 animate-pulse rounded" aria-hidden="true" />

      <div class="h-8 w-72 max-w-full bg-gray-200 animate-pulse rounded" aria-hidden="true" />

      <div class="flex flex-wrap gap-2">

        <div class="h-6 w-16 bg-gray-200 animate-pulse rounded" />

        <div class="h-6 w-24 bg-gray-200 animate-pulse rounded" />

      </div>

      <div class="flex flex-wrap justify-center md:justify-start gap-2">

        <div class="h-10 w-24 bg-gray-200 animate-pulse rounded" />

        <div class="h-10 w-28 bg-gray-200 animate-pulse rounded" />

        <div class="h-10 w-20 bg-gray-200 animate-pulse rounded" />

      </div>
       </template
    > <template v-else
      > <!-- Row 1: Hint (flashcards) + Title -->
      <div class="flex items-center gap-2 text-gray-500 italic text-sm mb-1">
         <GalleryHorizontalIcon class="w-5 h-5 shrink-0" aria-hidden="true" /> <span>{{
          t('components.flashcardCollectionView.pageHint')
        }}</span
        >
      </div>

      <h2 class="text-xl sm:text-2xl font-bold text-gray-800">
         {{ t('components.flashcardCollectionView.title', { collectionName: collection?.name }) }}
      </h2>
       <!-- Row 2: Description -->
      <div v-if="collection?.description" class="w-full">

        <div class="max-h-32 text-sm overflow-y-auto border rounded p-2 w-full read-box">
           <LazyMathJax :content="collection.description" />
        </div>

      </div>
       <!-- Row 3: public + owner + count -->
      <div class="flex flex-row gap-2 items-center text-sm text-gray-500">

        <div class="flex flex-wrap items-center gap-2">
           <span
            class="text-sm px-2 py-1 rounded-full select-none shrink-0"
            :class="
              collection?.is_public ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-700'
            "
            > {{
              collection?.is_public ? t('collectionDetail.public') : t('collectionDetail.private')
            }} </span
          > <span v-if="collection?.owner"
            > {{ t('collectionDetail.createdBy') }} <RouterLink
              :to="`/user/${collection.owner.username}`"
              class="text-blue-600 hover:text-blue-800 hover:underline"
              > {{ collection.owner.username }} </RouterLink
            > </span
          > <span>{{
            t('collectionDetail.itemsCount', { count: collection?.item_count ?? 0 })
          }}</span
          >
        </div>

      </div>
       <!-- Row 4: Action buttons -->
      <div class="space-y-4 md:space-y-0">

        <div class="flex flex-wrap items-center gap-2 w-auto">

          <div class="btn-group-forced flex flex-wrap items-center md:gap-y-2" role="group">
             <RouterLink
              :to="`/collections/${props.collectionId}`"
              class="ui-btn--neutral-muted ui-btn--group-item md:flex-none"
              > <List class="w-4 h-4 shrink-0" aria-hidden="true" /> {{
                t('components.flashcardCollectionView.collectionButton')
              }} </RouterLink
            > <template v-if="isOwner && collection?.item_count > existingFlashcardIds.size"
              > <RouterLink
                :to="`/collections/${collection.collection_id}?mode=add_flashcard`"
                class="ui-btn--create ui-btn--group-item md:flex-none"
                > <PlusCircle class="w-4 h-4 shrink-0" aria-hidden="true" /> {{
                  t('components.flashcardCollectionView.addFlashcardButton')
                }} </RouterLink
              > <button
                class="ui-btn--delete ui-btn--group-item md:flex-none"
                :disabled="isImporting"
                @click="handleImport"
              >
                 <Import class="w-4 h-4 shrink-0" aria-hidden="true" /> {{
                  isImporting
                    ? t('components.flashcardCollectionView.importing')
                    : t('components.flashcardCollectionView.importAllButton')
                }} </button
              > </template
            > <template v-if="!auth.state.isLoggedIn"
              >
              <div class="mt-2 gap-4">
                 <RouterLink
                  :to="`/collections/${props.collectionId}/levels`"
                  class="ui-btn--warning-orange ui-btn--group-item md:flex-none"
                  > {{ t('anonymousProgress.viewLevels') }} </RouterLink
                > <RouterLink
                  :to="`/collections/${props.collectionId}/levels`"
                  class="ui-btn--create ui-btn--group-item md:flex-none"
                  > {{ t('anonymousProgress.studyLevels') }} </RouterLink
                >
              </div>
               </template
            >
          </div>

        </div>

      </div>
       <!-- Row 5: Study (full session) + Levels, centered, not grouped -->
      <div
        v-if="auth.state.isLoggedIn"
        class="flex flex-row justify-center items-center gap-2 mt-4"
      >
         <Button
          variant="warning-orange"
          size="lg"
          :to="`/collections/${props.collectionId}/flashcards/study`"
          > {{ t('flashcardCollection.studyNow', { count: dueCount }) }} </Button
        > <RouterLink
          :to="`/collections/${props.collectionId}/levels`"
          class="ui-btn--neutral inline-flex items-center gap-2"
          > <LayoutPanelTop class="w-4 h-4 shrink-0" aria-hidden="true" /> {{
            t('collectionDetail.levels')
          }} </RouterLink
        >
      </div>
       </template
    >
  </div>
   <!-- Anonymous: sign-in prompt -->
  <div
    v-if="isAnonView"
    class="bg-amber-50 border border-amber-200 rounded-lg p-4 mb-6 text-center"
  >

    <p class="text-gray-700 mb-3"> {{ t('anonymousProgress.signInToSaveProgress') }} </p>

    <p class="text-sm text-gray-600"> {{ t('anonymousProgress.studyByLevelHint') }} </p>

  </div>
   <!-- Stats Overview -->
  <div v-if="!isAnonView" class="grid grid-cols-1 sm:grid-cols-4 gap-4 mb-6">

    <div class="bg-white p-4 rounded-lg border shadow-sm">

      <h3 class="text-sm font-medium text-gray-600">
         {{ t('components.flashcardCollectionView.stats.new') }}
      </h3>

      <p class="text-2xl font-bold text-blue-600"> {{ stats.new }} </p>

    </div>

    <div class="bg-white p-4 rounded-lg border shadow-sm">

      <h3 class="text-sm font-medium text-gray-600">
         {{ t('components.flashcardCollectionView.stats.learning') }}
      </h3>

      <p class="text-2xl font-bold text-yellow-600"> {{ stats.learning }} </p>

    </div>

    <div class="bg-white p-4 rounded-lg border shadow-sm">

      <h3 class="text-sm font-medium text-gray-600">
         {{ t('components.flashcardCollectionView.stats.review') }}
      </h3>

      <p class="text-2xl font-bold text-green-600"> {{ stats.review }} </p>

    </div>

    <div class="bg-white p-4 rounded-lg border shadow-sm">

      <h3 class="text-sm font-medium text-gray-600">
         {{ t('components.flashcardCollectionView.stats.graduated') }}
      </h3>

      <p class="text-2xl font-bold text-purple-600"> {{ stats.graduated }} </p>

    </div>

  </div>
   <!-- Filters -->
  <div v-if="!isAnonView" class="bg-white p-4 rounded-lg border shadow-sm mb-6">

    <div class="flex flex-wrap gap-4">
       <select v-model="filters.status" class="input-field">

        <option value=""> {{ t('components.flashcardCollectionView.filters.allStatus') }} </option>

        <option value="new"> {{ t('components.flashcardCollectionView.stats.new') }} </option>

        <option value="learning">
           {{ t('components.flashcardCollectionView.stats.learning') }}
        </option>

        <option value="review"> {{ t('components.flashcardCollectionView.stats.review') }} </option>

        <option value="graduated">
           {{ t('components.flashcardCollectionView.stats.graduated') }}
        </option>
         </select
      > <label class="flex items-center gap-2"
        > <input v-model="filters.onlyDue" type="checkbox" class="checkbox-toggle" /> <span
          class="text-sm text-gray-700"
          >{{ t('components.flashcardCollectionView.filters.dueCardsOnly') }}</span
        > </label
      >
    </div>

  </div>
   <!-- Flashcard List --> <LoadingSpinner v-if="!isAnonView && isLoading" class="py-12" />
  <div v-else-if="isAnonView" class="text-center py-8 bg-gray-50 rounded-lg border border-blue-100">

    <p class="text-gray-600 mb-4">{{ t('anonymousProgress.useLevelsToStudy') }}</p>

  </div>

  <div
    v-else-if="flashcards.length === 0"
    class="text-center py-12 bg-gray-50 rounded-lg border border-blue-100"
  >

    <p class="text-gray-600"> {{ t('components.flashcardCollectionView.noFlashcards') }} </p>

  </div>

  <div v-else class="space-y-4">

    <div
      v-for="(card, index) in flashcards"
      :key="card.flashcard.id"
      :class="{ 'cursor-pointer': isOwner }"
      @click="isOwner && openFlashcard(card)"
      class="bg-white p-4 rounded-lg border hover:border-blue-300 shadow hover:shadow-none transition-all duration-200 max-w-full overflow-hidden"
    >
       <!-- Card Content -->
      <div class="flex justify-between items-start gap-4">

        <div class="min-w-0 flex-1">

          <div class="flex flex-wrap items-center gap-2 min-w-0">

            <h3 class="text-lg font-medium text-gray-800">
               {{ card.flashcard.word ?? card.flashcard.free_content_front }}
            </h3>
             <span v-if="card.flashcard.sound_url" class="shrink-0" @click.stop
              > <AudioPlayer
                :url="card.flashcard.sound_url"
                :collection-id="
                  card.flashcard.sound_url?.startsWith?.('/api/')
                    ? card.flashcard.collection_id
                    : undefined
                "
                :item-id="
                  card.flashcard.sound_url?.startsWith?.('/api/')
                    ? card.flashcard.item_id
                    : undefined
                "
                class="shrink-0"
              /> </span
            >
          </div>

          <div v-if="showCanonicalForCard(card)" class="mt-2 flex flex-col gap-1">

            <div
              class="flex items-center gap-2 text-xs font-semibold text-gray-400 uppercase tracking-wider"
            >
               <EqualApproximately class="h-3.5 w-3.5 text-blue-400 shrink-0" /> <span>{{
                t('components.definitionCard.canonicalLabel')
              }}</span
              >
            </div>

            <div
              class="text-sm text-gray-700 font-mono bg-blue-50/30 p-2 rounded border border-blue-100/30"
            >
               {{ card.flashcard.canonical_form }}
            </div>

          </div>

          <div class="text-sm text-gray-600 mt-1">
             <LazyMathJax
              :content="card.flashcard.definition ?? card.flashcard.free_content_back"
            />
          </div>

          <div v-if="card.flashcard.has_front_image || card.flashcard.has_back_image" class="mt-2">

            <div v-if="card.flashcard.has_front_image" class="mb-2">
               <img
                :src="`/api/collections/${card.flashcard.collection_id}/items/${card.flashcard.item_id}/image/front`"
                class="max-h-40 rounded-lg object-contain bg-gray-100"
                alt="Front image"
              />
            </div>

            <div v-if="card.flashcard.has_back_image">
               <img
                :src="`/api/collections/${card.flashcard.collection_id}/items/${card.flashcard.item_id}/image/back`"
                class="max-h-40 rounded-lg object-contain bg-gray-100"
                alt="Back image"
              />
            </div>

          </div>

          <div v-if="card.flashcard.notes" class="text-sm text-gray-500 mt-1">
             Notes: <LazyMathJax :content="card.flashcard.notes" :enable-markdown="true" />
          </div>

        </div>
         <!-- Progress section -->
        <div class="flex flex-col items-end gap-3">

          <div v-for="progress in card.progress" :key="progress.card_side" class="w-32">

            <div class="flex items-center justify-between mb-1">
               <span class="text-xs font-medium text-gray-600">{{ progress.card_side }}</span
              > <span class="text-xs font-medium" :class="getStatusTextClass(progress.status)"
                > {{ progress.status }} </span
              >
            </div>

            <div class="w-full bg-gray-200 rounded-full h-2">

              <div
                class="h-2 rounded-full transition-all duration-300"
                :class="getProgressBarClass(progress.status)"
                :style="{ width: getProgressWidth(progress) }"
              />

            </div>

            <div v-if="progress.next_review_at" class="text-xs text-gray-500 mt-1 text-right">
               Next: {{ formatDate(progress.next_review_at) }}
            </div>

          </div>

          <div class="flex items-center gap-2 flex-wrap">
             <button
              class="ui-btn--empty flex items-center gap-1.5 hover:bg-orange-50 text-orange-600"
              :title="t('components.flashcardCollectionView.reviewNowAction')"
              @click.stop="reviewSingleCard(card.flashcard.id)"
            >
               <Repeat1 class="h-4 w-4" /> <span class="sr-only">{{
                t('components.flashcardCollectionView.reviewNowAction')
              }}</span
              > </button
            > <button
              :disabled="index === 0 || isReordering"
              class="ui-btn--empty flex items-center gap-1.5"
              :class="[
                index === 0 || isReordering ? 'opacity-50 cursor-not-allowed' : 'hover:bg-gray-100',
              ]"
              :title="t('components.flashcardCollectionView.moveUpAction')"
              @click.stop="moveCard(card, 'up')"
            >
               <ArrowUp class="h-4 w-4" /> <span class="sr-only">{{
                t('components.flashcardCollectionView.moveUpAction')
              }}</span
              > </button
            > <button
              :disabled="index === flashcards.length - 1 || isReordering"
              class="ui-btn--empty flex items-center gap-1.5"
              :class="[
                index === flashcards.length - 1 || isReordering
                  ? 'opacity-50 cursor-not-allowed'
                  : 'hover:bg-gray-100',
              ]"
              :title="t('components.flashcardCollectionView.moveDownAction')"
              @click="moveCard(card, 'down')"
            >
               <ArrowDown class="h-4 w-4" /> <span class="sr-only">{{
                t('components.flashcardCollectionView.moveDownAction')
              }}</span
              > </button
            >
          </div>

        </div>

      </div>

      <div class="text-sm text-gray-600 mt-1">
         {{ t('components.flashcardCollectionView.directionLabel') }} <span class="font-medium"
          > {{ t(`flashcardCollection.directions.${card.flashcard.direction}`) }} </span
        >
      </div>

    </div>

  </div>

  <div v-if="totalPages > 1">
     <PaginationComponent
      :current-page="currentPage"
      :total-pages="totalPages"
      :total="flashcards.length"
      :per-page="perPage"
      @prev="handlePageChange(currentPage - 1)"
      @next="handlePageChange(currentPage + 1)"
    />
  </div>

</template>

<script setup lang="ts">
import {
  ArrowUp,
  ArrowDown,
  Repeat1,
  EqualApproximately,
  GalleryHorizontalIcon,
  List,
  LayoutPanelTop,
  PlusCircle,
  Import,
} from 'lucide-vue-next'
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
import AudioPlayer from '@/components/AudioPlayer.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import { Button } from '@packages/ui'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'
import { useI18n } from 'vue-i18n'
import { SearchQueue } from '@/utils/searchQueue'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'
import { queryStr } from '@/utils/routeQuery'

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
      collection_id: parseInt(String(props.collectionId), 10),
    })

    successMessage.value = t('components.flashcardCollectionView.importSuccess', {
      importedCount: response.data.imported_count,
      skippedCount: response.data.skipped_count,
    })

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

const currentPage = ref(parseInt(queryStr(route.query.page), 10) || 1)
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
    const response = await listCollectionItems(
      props.collectionId,
      {
        page,
        per_page: modalItemsPerPage.value,
        search: normalizeSearchQuery(searchQuery.value) || undefined,
      },
      signal
    )

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
    if (
      err.name === 'AbortError' ||
      err.code === 'ERR_CANCELED' ||
      err.message?.includes('canceled')
    ) {
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

const showCanonicalForCard = (card) => {
  const fc = card?.flashcard
  if (!fc?.canonical_form) return false
  const main = (fc.word ?? fc.free_content_front ?? '').trim().toLowerCase()
  const canonical = fc.canonical_form.trim().toLowerCase()
  return main !== canonical
}

const openFlashcard = (card) => {
  router.push({
    path: `/collections/${props.collectionId}`,
    query: {
      editItem: card.flashcard.item_id,
    },
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
  currentPage.value = parseInt(queryStr(route.query.page), 10) || 1
}

watch(
  () => route.query.page,
  (newPage) => {
    const pageNum = parseInt(queryStr(newPage), 10) || 1
    if (pageNum !== currentPage.value) {
      syncFromRoute()
      loadFlashcards()
    }
  }
)

const { t, locale } = useI18n()
const pageTitle = ref('Flashcards')
useSeoHead({ title: pageTitle })

const isAnonView = computed(() => !auth.state.isLoggedIn)

/** True while collection or flashcards are loading; used to show header skeleton and avoid CLS */
const isHeaderLoading = computed(() => !collection.value || isLoading.value)

onMounted(async () => {
  syncFromRoute()
  try {
    const response = await getCollection(String(props.collectionId))
    collection.value = response.data
    pageTitle.value = collection.value.name
  } catch (error) {
    console.error('Error fetching collection:', error)
  }

  if (auth.state.isLoggedIn) {
    await Promise.all([loadFlashcards(), loadLanguages()])
  } else {
    await loadLanguages()
  }
})

onBeforeUnmount(() => {
  // Clean up any pending search timeout
  clearSearchTimeout()
})

const router = useRouter()

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
.ui-btn--previous,
.ui-btn--next {
  @apply px-4 py-2 text-sm border rounded-md;
}

.ui-btn--previous:disabled,
.ui-btn--next:disabled {
  @apply opacity-50 cursor-not-allowed;
}

.ui-btn--previous:not(:disabled),
.ui-btn--next:not(:disabled) {
  @apply hover:bg-gray-50;
}

.progress-bar {
  transition: width 0.3s ease-in-out;
}
</style>

