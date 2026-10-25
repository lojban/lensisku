<template>
  <div class="feed-page">
    <FeedPageBody>
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
        :items-count-label="
          t('collectionDetail.itemsCount', { count: collection?.item_count ?? 0 })
        "
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
        <template v-if="isOwner" #meta-actions>
          <CollectionActionsMenu
            :collection-id="props.collectionId"
            :collection="collection"
            :is-owner="isOwner"
            @updated="onCollectionMenuUpdated"
            @refresh="loadFlashcards"
          />
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
                  v-if="(collection?.item_count ?? 0) > 0"
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
                  <template #icon>
                    <ImportIcon class="w-4 h-4 shrink-0" aria-hidden="true" />
                  </template>
                </IconButton>
              </div>
              <template v-if="!auth.state.isLoggedIn">
                <div class="mt-2 gap-4 flex flex-wrap">
                  <IconButton
                    tag="router-link"
                    :to="`/collections/${props.collectionId}/lingo/levels`"
                    :label="t('anonymousProgress.studyLevels')"
                    button-classes="ui-btn--create ui-btn--group-item md:flex-none"
                  >
                    <template #icon>
                      <Repeat1 class="w-4 h-4 shrink-0" aria-hidden="true" />
                    </template>
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
          </div>
        </template>
      </CollectionPageHeader>
      <div class="flex flex-col gap-4">
        <!-- Anonymous: sign-in prompt -->
        <div
          v-if="isAnonView"
          class="bg-amber-50 border border-amber-200 rounded-lg p-4 text-center"
        >
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
            <Select
              v-model="filters.status"
              class="input-field"
              :options="[
                { value: '', label: t('components.flashcardCollectionView.filters.allStatus') },
                { value: 'new', label: t('components.flashcardCollectionView.stats.new') },
                {
                  value: 'learning',
                  label: t('components.flashcardCollectionView.stats.learning'),
                },
                { value: 'review', label: t('components.flashcardCollectionView.stats.review') },
                {
                  value: 'graduated',
                  label: t('components.flashcardCollectionView.stats.graduated'),
                },
              ]"
            />
            <label class="flex items-center gap-2">
              <Checkbox v-model="filters.onlyDue" class="checkbox-toggle" />
              <span class="text-sm text-gray-700">{{
                t('components.flashcardCollectionView.filters.dueCardsOnly')
              }}</span>
            </label>
          </div>
        </div>
        <LevelsPanel
          :collection-id="props.collectionId"
          :is-owner="isOwner"
          @changed="loadItemLevels"
        />
        <!-- Flashcard List -->
        <div
          v-if="isAnonView"
          class="text-center py-4 bg-gray-50 rounded-lg border border-blue-100"
        >
          <p class="text-gray-600">{{ t('anonymousProgress.useLevelsToStudy') }}</p>
        </div>
        <LoadingSpinner v-if="isLoading" class="py-12" />
        <div
          v-else-if="flashcards.length === 0"
          class="text-center py-12 bg-gray-50 rounded-lg border border-blue-100"
        >
          <p class="text-gray-600">{{ t('components.flashcardCollectionView.noFlashcards') }}</p>
        </div>

        <template v-else>
          <p v-if="!isCourseMode" class="text-xs text-gray-500">
            {{ t('courseOrder.filteredNote') }}
          </p>
          <CourseOrderedList
            :collection-id="props.collectionId"
            :is-owner="isOwner"
            :items="flashcards"
            :item-key="(c) => c.flashcard.item_id"
            :flashcard-id-of="(c) => c.flashcard.id"
            :flat="!isCourseMode"
            :focus-item-key="requestedItemId"
            :focus-level-id="focusLevelId"
            @changed="loadItemLevels"
          >
            <template #item="{ item: card }">
              <div
                :data-item-id="card.flashcard.item_id"
                :class="{
                  'cursor-pointer': isOwner,
                  'flashcard-item--highlight': highlightItemId === card.flashcard.item_id,
                }"
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
                      <ItemLevelBadges
                        :levels="levelsForItem(card.flashcard.item_id)"
                        :collection-id="props.collectionId"
                      />
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

                    <div
                      v-if="card.flashcard.has_front_image || card.flashcard.has_back_image"
                      class="mt-2"
                    >
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
                        <span class="text-xs font-medium text-gray-600">{{
                          progress.card_side
                        }}</span>
                        <span
                          class="text-xs font-medium"
                          :class="getStatusTextClass(progress.status)"
                        >
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

                      <div
                        v-if="progress.next_review_at"
                        class="text-xs text-gray-500 mt-1 text-right"
                      >
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
                        v-if="!isCourseMode"
                        :disabled="flashcards.indexOf(card) === 0 || isReordering"
                        :aria-label="t('components.flashcardCollectionView.moveUpAction')"
                        :button-classes="`ui-btn--empty ${flashcards.indexOf(card) === 0 || isReordering ? 'opacity-50 cursor-not-allowed' : 'hover:bg-gray-100'}`"
                        @click.stop="moveCard(card, 'up')"
                      >
                        <template #icon> <ArrowUp class="h-4 w-4" /> </template>
                      </IconButton>
                      <IconButton
                        v-if="!isCourseMode"
                        :disabled="
                          flashcards.indexOf(card) === flashcards.length - 1 || isReordering
                        "
                        :aria-label="t('components.flashcardCollectionView.moveDownAction')"
                        :button-classes="`ui-btn--empty ${flashcards.indexOf(card) === flashcards.length - 1 || isReordering ? 'opacity-50 cursor-not-allowed' : 'hover:bg-gray-100'}`"
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
                  <LazyMathJax
                    :content="card.flashcard.definition ?? card.flashcard.free_content_back"
                  />
                </div>
              </div>
            </template>
          </CourseOrderedList>
        </template>

        <div v-if="!isCourseMode && totalPages > 1">
          <PaginationComponent
            :current-page="currentPage"
            :total-pages="totalPages"
            :total="flashcards.length"
            :per-page="perPage"
            @prev="handlePageChange(currentPage - 1)"
            @next="handlePageChange(currentPage + 1)"
          />
        </div>
      </div>
    </FeedPageBody>
    <div ref="paginationHost" class="contents" />
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
  PlusCircle,
} from '@lucide/vue'
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'

import {
  getFlashcards,
  getCollectionFlashcardsPublic,
  getLanguages,
  updateCardPosition,
  importFromCollection,
  getCollection,
  getCollectionImage,
} from '@/api'
import AudioPlayer from '@/components/AudioPlayer.vue'
import ItemLevelBadges from '@/components/ItemLevelBadges.vue'
import LevelsPanel from '@/components/levels/LevelsPanel.vue'
import CourseOrderedList from '@/components/levels/CourseOrderedList.vue'
import { useItemLevels } from '@/composables/useItemLevels'
import CollectionActionsMenu from '@/components/CollectionActionsMenu.vue'
import CollectionPageHeader from '@/components/CollectionPageHeader.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import FeedPageBody from '@/components/layout/FeedPageBody.vue'
import { useFeedPaginationHost } from '@/composables/useFeedPaginationHost'
import { Checkbox, IconButton, Select, ImportIcon } from '@packages/ui'
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
const paginationHost = useFeedPaginationHost()
const isOwner = computed(() => {
  return auth.state.isLoggedIn && collection.value?.owner?.username === auth.state.username
})

function onCollectionMenuUpdated(next) {
  collection.value = next
}

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
  document.querySelector('.feed-page__body')?.scrollTo({ top: 0, behavior: 'instant' })
}

/** Course order (levels as headings) unless a status/due filter reorders the list. */
const isCourseMode = computed(() => !filters.value.status && !filters.value.onlyDue)
const focusLevelId = computed(() => {
  const m = /^#level-(\d+)$/.exec(route.hash || '')
  return m ? Number(m[1]) : null
})
const requestedItemId = computed(() => parseInt(queryStr(route.query.item), 10) || null)

/** Course mode lists every card: 100 per request in parallel (public endpoint for guests). */
async function fetchAllFlashcards() {
  if (!auth.state.isLoggedIn) {
    const { data } = await getCollectionFlashcardsPublic(props.collectionId)
    return { flashcards: data.flashcards ?? [], total: data.flashcards?.length ?? 0, due_count: 0 }
  }
  const per = 100
  const first = await getFlashcards({ collection_id: props.collectionId, page: 1, per_page: per })
  const rest = await Promise.all(
    Array.from({ length: Math.max(0, Math.ceil(first.data.total / per) - 1) }, (_, i) =>
      getFlashcards({ collection_id: props.collectionId, page: i + 2, per_page: per })
    )
  )
  const all = [first, ...rest].flatMap((r) => r.data.flashcards)
  return { ...first.data, flashcards: all, total: all.length }
}

const loadFlashcards = async () => {
  isLoading.value = true
  try {
    const response = isCourseMode.value
      ? { data: await fetchAllFlashcards() }
      : await getFlashcards({
          collection_id: props.collectionId,
          status: filters.value.status || undefined,
          due: filters.value.onlyDue || undefined,
          page: currentPage.value,
          per_page: perPage.value,
        })

    flashcards.value = response.data.flashcards
    dueCount.value = response.data.due_count
    totalPages.value = isCourseMode.value ? 1 : Math.ceil(response.data.total / perPage.value)
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

const { load: loadItemLevels, levelsForItem } = useItemLevels(() => props.collectionId)

/** `?item=<itemId>`: jump to the page holding that item, scroll to it and highlight it. */
const highlightItemId = ref<number | null>(null)

async function findItemPage(itemId: number): Promise<number | null> {
  const scanSize = 100
  for (let p = 1; p <= 50; p++) {
    const { data } = await getFlashcards({
      collection_id: props.collectionId,
      status: filters.value.status || undefined,
      due: filters.value.onlyDue || undefined,
      page: p,
      per_page: scanSize,
    })
    const idx = data.flashcards.findIndex((f) => f.flashcard.item_id === itemId)
    if (idx >= 0) return Math.floor(((p - 1) * scanSize + idx) / perPage.value) + 1
    if (p * scanSize >= data.total) return null
  }
  return null
}

async function focusRequestedItem() {
  const itemId = parseInt(queryStr(route.query.item), 10)
  if (!itemId) return
  highlightItemId.value = itemId
  if (!flashcards.value.some((f) => f.flashcard.item_id === itemId)) {
    try {
      const page = await findItemPage(itemId)
      if (page && page !== currentPage.value) {
        currentPage.value = page
        await router.replace({ query: { ...route.query, page: page > 1 ? page : undefined } })
        await loadFlashcards()
      }
    } catch (error) {
      console.error('Error locating item:', error)
    }
  }
  await nextTick()
  document
    .querySelector(`[data-item-id="${itemId}"]`)
    ?.scrollIntoView({ behavior: 'smooth', block: 'center' })
}

onMounted(async () => {
  syncFromRoute()
  loadItemLevels()
  try {
    const response = await getCollection(String(props.collectionId))
    collection.value = response.data
    pageTitle.value = collection.value.name
  } catch (error) {
    console.error('Error fetching collection:', error)
  }

  await Promise.all([loadFlashcards(), loadLanguages()])
  // Course mode focuses `?item=` itself (CourseOrderedList); flat mode pages to it.
  if (!isCourseMode.value) await focusRequestedItem()
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
.flashcard-item--highlight {
  box-shadow:
    0 0 0 2px #3b82f6,
    0 0 0 6px rgba(59, 130, 246, 0.2);
  animation: flashcard-item-pulse 1.6s ease-out 2;
}

@keyframes flashcard-item-pulse {
  0% {
    background-color: #dbeafe;
  }
  100% {
    background-color: transparent;
  }
}

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
