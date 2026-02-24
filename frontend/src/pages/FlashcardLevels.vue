<template>
  <div>
    <!-- Header -->
    <div class="mb-6">
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

      <div class="flex flex-col sm:flex-row gap-2">
        <IconButton v-if="isOwner" :label="t('flashcardLevels.createLevel')" button-classes="btn-aqua-emerald"
          @click="showCreateModal = true" />
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="flex justify-center py-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
    </div>

    <!-- Levels Display -->
    <div v-else class="grid gap-6">
      <!-- Empty State -->
      <div v-if="levels.length === 0" class="text-center py-12 bg-gray-50 rounded-lg border">
        <p class="text-gray-600 mb-4">
          {{ t('flashcardLevels.noLevels') }}
        </p>
        <div class="flex justify-center">
          <IconButton v-if="isOwner" :label="t('flashcardLevels.createFirst')" button-classes="btn-aqua-emerald"
            @click="showCreateModal = true" />
        </div>
      </div>

      <!-- Flow Container -->
      <div v-else class="h-[600px]">
        <VueFlow v-model="elements" :default-viewport="{ zoom: 1 }" :min-zoom="0.2" :max-zoom="4"
          class="vue-flow-wrapper" fit-view-on-init>
          <template #node-custom="nodeProps">
            <div
              class="level-card node-content relative overflow-hidden rounded-2xl border-2 transition-all duration-300"
              :class="getLevelCardClass(nodeProps.data)"
            >
              <!-- Top accent bar -->
              <div class="level-card-accent absolute inset-x-0 top-0 h-1" :class="getLevelAccentClass(nodeProps.data)" />

              <!-- Status icon badge (top-right) -->
              <div
                class="absolute top-3 right-3 flex h-10 w-10 items-center justify-center rounded-xl shadow-md"
                :class="getLevelBadgeClass(nodeProps.data)"
              >
                <Lock v-if="!isLevelUnlocked(nodeProps.data) && !nodeProps.data.progress?.is_completed"
                  class="h-5 w-5 text-white" />
                <Unlock v-else-if="isLevelUnlocked(nodeProps.data) && !nodeProps.data.progress?.is_completed"
                  class="h-5 w-5 text-white" />
                <CheckCircle2 v-else class="h-5 w-5 text-white" />
              </div>

              <!-- Level number / label (optional: position in list could be passed via data if needed) -->
              <div class="pt-4 pr-14 pl-4">
                <h3
                  class="text-lg font-bold tracking-tight"
                  :class="nodeProps.data.progress?.is_completed ? 'text-emerald-800' : isLevelUnlocked(nodeProps.data) ? 'text-slate-800' : 'text-slate-500'"
                >
                  {{ nodeProps.data.name }}
                </h3>
                <p
                  v-if="nodeProps.data.description"
                  class="mt-1 text-sm line-clamp-2"
                  :class="isLevelUnlocked(nodeProps.data) || nodeProps.data.progress?.is_completed ? 'text-slate-600' : 'text-slate-400'"
                >
                  {{ nodeProps.data.description }}
                </p>
              </div>

              <!-- Locked state: hint only when level has prerequisites -->
              <div
                v-if="!isLevelUnlocked(nodeProps.data) && !nodeProps.data.progress?.is_completed"
                class="px-4 pb-4 pt-2"
              >
                <p v-if="nodeProps.data.prerequisites?.length" class="flex items-center gap-2 text-sm text-slate-400">
                  <Lock class="h-4 w-4 shrink-0" />
                  {{ t('flashcardLevels.unlockHint', 'Complete previous levels to unlock') }}
                </p>
                <div v-if="isOwner" class="mt-3 flex flex-wrap gap-2">
                  <button
                    type="button"
                    class="inline-flex items-center gap-1.5 rounded-lg border border-slate-300 bg-slate-50 px-3 py-1.5 text-sm text-slate-600 hover:bg-slate-100"
                    @click="editLevel(nodeProps.data)"
                  >
                    <Settings class="h-4 w-4" />
                    {{ t('flashcardLevels.actions.edit') }}
                  </button>
                  <button
                    type="button"
                    class="inline-flex items-center gap-1.5 rounded-lg border border-slate-300 bg-slate-50 px-3 py-1.5 text-sm text-slate-600 hover:bg-slate-100"
                    @click="showAddCardsModal(nodeProps.data)"
                  >
                    <PlusCircle class="h-4 w-4" />
                    {{ t('flashcardLevels.actions.addCards') }}
                  </button>
                </div>
              </div>

              <!-- Unlocked / Completed: progress + actions -->
              <div v-else class="px-4 pb-4 pt-2">
                <!-- Progress bar -->
                <div class="space-y-1.5">
                  <div class="flex justify-between text-xs font-medium">
                    <span :class="nodeProps.data.progress?.is_completed ? 'text-emerald-600' : 'text-slate-500'">
                      {{ t('flashcardLevels.progress') }}
                    </span>
                    <span :class="nodeProps.data.progress?.is_completed ? 'text-emerald-700' : 'text-slate-700'">
                      {{ nodeProps.data.progress?.cards_completed || 0 }}/{{ nodeProps.data.card_count }}
                    </span>
                  </div>
                  <div class="h-2 w-full overflow-hidden rounded-full bg-slate-200">
                    <div
                      class="h-full rounded-full transition-all duration-300"
                      :class="getProgressBarClass(nodeProps.data)"
                      :style="{ width: getProgressWidth(nodeProps.data) }"
                    />
                  </div>
                  <div class="flex justify-between text-xs">
                    <span class="text-slate-500">{{ t('flashcardLevels.successRate') }}</span>
                    <span class="font-medium text-slate-700">
                      {{ formatSuccessRate(nodeProps.data.progress?.success_rate) }}
                    </span>
                  </div>
                </div>

                <!-- Primary action: View cards / Practice -->
                <div class="mt-3 flex flex-wrap items-center gap-2">
                  <button
                    type="button"
                    class="inline-flex items-center gap-2 rounded-xl px-4 py-2 text-sm font-semibold shadow-sm transition focus:outline-none focus:ring-2 focus:ring-offset-2"
                    :class="nodeProps.data.progress?.is_completed
                      ? 'bg-emerald-500 text-white hover:bg-emerald-600 focus:ring-emerald-500'
                      : 'bg-blue-500 text-white hover:bg-blue-600 focus:ring-blue-500'"
                    @click="showLevelCards(nodeProps.data)"
                  >
                    <BookOpen class="h-4 w-4" />
                    {{ t('flashcardLevels.actions.viewCards') }}
                  </button>
                  <RouterLink
                    v-if="nodeProps.data.card_count > 0"
                    :to="`/collections/${props.collectionId}/flashcards`"
                    class="inline-flex items-center gap-2 rounded-xl border-2 border-slate-300 bg-white px-4 py-2 text-sm font-semibold text-slate-700 transition hover:border-blue-400 hover:bg-blue-50 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                  >
                    <Play class="h-4 w-4" />
                    {{ t('flashcardLevels.actions.practice', 'Practice') }}
                  </RouterLink>
                  <template v-if="isOwner">
                    <button
                      type="button"
                      class="inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm text-slate-600 hover:bg-slate-100 hover:text-slate-800"
                      @click="editLevel(nodeProps.data)"
                    >
                      <Settings class="h-4 w-4" />
                      {{ t('flashcardLevels.actions.edit') }}
                    </button>
                    <button
                      type="button"
                      class="inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm text-slate-600 hover:bg-slate-100 hover:text-slate-800"
                      @click="showAddCardsModal(nodeProps.data)"
                    >
                      <PlusCircle class="h-4 w-4" />
                      {{ t('flashcardLevels.actions.addCards') }}
                    </button>
                  </template>
                </div>

                <!-- Completed ribbon hint -->
                <p
                  v-if="nodeProps.data.progress?.is_completed"
                  class="mt-2 flex items-center gap-1.5 text-xs font-medium text-emerald-600"
                >
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
  Unlock,
  CheckCircle2,
  Play,
  BookOpen,
  Settings,
  PlusCircle,
  Sparkles,
} from 'lucide-vue-next';
import { ref, computed, onMounted, watch } from 'vue';
import { RouterLink } from 'vue-router';
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
const { t, locale } = useI18n();

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
  if (level.progress?.is_completed) {
    return 'level-card--completed border-emerald-400 bg-gradient-to-b from-emerald-50 to-white shadow-lg shadow-emerald-100/50'
  }
  if (isLevelUnlocked(level)) {
    return 'level-card--unlocked border-blue-400 bg-gradient-to-b from-blue-50/80 to-white shadow-md hover:shadow-lg hover:border-blue-500'
  }
  return 'level-card--locked border-slate-300 bg-gradient-to-b from-slate-50 to-white opacity-90'
}

const getLevelAccentClass = (level) => {
  if (level.progress?.is_completed) return 'bg-gradient-to-r from-emerald-400 to-emerald-500'
  if (isLevelUnlocked(level)) return 'bg-gradient-to-r from-blue-400 to-blue-500'
  return 'bg-slate-300'
}

const getLevelBadgeClass = (level) => {
  if (level.progress?.is_completed) return 'bg-emerald-500'
  if (isLevelUnlocked(level)) return 'bg-blue-500'
  return 'bg-slate-400'
}

const getProgressBarClass = (level) => {
  if (level.progress?.is_completed) return 'bg-emerald-500'
  if (isLevelUnlocked(level)) return 'bg-blue-500'
  return 'bg-slate-400'
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

const getTreeLevel = (level) => {
  if (!level || !level.prerequisites || !level.prerequisites.length) return 0
  return Math.max(...level.prerequisites.map((p) => getTreeLevel(p))) + 1
}

const { elements, setElements } = useVueFlow()

// Convert levels to Vue Flow elements (vertical layout: levels stacked top-to-bottom, depth as horizontal offset)
const convertLevelsToElements = (levelsData) => {
  const sorted = [...levelsData].sort((a, b) => a.position - b.position)
  const nodes = sorted.map((level, index) => ({
    id: level.level_id.toString(),
    type: 'custom',
    position: { x: getTreeLevel(level) * 220, y: index * 220 },
    data: level,
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

// Watch for changes in levels and update Vue Flow elements
watch(
  () => levels.value,
  (newLevels) => {
    if (newLevels) {
      setElements(convertLevelsToElements(newLevels))
    }
  },
  { deep: true }
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

.level-card {
  min-width: 280px;
  max-width: 320px;
}

.level-card--locked {
  filter: saturate(0.7);
}

.level-card--unlocked:hover {
  transform: translateY(-1px);
}

.level-card--completed .level-card-accent {
  box-shadow: 0 0 12px rgba(16, 185, 129, 0.4);
}
</style>
