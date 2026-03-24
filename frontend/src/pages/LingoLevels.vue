<template>
  <div class="lingo-levels-root flex min-h-screen flex-col bg-slate-50">
    <!-- Duolingo-style sticky header: back + title + primary CTA -->
    <header
      class="sticky top-0 z-40 flex items-center justify-between gap-4 border-b-2 border-slate-200 bg-white px-4 py-3 lg:px-6"
    >
      <RouterLink
        to="/lingo/courses"
        class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg text-slate-500 transition hover:bg-slate-100 focus:outline-none focus:ring-2 focus:ring-green-500"
        aria-label="Back"
      >
        <ArrowLeft class="h-5 w-5" />
      </RouterLink>
      <div class="flex shrink-0 items-center gap-2">
        <IconButton
          :label="t('flashcardLevels.studyButton')"
          button-classes="btn-aqua-orange h-10 px-4 text-sm font-semibold"
          icon-classes="h-4 w-4"
          @click="startStudy"
        >
          <template #icon>
            <Dumbbell class="h-4 w-4" />
          </template>
        </IconButton>
        <IconButton
          v-if="isOwner"
          :label="t('flashcardLevels.createLevel')"
          button-classes="btn-aqua-emerald h-10"
          @click="showCreateModal = true"
        />
      </div>
    </header>

    <!-- Main content: Duolingo-style feed (max-w, padded) -->
    <main class="relative flex-1 pb-10">
      <!-- Loading State -->
      <div v-if="isLoading" class="flex min-h-[40vh] items-center justify-center py-12">
        <div
          class="h-10 w-10 animate-spin rounded-full border-2 border-green-500 border-t-transparent"
        />
      </div>

      <!-- Levels Display -->
      <div v-else class="mx-auto flex max-w-[1056px] flex-col gap-6 px-4 pt-6 lg:px-6">
        <!-- Empty State -->
        <div
          v-if="levels.length === 0"
          class="rounded-2xl border-2 border-slate-200 bg-white py-12 text-center shadow-sm"
        >
          <p class="mb-4 text-slate-600">
            {{ t('flashcardLevels.noLevels') }}
          </p>
          <IconButton
            v-if="isOwner"
            :label="t('flashcardLevels.createFirst')"
            button-classes="btn-aqua-emerald"
            @click="showCreateModal = true"
          />
        </div>

        <!-- Duolingo-style ladder: unit banner + lesson buttons -->
        <div v-else class="relative flex flex-col items-center pb-16">
          <LingoUnitBanner
            :title="collection?.name || ''"
            :description="collection?.description || ''"
            :continue-label="t('flashcardLevels.continueLabel')"
            @continue="startStudy"
          />

          <div class="relative mt-4 flex flex-col items-center">
            <LingoLessonButton
              v-for="(level, i) in sortedEffectiveLevels"
              :key="level.level_id"
              :index="i"
              :total-count="sortedEffectiveLevels.length - 1"
              :locked="!isLevelUnlocked(level) && !level.progress?.is_completed"
              :current="level === activeLevel"
              :completed="!!level.progress?.is_completed"
              :percentage="
                activeLevel && level.level_id === activeLevel.level_id ? activeLevelPercentage : 0
              "
              :study-url="`/collections/${props.collectionId}/lingo/study?levelId=${level.level_id}`"
              :start-label="t('flashcardLevels.start')"
              :show-owner-menu="isOwner"
            >
              <template #menu>
                <div class="flex flex-row">
                  <button
                    type="button"
                    class="flex items-center gap-1.5 px-3 py-1.5 text-left text-sm text-slate-700 hover:bg-slate-100"
                    @click.stop="editLevel(level)"
                  >
                    <Settings class="h-4 w-4" />
                    {{ t('flashcardLevels.actions.edit') }}
                  </button>
                  <button
                    type="button"
                    class="flex items-center gap-1.5 px-3 py-1.5 text-left text-sm text-slate-700 hover:bg-slate-100"
                    @click.stop="showAddCardsModal(level)"
                  >
                    <PlusCircle class="h-4 w-4" />
                    {{ t('flashcardLevels.actions.addCards') }}
                  </button>
                  <button
                    type="button"
                    class="flex items-center gap-1.5 px-3 py-1.5 text-left text-sm text-slate-700 hover:bg-slate-100"
                    @click.stop="showLevelCards(level)"
                  >
                    <BookOpen class="h-4 w-4" />
                    {{ t('flashcardLevels.actions.viewCards') }}
                  </button>
                </div>
              </template>
            </LingoLessonButton>
          </div>
        </div>
      </div>
    </main>

    <!-- Create/Edit Level ModalComponent -->
    <div
      v-if="showCreateModal || showEditModal"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
    >
      <div class="bg-white rounded-lg max-w-md w-full p-6">
        <h3 class="text-lg font-semibold mb-4">
          {{
            showEditModal
              ? t('flashcardLevels.editLevelTitle')
              : t('flashcardLevels.createLevelTitle')
          }}
        </h3>

        <form class="space-y-4" @submit.prevent="handleSubmit">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{{
              t('flashcardLevels.nameLabel')
            }}</label>
            <input v-model="levelForm.name" type="text" required class="w-full input-field" />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{{
              t('flashcardLevels.descriptionLabel')
            }}</label>
            <textarea v-model="levelForm.description" rows="3" class="textarea-field" />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{{
              t('flashcardLevels.prerequisitesLabel')
            }}</label>
            <select
              v-model="levelForm.prerequisite_ids"
              multiple
              class="w-full px-3 py-2 border rounded-md focus:ring-blue-500 focus:border-blue-500"
            >
              <option
                v-for="level in availablePrerequisites"
                :key="level.level_id"
                :value="level.level_id"
              >
                {{ level.name }}
              </option>
            </select>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">{{
                t('flashcardLevels.minCardsLabel')
              }}</label>
              <input
                v-model.number="levelForm.min_cards"
                type="number"
                min="1"
                class="w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">{{
                t('flashcardLevels.minSuccessRateLabel')
              }}</label>
              <input
                v-model.number="levelForm.min_success_rate"
                type="number"
                min="0"
                max="100"
                class="w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
          </div>

          <div class="flex justify-end gap-3 pt-4">
            <button
              v-if="showEditModal"
              type="button"
              class="btn-delete mr-auto"
              @click="showDeleteLevelConfirmDialog(currentLevel)"
            >
              {{ t('flashcardLevels.deleteLevelButton') }}
            </button>
            <button type="button" class="btn-cancel" @click="closeModal">
              {{ t('flashcardLevels.cancelButton') }}
            </button>
            <button type="submit" :disabled="isSubmitting" class="btn-create">
              {{
                isSubmitting
                  ? t('flashcardLevels.savingButton')
                  : showEditModal
                    ? t('flashcardLevels.saveChangesButton')
                    : t('flashcardLevels.createLevelButton')
              }}
            </button>
          </div>
        </form>
      </div>
    </div>

    <!-- Add Cards ModalComponent -->
    <ModalComponent
      :show="showCardsModal"
      :title="t('flashcardLevels.addCardsTitle')"
      @close="closeCardsModal"
    >
      <div class="flex-1 overflow-y-auto">
        <div v-if="isLoadingCards" class="flex justify-center py-8">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
        </div>
        <div v-else>
          <div class="space-y-4">
            <div
              v-for="card in availableCards"
              :key="card.flashcard_id"
              class="border rounded-lg p-4 hover:border-blue-300"
            >
              <div class="flex flex-col justify-between items-start w-full">
                <div
                  v-if="card.progress"
                  class="flex flex-row w-full justify-between gap-2 mb-4 text-sm text-gray-500"
                >
                  <div class="flex flex-row text-sm text-gray-500">
                    <span
                      >{{ t('flashcardLevels.mySuccessRate') }}
                      {{ formatSuccessRate(card.progress.success_rate) }}</span
                    >
                    <span class="mx-2">|</span>
                    <span
                      >{{ t('flashcardLevels.myAttempts') }}
                      {{ card.progress.total_attempts }}</span
                    >
                  </div>
                  <button
                    :class="[
                      selectedCards.includes(card.flashcard_id) ? 'btn-cancel' : 'btn-insert',
                    ]"
                    @click="toggleCardSelection(card)"
                  >
                    {{
                      selectedCards.includes(card.flashcard_id)
                        ? t('flashcardLevels.selected')
                        : t('flashcardLevels.select')
                    }}
                  </button>
                </div>
                <DefinitionCard
                  :definition="{
                    definitionid: card.definition_id,
                    item_id: card.item_id, // Still missing, omit or handle in DefinitionCard
                    valsiid: card.valsi_id, // Still missing, omit or handle in DefinitionCard
                    valsiword: card.word ?? card.free_content_front,
                    definition: card.definition ?? card.free_content_back ?? '', // Default to empty string
                    langid: card.lang_id,
                    notes: card.notes ?? '', // Default to empty string (Definition's notes if available)
                    free_content_front: card.free_content_front,
                    free_content_back: card.free_content_back,
                    has_front_image: card.has_front_image,
                    has_back_image: card.has_back_image,
                  }"
                  :disable-discussion-button="true"
                  :disable-toolbar="true"
                  :show-vote-buttons="false"
                  :notes="card.notes ?? ''"
                  :disable-border="true"
                  :languages="languages"
                  :collection-id="card.collection_id"
                  :item-id="card.item_id"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
      <template #footer>
        <div class="flex justify-end gap-3">
          <button class="btn-cancel" @click="closeCardsModal">
            {{ t('flashcardLevels.cancelButton') }}
          </button>
          <button
            :disabled="selectedCards.length === 0 || isAddingCards"
            class="btn-create"
            @click="addSelectedCards"
          >
            {{
              isAddingCards
                ? t('flashcardLevels.addingCards')
                : t('flashcardLevels.addNCards', {
                    count: selectedCards.length,
                  })
            }}
          </button>
        </div>
      </template>
    </ModalComponent>

    <!-- Level Cards ModalComponent -->
    <ModalComponent
      :show="showLevelCardsModal"
      :title="t('flashcardLevels.levelCardsTitle', { levelName: currentLevel?.name || '' })"
      @close="closeLevelCardsModal"
    >
      <div class="flex-1 overflow-y-auto">
        <div v-if="isLoadingLevelCards" class="flex justify-center py-8">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
        </div>
        <div v-else>
          <div class="space-y-4">
            <div
              v-for="card in levelCards"
              :key="card.flashcard_id"
              class="border rounded-lg p-4 hover:border-blue-300"
            >
              <div class="flex flex-col justify-between items-start w-full">
                <div
                  v-if="card.progress"
                  class="flex flex-row w-full justify-between gap-2 mb-4 text-sm text-gray-500"
                >
                  <div class="flex flex-row text-sm text-gray-500">
                    <span
                      >My Success Rate: {{ formatSuccessRate(card.progress.success_rate) }}</span
                    >
                    <span class="mx-2">|</span>
                    <span>My Attempts: {{ card.progress.total_attempts }}</span>
                  </div>
                </div>
                <DefinitionCard
                  :definition="{
                    definitionid: card.definition_id,
                    valsiid: card.valsi_id,
                    valsiword: card.word ?? card.free_content_front,
                    definition: card.definition ?? card.free_content_back ?? '',
                    langid: card.lang_id, // Still missing, omit or handle in DefinitionCard
                    notes: card.notes ?? '',
                    free_content_front: card.free_content_front,
                    free_content_back: card.free_content_back,
                    has_front_image: card.has_front_image,
                    has_back_image: card.has_back_image,
                    item_id: card.item_id,
                  }"
                  :disable-discussion="true"
                  :show-vote-buttons="false"
                  :notes="card.ci_notes"
                  :disable-border="true"
                  :hide-history="true"
                  :languages="languages"
                  :collection-id="parseInt(props.collectionId)"
                  :item-id="card.item_id"
                  :show-external-delete-button="true"
                  :is-owner="isOwner"
                  @delete-item="confirmDeleteCard(card)"
                />
              </div>
            </div>
          </div>

          <!-- PaginationComponent -->
          <div v-if="levelCardsTotal > 0" class="mt-6 flex justify-between items-center">
            <button
              :disabled="currentLevelCardsPage === 1"
              class="btn-empty"
              @click="loadLevelCards(currentLevelCardsPage - 1)"
            >
              {{ t('flashcardLevels.previousPage') }}
            </button>
            <span class="text-sm text-gray-600">
              {{
                t('flashcardLevels.pageInfo', {
                  currentPage: currentLevelCardsPage,
                  totalPages: totalLevelCardsPages,
                })
              }}
            </span>
            <button
              :disabled="currentLevelCardsPage === totalLevelCardsPages"
              class="btn-empty"
              @click="loadLevelCards(currentLevelCardsPage + 1)"
            >
              {{ t('flashcardLevels.nextPage') }}
            </button>
          </div>
        </div>
        <!-- This closes the v-else -->
      </div>
      <!-- This closes the flex-1 overflow-y-auto -->
      <template #footer>
        <div class="flex justify-end">
          <button class="btn-cancel" @click="closeLevelCardsModal">
            {{ t('flashcardLevels.closeButton') }}
          </button>
        </div>
      </template>
    </ModalComponent>

    <!-- Delete Confirmation ModalComponent -->
    <div
      v-if="showDeleteConfirmation"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-[60]"
    >
      <div class="bg-white rounded-lg max-w-md w-full p-6">
        <h3 class="text-lg font-semibold mb-4">
          {{ t('flashcardLevels.deleteCardTitle') }}
        </h3>
        <p class="text-gray-600 mb-6">
          {{ t('flashcardLevels.deleteCardMessage') }}
        </p>
        <div class="flex justify-end gap-3">
          <button class="btn-cancel" @click="showDeleteConfirmation = false">
            {{ t('flashcardLevels.cancelButton') }}
          </button>
          <button class="btn-delete" @click="deleteCard">
            {{ t('flashcardLevels.deleteButton') }}
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- Delete Level Confirmation Modal -->
  <DeleteConfirmationModal
    :show="showDeleteLevelConfirm"
    :title="t('flashcardLevels.deleteLevelConfirmTitle')"
    :message="t('flashcardLevels.deleteLevelConfirmMessage', { levelName: levelToDelete?.name })"
    :is-deleting="isDeletingLevel"
    @confirm="performDeleteLevel"
    @cancel="showDeleteLevelConfirm = false"
  />
</template>

<script setup>
import { ArrowLeft, BookOpen, Settings, PlusCircle, Dumbbell } from 'lucide-vue-next'
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import DeleteConfirmationModal from '@/components/DeleteConfirmation.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import LingoUnitBanner from '@/components/LingoUnitBanner.vue'
import LingoLessonButton from '@/components/LingoLessonButton.vue'

import {
  getCollection,
  getFlashcards,
  createLevel,
  updateLevel,
  addCardsToLevel,
  getLevels,
  getLevelCards,
  removeCardFromLevel,
  deleteLevel, // Import the new function
} from '@/api'
import DefinitionCard from '@/components/DefinitionCard.vue'
import { IconButton } from '@packages/ui'
import { useAuth } from '@/composables/useAuth'
import { useAnonymousProgress } from '@/composables/useAnonymousProgress'
import { useSeoHead } from '@/composables/useSeoHead'

const props = defineProps({
  collectionId: {
    type: [String, Number],
    required: true,
    validator: (value) => !isNaN(Number(value)),
  },
})

const auth = useAuth()
const router = useRouter()
const { t, locale } = useI18n()
const { getProgress } = useAnonymousProgress()

function startStudy() {
  const sorted = [...levels.value].sort((a, b) => a.position - b.position)
  let targetLevelId = null

  if (auth.state.isLoggedIn) {
    const firstUnlockedIncomplete = sorted.find(
      (level) =>
        (!level.is_locked || !!level.progress?.is_unlocked) && !level.progress?.is_completed
    )
    targetLevelId = firstUnlockedIncomplete?.level_id ?? sorted[0]?.level_id
  } else {
    const firstUnlocked = sorted.find((l) => isLevelUnlockedForAnon(l))
    targetLevelId = firstUnlocked?.level_id ?? sorted[0]?.level_id
  }

  if (targetLevelId) {
    router.push(`/collections/${props.collectionId}/lingo/study?levelId=${targetLevelId}`)
  } else {
    router.push(`/collections/${props.collectionId}/lingo/study`)
  }
}

// State
const collection = ref(null)
const levels = ref([])
const isLoading = ref(true)
const isSubmitting = ref(false)
const showCreateModal = ref(false)
const showEditModal = ref(false)
const showCardsModal = ref(false)
const showLevelCardsModal = ref(false)
const showDeleteConfirmation = ref(false)
const currentLevel = ref(null)
const availableCards = ref([])
const levelCards = ref([])
const selectedCards = ref([])
const isLoadingCards = ref(false)
const languages = ref([])
const isAddingCards = ref(false)
const isLoadingLevelCards = ref(false)
const currentLevelCardsPage = ref(1)
const levelCardsTotal = ref(0)
const cardsPerPage = 10
const cardToDelete = ref(null)
const showDeleteLevelConfirm = ref(false)
const levelToDelete = ref(null)
const isDeletingLevel = ref(false)

const levelForm = ref({
  name: '',
  description: '',
  min_cards: 5,
  min_success_rate: 80,
  prerequisite_ids: [],
})

// Computed
const isOwner = computed(() => {
  return auth.state.isLoggedIn && collection.value?.owner?.username === auth.state.username
})

const sortedLevels = computed(() => {
  return [...levels.value].sort((a, b) => a.position - b.position)
})

/** Levels in ladder order (effectiveLevels sorted by position) */
const sortedEffectiveLevels = computed(() => {
  return [...effectiveLevels.value].sort((a, b) => a.position - b.position)
})

/** First unlocked, incomplete level (current “lesson” in Duolingo terms) */
const activeLevel = computed(() => {
  return (
    sortedEffectiveLevels.value.find((l) => isLevelUnlocked(l) && !l.progress?.is_completed) ?? null
  )
})

/** Progress 0–100 for the active level (for circular progress on current node) */
const activeLevelPercentage = computed(() => {
  const level = activeLevel.value
  if (!level?.card_count) return 0
  const done = level.progress?.cards_completed ?? 0
  return Math.round((done / level.card_count) * 100)
})

// When anonymous, a level is unlocked if it has no prerequisites or all prerequisites are completed in localStorage
function isLevelUnlockedForAnon(level) {
  if (!level?.prerequisites?.length) return true
  const cid = props.collectionId
  return level.prerequisites.every((p) => {
    const local = getProgress(cid, p.level_id)
    return local?.completed_at != null
  })
}

// Levels with anonymous progress merged in (is_locked, progress) when not logged in
const effectiveLevels = computed(() => {
  const raw = levels.value
  if (auth.state.isLoggedIn || !raw.length) return raw
  const cid = props.collectionId
  return raw.map((level) => {
    const local = getProgress(cid, level.level_id)
    const hasLocal = local && (local.cards_completed > 0 || local.total_answers > 0)
    const isLocked = hasLocal ? !isLevelUnlockedForAnon(level) : level.is_locked
    const progress = hasLocal
      ? {
          cards_completed: local.cards_completed ?? 0,
          correct_answers: local.correct_answers ?? 0,
          total_answers: local.total_answers ?? 0,
          is_unlocked: !isLocked,
          is_completed: !!local.completed_at,
        }
      : level.progress
    return { ...level, is_locked: isLocked, progress }
  })
})

const availablePrerequisites = computed(() => {
  if (!currentLevel.value) {
    return levels.value
  }
  return levels.value.filter((level) => level.level_id !== currentLevel.value.level_id)
})

// Methods
const fetchCollection = async () => {
  try {
    const response = await getCollection(props.collectionId)
    collection.value = response.data
  } catch (error) {
    console.error('Error fetching collection:', error)
  }
}

const totalLevelCardsPages = computed(() => {
  return Math.ceil(levelCardsTotal.value / cardsPerPage)
})

const fetchLevels = async () => {
  try {
    const response = await getLevels(props.collectionId)
    levels.value = response.data.levels
  } catch (error) {
    console.error('Error fetching levels:', error)
  } finally {
    isLoading.value = false
  }
}

const handleSubmit = async () => {
  if (isSubmitting.value) return
  isSubmitting.value = true

  try {
    const formData = {
      ...levelForm.value,
      min_success_rate: levelForm.value.min_success_rate / 100,
    }

    if (showEditModal.value) {
      await updateLevel(currentLevel.value.level_id, formData)
    } else {
      await createLevel(props.collectionId, formData)
    }

    await fetchLevels()
    closeModal()
  } catch (error) {
    console.error('Error saving level:', error)
  } finally {
    isSubmitting.value = false
  }
}

const editLevel = (level) => {
  currentLevel.value = level
  levelForm.value = {
    name: level.name,
    description: level.description || '',
    min_cards: level.min_cards,
    min_success_rate: Math.round(level.min_success_rate * 100),
    prerequisite_ids: level.prerequisites.map((p) => p.level_id),
  }
  showEditModal.value = true
}

const closeModal = () => {
  showCreateModal.value = false
  showEditModal.value = false
  currentLevel.value = null
  levelForm.value = {
    name: '',
    description: '',
    min_cards: 5,
    min_success_rate: 80,
    prerequisite_ids: [],
  }
}

const showAddCardsModal = async (level) => {
  currentLevel.value = level
  showCardsModal.value = true
  isLoadingCards.value = true
  selectedCards.value = []

  try {
    const response = await getFlashcards({
      collection_id: props.collectionId,
    })
    availableCards.value = response.data.flashcards.map((f) => ({
      flashcard_id: f.flashcard.id,
      word: f.flashcard.word,
      definition: f.flashcard.definition,
      definition_id: f.flashcard.definition_id,
      lang_id: f.flashcard.definition_language_id,
      notes: f.flashcard.notes,
      free_content_front: f.flashcard.free_content_front,
      free_content_back: f.flashcard.free_content_back,
      has_front_image: f.flashcard.has_front_image,
      has_back_image: f.flashcard.has_back_image,
      progress: f.progress,
      collection_id: f.flashcard.collection_id,
      item_id: f.flashcard.item_id,
    }))
  } catch (error) {
    console.error('Error loading flashcards:', error)
  } finally {
    isLoadingCards.value = false
  }
}

const closeCardsModal = () => {
  showCardsModal.value = false
  currentLevel.value = null
  availableCards.value = []
  selectedCards.value = []
}

const toggleCardSelection = (card) => {
  const index = selectedCards.value.indexOf(card.flashcard_id)
  if (index === -1) {
    selectedCards.value.push(card.flashcard_id)
  } else {
    selectedCards.value.splice(index, 1)
  }
}

const showLevelCards = async (level) => {
  currentLevel.value = level
  showLevelCardsModal.value = true
  await loadLevelCards(1)
}

const closeLevelCardsModal = () => {
  showLevelCardsModal.value = false
  currentLevel.value = null
  levelCards.value = []
  levelCardsTotal.value = 0
  currentLevelCardsPage.value = 1
}

const loadLevelCards = async (page) => {
  if (!currentLevel.value) return
  isLoadingLevelCards.value = true

  try {
    const response = await getLevelCards(currentLevel.value.level_id, page, cardsPerPage)
    levelCards.value = response.data.cards
    levelCardsTotal.value = response.data.total
    currentLevelCardsPage.value = page
  } catch (error) {
    console.error('Error loading level cards:', error)
  } finally {
    isLoadingLevelCards.value = false
  }
}

const confirmDeleteCard = (card) => {
  cardToDelete.value = card
  showDeleteConfirmation.value = true
}

const deleteCard = async () => {
  if (!cardToDelete.value || !currentLevel.value) return

  try {
    await removeCardFromLevel(currentLevel.value.level_id, cardToDelete.value.flashcard_id)
    await loadLevelCards(currentLevelCardsPage.value)
    showDeleteConfirmation.value = false
    cardToDelete.value = null
  } catch (error) {
    console.error('Error deleting card:', error)
  }
}

const addSelectedCards = async () => {
  if (isAddingCards.value || !currentLevel.value) return
  isAddingCards.value = true

  try {
    await addCardsToLevel(currentLevel.value.level_id, {
      flashcard_ids: selectedCards.value,
    })
    await fetchLevels()
    closeCardsModal()
  } catch (error) {
    console.error('Error adding cards:', error)
  } finally {
    isAddingCards.value = false
  }
}

// Level is playable when backend says not locked (e.g. no prerequisites) or user progress has unlocked_at set
const isLevelUnlocked = (level) => level && (!level.is_locked || !!level.progress?.is_unlocked)

const formatSuccessRate = (rate) => {
  if (!rate) return '0%'
  return `${Math.round(rate * 100)}%`
}

const showDeleteLevelConfirmDialog = (level) => {
  levelToDelete.value = level
  showDeleteLevelConfirm.value = true
}

const performDeleteLevel = async () => {
  if (!levelToDelete.value) return
  isDeletingLevel.value = true

  try {
    await deleteLevel(levelToDelete.value.level_id) // Use the new API function
    await fetchLevels() // Refresh the list
    showDeleteLevelConfirm.value = false
    levelToDelete.value = null
  } catch (error) {
    console.error('Error deleting level:', error)
    // Optionally show an error message to the user
  } finally {
    isDeletingLevel.value = false
  }
}

useSeoHead(
  { title: t('flashcardLevels.title', { collectionName: collection.value?.name || '' }) },
  locale.value
)

onMounted(async () => {
  await Promise.all([fetchCollection(), fetchLevels()])
})
</script>

<style scoped>
/* Ladder layout uses LingoUnitBanner + LingoLessonButton */
</style>
