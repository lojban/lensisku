<template>
  <CollectionPageHeader
    :loading="!collection"
    :collection="collection"
    :cover-image-url="collectionCoverDisplayUrl"
    :cover-image-alt="
      collection ? t('collectionDetail.coverImageAlt', { name: collection.name }) : ''
    "
    :cover-lightbox-dialog-label="
      collection ? t('collectionDetail.coverLightboxDialog', { name: collection.name }) : ''
    "
    :cover-lightbox-close-label="t('collectionDetail.coverLightboxClose')"
    :public-label="t('collectionDetail.public')"
    :private-label="t('collectionDetail.private')"
    :created-by-label="t('collectionDetail.createdBy')"
    :items-count-label="t('collectionDetail.itemsCount', { count: collection?.item_count ?? 0 })"
  >
    <template #hint>
      <div class="flex items-center gap-2 text-gray-500 italic text-sm mb-1">
        <GalleryHorizontalEnd class="w-5 h-5 shrink-0" aria-hidden="true" />
        <span>{{ t('components.flashcardCollectionView.pageHint') }}</span>
      </div>
    </template>
    <template #title>
      {{ t('components.flashcardCollectionView.title', { collectionName: collection?.name }) }}
    </template>
    <template #toolbar>
      <div class="space-y-4 md:space-y-0">
        <div class="flex w-full flex-wrap items-center justify-center gap-2">
          <div class="btn-group-forced flex flex-wrap items-center md:gap-y-2" role="group">
            <IconButton
              tag="router-link"
              :to="`/collections/${props.collectionId}`"
              :label="t('components.flashcardCollectionView.collectionButton')"
              button-classes="ui-btn--neutral-muted ui-btn--group-item md:flex-none"
            >
              <template #icon> <List class="w-4 h-4 shrink-0" aria-hidden="true" /> </template>
            </IconButton>
            <IconButton
              v-if="collection?.has_collection_image"
              tag="router-link"
              :to="`/collections/${props.collectionId}/tiktoknu`"
              :label="t('components.flashcardCollectionView.viewAsTiktoknu')"
              button-classes="ui-btn--accent-purple ui-btn--group-item"
            >
              <template #icon>
                <GalleryVerticalIcon class="w-4 h-4 shrink-0" aria-hidden="true" />
              </template>
            </IconButton>
          </div>

          <div
            v-if="isOwner && collection && collection.item_count > existingFlashcardIds.size"
            class="btn-group-forced flex flex-wrap items-center md:gap-y-2"
            role="group"
          >
            <IconButton
              tag="router-link"
              :to="`/collections/${collection.collection_id}?mode=add_flashcard`"
              :label="t('components.flashcardCollectionView.addFlashcardButton')"
              button-classes="ui-btn--create ui-btn--group-item md:flex-none"
            >
              <template #icon>
                <PlusCircle class="w-4 h-4 shrink-0" aria-hidden="true" />
              </template>
            </IconButton>
            <IconButton
              :label="
                isImporting
                  ? t('components.flashcardCollectionView.importing')
                  : t('components.flashcardCollectionView.importAllButton')
              "
              button-classes="ui-btn--delete ui-btn--group-item md:flex-none"
              :disabled="isImporting"
              @click="handleImport"
            >
              <template #icon> <Import class="w-4 h-4 shrink-0" aria-hidden="true" /> </template>
            </IconButton>
          </div>
          <template v-if="!auth.state.isLoggedIn">
            <div class="mt-2 gap-4 flex flex-wrap">
              <IconButton
                tag="router-link"
                :to="`/collections/${props.collectionId}/levels`"
                :label="t('anonymousProgress.viewLevels')"
                button-classes="ui-btn--warning-orange ui-btn--group-item md:flex-none"
              >
                <template #icon>
                  <LayoutPanelTop class="w-4 h-4 shrink-0" aria-hidden="true" />
                </template>
              </IconButton>
              <IconButton
                tag="router-link"
                :to="`/collections/${props.collectionId}/levels`"
                :label="t('anonymousProgress.studyLevels')"
                button-classes="ui-btn--create ui-btn--group-item md:flex-none"
              >
                <template #icon> <Repeat1 class="w-4 h-4 shrink-0" aria-hidden="true" /> </template>
              </IconButton>
            </div>
          </template>
        </div>
      </div>

      <div
        v-if="auth.state.isLoggedIn && collection"
        class="flex flex-row justify-center items-center gap-2 mt-4"
      >
        <IconButton
          tag="router-link"
          size="lg"
          :to="`/collections/${props.collectionId}/flashcards/study`"
          :label="t('flashcardCollection.studyNow', { count: dueCount })"
          button-classes="ui-btn--warning-orange"
        >
          <template #icon> <Repeat1 class="w-4 h-4 shrink-0" aria-hidden="true" /> </template>
        </IconButton>
        <IconButton
          tag="router-link"
          :to="`/collections/${props.collectionId}/levels`"
          :label="t('collectionDetail.levels')"
          button-classes="ui-btn--neutral"
        >
          <template #icon>
            <LayoutPanelTop class="w-4 h-4 shrink-0" aria-hidden="true" />
          </template>
        </IconButton>
      </div>
    </template>
  </CollectionPageHeader>
  <!-- Anonymous: sign-in prompt -->
  <div v-if="isAnonView" class="bg-amber-50 border border-amber-200 rounded-lg p-4 text-center">
    <p class="text-gray-700 mb-3">{{ t('anonymousProgress.signInToSaveProgress') }}</p>

    <p class="text-sm text-gray-600">{{ t('anonymousProgress.studyByLevelHint') }}</p>
  </div>
  <!-- Stats Overview -->
  <div v-if="!isAnonView" class="grid grid-cols-1 sm:grid-cols-4 gap-4">
    <div class="bg-white p-4 rounded-lg border shadow-sm">
      <h3 class="text-sm font-medium text-gray-600">
        {{ t('components.flashcardCollectionView.stats.new') }}
      </h3>

      <p class="text-2xl font-bold text-blue-600">{{ stats.new }}</p>
    </div>

    <div class="bg-white p-4 rounded-lg border shadow-sm">
      <h3 class="text-sm font-medium text-gray-600">
        {{ t('components.flashcardCollectionView.stats.learning') }}
      </h3>

      <p class="text-2xl font-bold text-yellow-600">{{ stats.learning }}</p>
    </div>

    <div class="bg-white p-4 rounded-lg border shadow-sm">
      <h3 class="text-sm font-medium text-gray-600">
        {{ t('components.flashcardCollectionView.stats.review') }}
      </h3>

      <p class="text-2xl font-bold text-green-600">{{ stats.review }}</p>
    </div>

    <div class="bg-white p-4 rounded-lg border shadow-sm">
      <h3 class="text-sm font-medium text-gray-600">
        {{ t('components.flashcardCollectionView.stats.graduated') }}
      </h3>

      <p class="text-2xl font-bold text-purple-600">{{ stats.graduated }}</p>
    </div>
  </div>
  <!-- Filters -->
  <div v-if="!isAnonView" class="bg-white p-4 rounded-lg border shadow-sm">
    <div class="flex flex-wrap gap-4">
      <select v-model="filters.status" class="input-field">
        <option value="">{{ t('components.flashcardCollectionView.filters.allStatus') }}</option>

        <option value="new">{{ t('components.flashcardCollectionView.stats.new') }}</option>

        <option value="learning">
          {{ t('components.flashcardCollectionView.stats.learning') }}
        </option>

        <option value="review">{{ t('components.flashcardCollectionView.stats.review') }}</option>

        <option value="graduated">
          {{ t('components.flashcardCollectionView.stats.graduated') }}
        </option>
      </select>
      <label class="flex items-center gap-2">
        <input v-model="filters.onlyDue" type="checkbox" class="checkbox-toggle" />
        <span class="text-sm text-gray-700">{{
          t('components.flashcardCollectionView.filters.dueCardsOnly')
        }}</span>
      </label>
    </div>
  </div>
  <!-- Flashcard List -->
  <LoadingSpinner v-if="!isAnonView && isLoading" class="py-12" />
  <div v-else-if="isAnonView" class="text-center py-8 bg-gray-50 rounded-lg border border-blue-100">
    <p class="text-gray-600 mb-4">{{ t('anonymousProgress.useLevelsToStudy') }}</p>
  </div>

  <div
    v-else-if="flashcards.length === 0"
    class="text-center py-12 bg-gray-50 rounded-lg border border-blue-100"
  >
    <p class="text-gray-600">{{ t('components.flashcardCollectionView.noFlashcards') }}</p>
  </div>

  <div v-else class="space-y-4">
    <div
      v-for="(card, index) in flashcards"
      :key="card.flashcard.id"
      :class="{ 'cursor-pointer': isOwner }"
      class="surface-flashcard-summary"
      @click="isOwner && openFlashcard(card)"
    >
      <!-- Card Content -->
      <div class="flex justify-between items-start gap-4">
        <div class="min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-2 min-w-0">
            <h3 class="text-lg font-medium text-gray-800">
              {{ card.flashcard.word ?? card.flashcard.free_content_front }}
            </h3>
            <span v-if="card.flashcard.sound_url" class="shrink-0" @click.stop>
              <AudioPlayer
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
              />
            </span>
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
          <div
            v-for="progress in getOrderedProgress(card.progress)"
            :key="progress.card_side"
            class="w-32"
          >
            <div class="flex items-center justify-between mb-1">
              <span class="text-xs font-medium text-gray-600">{{ progress.card_side }}</span>
              <span class="text-xs font-medium" :class="getStatusTextClass(progress.status)">
                {{ progress.status }}
              </span>
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
            <IconButton
              :aria-label="t('components.flashcardCollectionView.reviewNowAction')"
              :button-classes="'ui-btn--empty text-orange-600 hover:bg-orange-50'"
              @click.stop="reviewSingleCard(card.flashcard.id)"
            >
              <template #icon> <Repeat1 class="h-4 w-4" /> </template>
            </IconButton>
            <IconButton
              :disabled="index === 0 || isReordering"
              :aria-label="t('components.flashcardCollectionView.moveUpAction')"
              :button-classes="`ui-btn--empty ${index === 0 || isReordering ? 'opacity-50 cursor-not-allowed' : 'hover:bg-gray-100'}`"
              @click.stop="moveCard(card, 'up')"
            >
              <template #icon> <ArrowUp class="h-4 w-4" /> </template>
            </IconButton>
            <IconButton
              :disabled="index === flashcards.length - 1 || isReordering"
              :aria-label="t('components.flashcardCollectionView.moveDownAction')"
              :button-classes="`ui-btn--empty ${index === flashcards.length - 1 || isReordering ? 'opacity-50 cursor-not-allowed' : 'hover:bg-gray-100'}`"
              @click.stop="moveCard(card, 'down')"
            >
              <template #icon> <ArrowDown class="h-4 w-4" /> </template>
            </IconButton>
          </div>
        </div>
      </div>

      <div v-if="showCanonicalForCard(card)" class="mt-2 flex w-full flex-col gap-1">
        <div
          class="flex items-center gap-2 text-xs font-semibold text-gray-400 uppercase tracking-wider"
        >
          <EqualApproximately class="h-3.5 w-3.5 text-blue-400 shrink-0" />
          <span>{{ t('components.definitionCard.canonicalLabel') }}</span>
        </div>

        <div
          class="text-sm text-gray-700 font-mono bg-blue-50/30 p-2 rounded border border-blue-100/30"
        >
          {{ card.flashcard.canonical_form }}
        </div>
      </div>

      <div class="mt-2 w-full text-sm text-gray-600">
        <LazyMathJax :content="card.flashcard.definition ?? card.flashcard.free_content_back" />
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
  GalleryHorizontalEnd,
  GalleryVerticalIcon,
  List,
  LayoutPanelTop,
  PlusCircle,
  Import,
} from 'lucide-vue-next'
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'

import {
  getFlashcards,
  getLanguages,
  updateCardPosition,
  importFromCollection,
  getCollection,
  getCollectionImage,
} from '@/api'
import AudioPlayer from '@/components/AudioPlayer.vue'
import CollectionPageHeader from '@/components/CollectionPageHeader.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import { IconButton } from '@packages/ui'
import { useAuth } from '@/composables/useAuth'
import { useSeoHead } from '@/composables/useSeoHead'
import { useI18n } from 'vue-i18n'
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
const flashcards = ref([])
const isLoading = ref(true)
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
    new: 'text-nav-link',
    learning: 'text-yellow-600',
    review: 'text-green-600',
    graduated: 'text-purple-600',
  }
  return classes[status] || 'text-gray-600'
}

const getProgressBarClass = (status) => {
  const classes = {
    new: 'bg-nav-link',
    learning: 'bg-yellow-500',
    review: 'bg-green-500',
    graduated: 'bg-purple-500',
  }
  return classes[status] || 'bg-gray-400'
}

const getOrderedProgress = (progressList = []) => {
  const sideOrder = {
    direct: 0,
    reverse: 1,
  }

  return [...progressList].sort((a, b) => {
    const left = sideOrder[a.card_side] ?? Number.MAX_SAFE_INTEGER
    const right = sideOrder[b.card_side] ?? Number.MAX_SAFE_INTEGER
    return left - right
  })
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

const collectionCoverDisplayUrl = computed(() => {
  if (!collection.value?.has_cover_image) return null
  return getCollectionImage(props.collectionId, { cached: true })
})

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

const router = useRouter()

const isReordering = ref(false)

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
.ui-btn--back,
.ui-btn--forward {
  @apply px-4 py-2 text-sm border rounded-md;
}

.ui-btn--back:disabled,
.ui-btn--forward:disabled {
  @apply opacity-50 cursor-not-allowed;
}

.ui-btn--back:not(:disabled),
.ui-btn--forward:not(:disabled) {
  @apply hover:bg-gray-50;
}

.progress-bar {
  transition: width 0.3s ease-in-out;
}
</style>
