<template>
  <div class="flex flex-col min-h-[calc(100vh-6rem)]">
    <!-- Header -->
    <div class="mb-4 flex-shrink-0">
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center space-x-3">
          <RouterLink :to="`/collections/${props.collectionId}/flashcards`" class="btn-history">
            <ArrowLeft class="h-5 w-5" />
          </RouterLink>
          <h2 class="text-2xl font-bold inline-flex items-center">
            <span class="ml-1">{{ collection?.name }} - Levels</span>
          </h2>
        </div>
      </div>

      <div class="flex flex-col sm:flex-row gap-2 items-center sm:items-start">
        <button type="button" class="btn-aqua-orange h-10 text-base" @click="startStudy">
          {{ t('flashcardLevels.studyButton', 'Study') }}
        </button>
        <IconButton v-if="isOwner" :label="t('flashcardLevels.createLevel')" button-classes="btn-aqua-emerald"
          @click="showCreateModal = true" />
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="flex justify-center py-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
    </div>

    <!-- Levels Display -->
    <div v-else class="flex flex-col flex-1 min-h-0 gap-6">
      <!-- Empty State -->
      <div v-if="levels.length === 0" class="text-center py-12 bg-gray-50 rounded-lg border flex-shrink-0">
        <p class="text-gray-600 mb-4">
          {{ t('flashcardLevels.noLevels') }}
        </p>
        <div class="flex justify-center">
          <IconButton v-if="isOwner" :label="t('flashcardLevels.createFirst')" button-classes="btn-aqua-emerald"
            @click="showCreateModal = true" />
        </div>
      </div>

      <!-- Flow Container -->
      <div v-else class="flex-1 min-h-0 flex flex-col">
        <VueFlow v-model="elements" :default-viewport="{ zoom: 1 }" :min-zoom="0.2" :max-zoom="4"
          class="vue-flow-wrapper flex-1 min-h-0">
          <template #node-custom="nodeProps">
            <div
              class="level-card node-content"
              :class="getLevelCardClass(nodeProps.data)"
              :style="getLevelCardStyle(nodeProps.data)"
            >
              <!-- Top accent + glow -->
              <div class="level-card-glow" />

              <!-- Level number pill (game-style) -->
              <div class="level-card-number">
                <span>{{ nodeProps.data.levelIndex ?? '?' }}</span>
              </div>

              <!-- Status badge (top-right) -->
              <div class="level-card-badge">
                <Lock v-if="!isLevelUnlocked(nodeProps.data) && !nodeProps.data.progress?.is_completed"
                  class="level-card-badge-icon" />
                <Flame v-else-if="isLevelUnlocked(nodeProps.data) && !nodeProps.data.progress?.is_completed"
                  class="level-card-badge-icon" />
                <Trophy v-else class="level-card-badge-icon" />
              </div>

              <!-- Title & description -->
              <div class="level-card-header">
                <h3 class="level-card-title">
                  {{ nodeProps.data.name }}
                </h3>
                <p v-if="nodeProps.data.description" class="level-card-desc">
                  {{ nodeProps.data.description }}
                </p>
              </div>

              <!-- Locked: hint + owner actions -->
              <div
                v-if="!isLevelUnlocked(nodeProps.data) && !nodeProps.data.progress?.is_completed"
                class="level-card-body"
              >
                <p v-if="nodeProps.data.prerequisites?.length" class="level-card-hint">
                  <Lock class="h-4 w-4 shrink-0 opacity-80" />
                  {{ t('flashcardLevels.unlockHint', 'Complete previous levels to unlock') }}
                </p>
                <div v-if="isOwner" class="level-card-actions">
                  <button type="button" class="level-card-btn level-card-btn-ghost" @click="editLevel(nodeProps.data)">
                    <Settings class="h-4 w-4" />
                    {{ t('flashcardLevels.actions.edit') }}
                  </button>
                  <button type="button" class="level-card-btn level-card-btn-ghost" @click="showAddCardsModal(nodeProps.data)">
                    <PlusCircle class="h-4 w-4" />
                    {{ t('flashcardLevels.actions.addCards') }}
                  </button>
                </div>
              </div>

              <!-- Unlocked / Completed: progress + actions -->
              <div v-else class="level-card-body">
                <div class="level-card-progress">
                  <div class="level-card-progress-head">
                    <Target class="h-3.5 w-3.5 opacity-80" />
                    <span>{{ t('flashcardLevels.progress') }}</span>
                    <span class="level-card-progress-count">
                      {{ nodeProps.data.progress?.cards_completed || 0 }}/{{ nodeProps.data.card_count }}
                    </span>
                  </div>
                  <div class="level-card-progress-track">
                    <div
                      class="level-card-progress-fill"
                      :style="{ width: getProgressWidth(nodeProps.data) }"
                    />
                  </div>
                  <div class="level-card-stats">
                    <span>{{ t('flashcardLevels.successRate') }}</span>
                    <span class="font-semibold">{{ formatSuccessRate(nodeProps.data.progress?.success_rate) }}</span>
                  </div>
                </div>

                <div class="level-card-actions level-card-actions-main">
                  <button
                    type="button"
                    class="level-card-btn level-card-btn-primary"
                    @click="showLevelCards(nodeProps.data)"
                  >
                    <BookOpen class="h-4 w-4" />
                    {{ t('flashcardLevels.actions.viewCards') }}
                  </button>
                  <RouterLink
                    v-if="nodeProps.data.card_count > 0"
                    :to="`/collections/${props.collectionId}/flashcards`"
                    class="level-card-btn level-card-btn-secondary"
                  >
                    <Play class="h-4 w-4" />
                    {{ t('flashcardLevels.actions.practice', 'Practice') }}
                  </RouterLink>
                  <template v-if="isOwner">
                    <button type="button" class="level-card-btn level-card-btn-ghost" @click="editLevel(nodeProps.data)">
                      <Settings class="h-4 w-4" />
                      {{ t('flashcardLevels.actions.edit') }}
                    </button>
                    <button type="button" class="level-card-btn level-card-btn-ghost" @click="showAddCardsModal(nodeProps.data)">
                      <PlusCircle class="h-4 w-4" />
                      {{ t('flashcardLevels.actions.addCards') }}
                    </button>
                  </template>
                </div>

                <p v-if="nodeProps.data.progress?.is_completed" class="level-card-completed">
                  <Sparkles class="h-3.5 w-3.5" />
                  {{ t('flashcardLevels.status.completed') }}
                </p>
              </div>
            </div>
          </template>
          <Background :pattern-color="'#aaa'" :gap="8" />
          <Controls />
        </VueFlow>
      </div>
    </div>

    <!-- Create/Edit Level ModalComponent -->
    <div v-if="showCreateModal || showEditModal"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div class="bg-white rounded-lg max-w-md w-full p-6">
        <h3 class="text-lg font-semibold mb-4">
          {{ showEditModal ? t('flashcardLevels.editLevelTitle') : t('flashcardLevels.createLevelTitle') }}
        </h3>

        <form class="space-y-4" @submit.prevent="handleSubmit">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('flashcardLevels.nameLabel') }}</label>
            <input v-model="levelForm.name" type="text" required class="w-full input-field">
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('flashcardLevels.descriptionLabel')
            }}</label>
            <textarea v-model="levelForm.description" rows="3" class="textarea-field" />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('flashcardLevels.prerequisitesLabel')
            }}</label>
            <select v-model="levelForm.prerequisite_ids" multiple
              class="w-full px-3 py-2 border rounded-md focus:ring-blue-500 focus:border-blue-500">
              <option v-for="level in availablePrerequisites" :key="level.level_id" :value="level.level_id">
                {{ level.name }}
              </option>
            </select>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('flashcardLevels.minCardsLabel')
              }}</label>
              <input v-model.number="levelForm.min_cards" type="number" min="1"
                class="w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">{{ t('flashcardLevels.minSuccessRateLabel')
              }}</label>
              <input v-model.number="levelForm.min_success_rate" type="number" min="0" max="100"
                class="w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500">
            </div>
          </div>

          <div class="flex justify-end gap-3 pt-4">
            <button v-if="showEditModal" type="button" class="btn-delete mr-auto"
              @click="showDeleteLevelConfirmDialog(currentLevel)">
              {{ t('flashcardLevels.deleteLevelButton') }}
            </button>
            <button type="button" class="btn-cancel" @click="closeModal">
              {{ t('flashcardLevels.cancelButton') }}
            </button>
            <button type="submit" :disabled="isSubmitting" class="btn-create">
              {{ isSubmitting ? t('flashcardLevels.savingButton') : showEditModal ?
                t('flashcardLevels.saveChangesButton') :
                t('flashcardLevels.createLevelButton') }}
            </button>
          </div>
        </form>
      </div>
    </div>

    <!-- Add Cards ModalComponent -->
    <ModalComponent :show="showCardsModal" :title="t('flashcardLevels.addCardsTitle')" @close="closeCardsModal">
      <div class="flex-1 overflow-y-auto">
        <div v-if="isLoadingCards" class="flex justify-center py-8">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
        </div>
        <div v-else>
          <div class="space-y-4">
            <div v-for="card in availableCards" :key="card.flashcard_id"
              class="border rounded-lg p-4 hover:border-blue-300">
              <div class="flex flex-col justify-between items-start w-full">
                <div v-if="card.progress" class="flex flex-row w-full justify-between gap-2 mb-4 text-sm text-gray-500">
                  <div class="flex flex-row text-sm text-gray-500">
                    <span>{{ t('flashcardLevels.mySuccessRate') }} {{ formatSuccessRate(card.progress.success_rate)
                    }}</span>
                    <span class="mx-2">|</span>
                    <span>{{ t('flashcardLevels.myAttempts') }} {{ card.progress.total_attempts }}</span>
                  </div>
                  <button :class="[
                    selectedCards.includes(card.flashcard_id) ? 'btn-cancel' : 'btn-insert',
                  ]" @click="toggleCardSelection(card)">
                    {{ selectedCards.includes(card.flashcard_id) ? t('flashcardLevels.selected') :
                      t('flashcardLevels.select') }}
                  </button>
                </div>
                <DefinitionCard :definition="{
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
                }" :disable-discussion-button="true" :disable-toolbar="true" :show-vote-buttons="false"
                  :notes="card.notes ?? ''" :disable-border="true" :languages="languages"
                  :collection-id="card.collection_id" :item-id="card.item_id" />
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
          <button :disabled="selectedCards.length === 0 || isAddingCards" class="btn-create" @click="addSelectedCards">
            {{ isAddingCards ? t('flashcardLevels.addingCards') : t('flashcardLevels.addNCards', {
              count:
                selectedCards.length
            }) }}
          </button>
        </div>
      </template>
    </ModalComponent>

    <!-- Level Cards ModalComponent -->
    <ModalComponent :show="showLevelCardsModal"
      :title="t('flashcardLevels.levelCardsTitle', { levelName: currentLevel?.name || '' })"
      @close="closeLevelCardsModal">
      <div class="flex-1 overflow-y-auto">
        <div v-if="isLoadingLevelCards" class="flex justify-center py-8">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
        </div>
        <div v-else>
          <div class="space-y-4">
            <div v-for="card in levelCards" :key="card.flashcard_id"
              class="border rounded-lg p-4 hover:border-blue-300">
              <div class="flex flex-col justify-between items-start w-full">
                <div v-if="card.progress" class="flex flex-row w-full justify-between gap-2 mb-4 text-sm text-gray-500">
                  <div class="flex flex-row text-sm text-gray-500">
                    <span>My Success Rate: {{ formatSuccessRate(card.progress.success_rate) }}</span>
                    <span class="mx-2">|</span>
                    <span>My Attempts: {{ card.progress.total_attempts }}</span>
                  </div>
                </div>
                <DefinitionCard :definition="{
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
                }" :disable-discussion="true" :show-vote-buttons="false" :notes="card.ci_notes" :disable-border="true"
                  :hide-history="true" :languages="languages" :collection-id="parseInt(props.collectionId)"
                  :item-id="card.item_id" :show-external-delete-button="true" :is-owner="isOwner"
                  @delete-item="confirmDeleteCard(card)" />
              </div>
            </div>
          </div>

          <!-- PaginationComponent -->
          <div v-if="levelCardsTotal > 0" class="mt-6 flex justify-between items-center">
            <button :disabled="currentLevelCardsPage === 1" class="btn-empty"
              @click="loadLevelCards(currentLevelCardsPage - 1)">
              {{ t('flashcardLevels.previousPage') }}
            </button>
            <span class="text-sm text-gray-600">
              {{ t('flashcardLevels.pageInfo', { currentPage: currentLevelCardsPage, totalPages: totalLevelCardsPages })
              }}
            </span>
            <button :disabled="currentLevelCardsPage === totalLevelCardsPages" class="btn-empty"
              @click="loadLevelCards(currentLevelCardsPage + 1)">
              {{ t('flashcardLevels.nextPage') }}
            </button>
          </div>
        </div> <!-- This closes the v-else -->
      </div> <!-- This closes the flex-1 overflow-y-auto -->
      <template #footer>
        <div class="flex justify-end">
          <button class="btn-cancel" @click="closeLevelCardsModal">
            {{ t('flashcardLevels.closeButton') }}
          </button>
        </div>
      </template>
    </ModalComponent>

    <!-- Delete Confirmation ModalComponent -->
    <div v-if="showDeleteConfirmation"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-[60]">
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
  <DeleteConfirmationModal :show="showDeleteLevelConfirm" :title="t('flashcardLevels.deleteLevelConfirmTitle')"
    :message="t('flashcardLevels.deleteLevelConfirmMessage', { levelName: levelToDelete?.name })"
    :is-deleting="isDeletingLevel" @confirm="performDeleteLevel" @cancel="showDeleteLevelConfirm = false" />
</template>

<script setup>
import { Background } from '@vue-flow/background';
import { Controls } from '@vue-flow/controls';
import { VueFlow, useVueFlow } from '@vue-flow/core';
import {
  ArrowLeft,
  Lock,
  Play,
  BookOpen,
  Settings,
  PlusCircle,
  Sparkles,
  Trophy,
  Flame,
  Target,
} from 'lucide-vue-next';
import { ref, computed, onMounted, watch, nextTick } from 'vue';
import { RouterLink, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import DeleteConfirmationModal from '@/components/DeleteConfirmation.vue';
import ModalComponent from '@/components/ModalComponent.vue';

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
import IconButton from '@/components/icons/IconButton.vue'
import { useAuth } from '@/composables/useAuth';
import { useSeoHead } from '@/composables/useSeoHead';
import '@vue-flow/core/dist/style.css';
import '@vue-flow/core/dist/theme-default.css';

const props = defineProps({
  collectionId: {
    type: [String, Number],
    required: true,
    validator: (value) => !isNaN(Number(value)),
  },
});

const auth = useAuth();
const router = useRouter();
const { t, locale } = useI18n();

const startStudy = () => {
  router.push(`/collections/${props.collectionId}/flashcards/study`);
};

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

const getLevelCardClass = (level) => {
  if (level?.progress?.is_completed) return 'level-card--completed'
  if (isLevelUnlocked(level)) return 'level-card--unlocked'
  return 'level-card--locked'
}

const getProgressWidth = (level) => {
  if (!level.card_count) return '0%'
  const progress = (level.progress?.cards_completed || 0) / level.card_count
  return `${Math.round(progress * 100)}%`
}

const formatSuccessRate = (rate) => {
  if (!rate) return '0%'
  return `${Math.round(rate * 100)}%`
}

/** Hash level name to a hue 0–360 for consistent per-level color. */
const levelNameToHue = (name) => {
  if (!name || typeof name !== 'string') return 220
  let h = 0
  for (let i = 0; i < name.length; i++) {
    h = (h << 5) - h + name.charCodeAt(i)
    h |= 0
  }
  return Math.abs(h) % 360
}

/** CSS custom properties for card theming by level name (hue) and state. */
const getLevelCardStyle = (level) => {
  const hue = levelNameToHue(level?.name)
  const completed = level?.progress?.is_completed
  const unlocked = isLevelUnlocked(level)
  const locked = !unlocked && !completed
  return {
    '--level-hue': String(hue),
    '--level-sat': locked ? '25%' : '55%',
    '--level-light': completed ? '52%' : unlocked ? '60%' : '75%',
    '--level-bg-from': locked ? `hsl(${hue}, 15%, 96%)` : `hsl(${hue}, 40%, 97%)`,
    '--level-bg-to': locked ? 'hsl(0, 0%, 100%)' : `hsl(${hue}, 25%, 99%)`,
    '--level-accent': locked ? 'hsl(0, 0%, 75%)' : `hsl(${hue}, 55%, 50%)`,
    '--level-glow': locked ? 'transparent' : `hsla(${hue}, 60%, 50%, 0.25)`,
  }
}

const getTreeLevel = (level) => {
  if (!level || !level.prerequisites || !level.prerequisites.length) return 0
  return Math.max(...level.prerequisites.map((p) => getTreeLevel(p))) + 1
}

const { elements, setElements, fitView } = useVueFlow()

// First unlocked level and next level (by position) for scroll-into-view — fit both into view
const getFirstUnlockedAndNextLevelIds = (levelsData) => {
  if (!levelsData?.length) return []
  const sorted = [...levelsData].sort((a, b) => a.position - b.position)
  const firstIndex = sorted.findIndex(
    (l) => l.progress?.is_unlocked || l.progress?.is_completed
  )
  if (firstIndex === -1) return []
  const ids = [sorted[firstIndex].level_id.toString()]
  if (firstIndex + 1 < sorted.length) {
    ids.push(sorted[firstIndex + 1].level_id.toString())
  }
  return ids
}

// Inset so level cards never touch the flow area borders (hover scale/shadow have room)
const FLOW_PADDING = 12

// Vertical spacing between level cards (enough to avoid overlap with variable card height and hover lift)
const CARD_VERTICAL_GAP = 380

// Convert levels to Vue Flow elements (vertical layout: levels stacked top-to-bottom, depth as horizontal offset)
const convertLevelsToElements = (levelsData) => {
  const sorted = [...levelsData].sort((a, b) => a.position - b.position)
  const nodes = sorted.map((level, index) => ({
    id: level.level_id.toString(),
    type: 'custom',
    position: { x: getTreeLevel(level) * 220 + FLOW_PADDING, y: index * CARD_VERTICAL_GAP + FLOW_PADDING },
    data: { ...level, levelIndex: index + 1 },
  }))

  const edges = levelsData.flatMap((level) =>
    level.prerequisites.map((prereq) => ({
      id: `e${prereq.level_id}-${level.level_id}`,
      source: prereq.level_id.toString(),
      target: level.level_id.toString(),
      animated: true,
      style: { stroke: '#94a3b8' },
    }))
  )

  return [...nodes, ...edges]
}

// Only run initial scroll once so we never get a second fitView that zooms out to fit all
let initialScrollScheduled = false

// Scroll viewport to first unlocked level and next level, zoom both into view (runs once after first load)
const scrollToFirstUnlocked = (levelsData) => {
  const nodeIds = getFirstUnlockedAndNextLevelIds(levelsData)
  if (!nodeIds.length) return // never call fitView() for "all" — that would zoom out and override our zoom-to-node
  if (initialScrollScheduled) return
  initialScrollScheduled = true
  nextTick(() => {
    setTimeout(() => {
      fitView({ nodes: nodeIds, padding: 0.1, duration: 400 })
    }, 150)
  })
}

// Watch for changes in levels and update Vue Flow elements (no deep: avoid double run and second fitView)
watch(
  () => levels.value,
  (newLevels) => {
    if (newLevels?.length) {
      setElements(convertLevelsToElements(newLevels))
      scrollToFirstUnlocked(newLevels)
    }
  }
)

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

useSeoHead({ title: t('flashcardLevels.title', { collectionName: collection.value?.name || '' }) }, locale.value)

onMounted(async () => {
  await Promise.all([fetchCollection(), fetchLevels()])
})
</script>

<style scoped>
.vue-flow-wrapper {
  width: 100%;
  height: 100%;
}

/* Internal padding so cards don't touch Vue Flow area borders */
.vue-flow-wrapper :deep(.vue-flow__viewport) {
  padding: 0.5rem; /* p-2 equivalent */
  box-sizing: border-box;
}

/* ---- Level card: game-style with hue from level name ---- */
.level-card {
  --level-hue: 220;
  --level-bg-from: hsl(220, 25%, 97%);
  --level-bg-to: hsl(220, 15%, 99%);
  --level-accent: hsl(220, 55%, 50%);
  --level-glow: hsla(220, 60%, 50%, 0.2);

  min-width: 280px;
  max-width: 320px;
  position: relative;
  overflow: hidden;
  border-radius: 1rem;
  border: 2px solid var(--level-accent);
  background: linear-gradient(165deg, var(--level-bg-from) 0%, var(--level-bg-to) 100%);
  box-shadow:
    0 4px 6px -1px rgba(0, 0, 0, 0.08),
    0 2px 4px -2px rgba(0, 0, 0, 0.05),
    0 0 0 1px rgba(0, 0, 0, 0.03);
  transition: transform 0.2s ease, box-shadow 0.2s ease, filter 0.2s ease;
}

.level-card--locked {
  filter: saturate(0.75) brightness(0.98);
  opacity: 0.92;
}

.level-card--unlocked:hover {
  transform: translateY(-3px) scale(1.01);
  box-shadow:
    0 12px 24px -8px rgba(0, 0, 0, 0.15),
    0 0 20px -4px var(--level-glow);
}

.level-card--completed {
  border-width: 2px;
  box-shadow:
    0 8px 20px -6px rgba(0, 0, 0, 0.12),
    0 0 16px -4px var(--level-glow);
}

.level-card-glow {
  position: absolute;
  inset: -20px;
  background: radial-gradient(ellipse 80% 50% at 50% -20%, var(--level-glow), transparent 60%);
  pointer-events: none;
  opacity: 0.6;
}

.level-card-number {
  position: absolute;
  top: 0.75rem;
  left: 1rem;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--level-accent);
  color: white;
  font-size: 0.8125rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
}

.level-card-badge {
  position: absolute;
  top: 0.75rem;
  right: 1rem;
  width: 2.5rem;
  height: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.75rem;
  background: var(--level-accent);
  color: white;
  box-shadow: 0 3px 8px rgba(0, 0, 0, 0.2);
}

.level-card-badge-icon {
  width: 1.25rem;
  height: 1.25rem;
}

.level-card-header {
  padding: 3rem 1rem 0.5rem 1rem;
  padding-right: 3.5rem;
}

.level-card-title {
  font-size: 1.0625rem;
  font-weight: 700;
  letter-spacing: -0.02em;
  line-height: 1.3;
  color: #1e293b;
}

.level-card--locked .level-card-title {
  color: #64748b;
}

.level-card-desc {
  margin-top: 0.25rem;
  font-size: 0.8125rem;
  line-height: 1.4;
  color: #475569;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.level-card--locked .level-card-desc {
  color: #94a3b8;
}

.level-card-body {
  padding: 0.5rem 1rem 1rem;
}

.level-card-hint {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8125rem;
  color: #64748b;
  margin-bottom: 0.75rem;
}

.level-card-progress {
  margin-bottom: 0.75rem;
}

.level-card-progress-head {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.75rem;
  font-weight: 600;
  color: #475569;
  margin-bottom: 0.35rem;
}

.level-card-progress-count {
  margin-left: auto;
  color: #334155;
}

.level-card-progress-track {
  height: 6px;
  width: 100%;
  border-radius: 9999px;
  background: #e2e8f0;
  overflow: hidden;
}

.level-card-progress-fill {
  height: 100%;
  border-radius: 9999px;
  background: var(--level-accent);
  transition: width 0.3s ease;
}

.level-card-stats {
  display: flex;
  justify-content: space-between;
  font-size: 0.6875rem;
  color: #64748b;
  margin-top: 0.25rem;
}

.level-card-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

.level-card-actions-main {
  margin-top: 0.5rem;
}

.level-card-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.8125rem;
  font-weight: 600;
  border-radius: 0.75rem;
  transition: background 0.15s, color 0.15s, box-shadow 0.15s;
  border: none;
  cursor: pointer;
  text-decoration: none;
}

.level-card-btn-primary {
  background: var(--level-accent);
  color: white;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.level-card-btn-primary:hover {
  filter: brightness(1.08);
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.2);
}

.level-card-btn-secondary {
  background: white;
  color: #334155;
  border: 2px solid #cbd5e1;
}

.level-card-btn-secondary:hover {
  border-color: var(--level-accent);
  background: color-mix(in hsl, var(--level-accent) 12%, white);
  color: #0f172a;
}

.level-card-btn-ghost {
  background: transparent;
  color: #64748b;
}

.level-card-btn-ghost:hover {
  background: #f1f5f9;
  color: #334155;
}

.level-card-completed {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  margin-top: 0.5rem;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--level-accent);
}
</style>
