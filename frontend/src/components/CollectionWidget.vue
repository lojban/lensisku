<template>
  <div class="collection-widget">
    <!-- Add to Collection Button -->
    <button class="btn-empty flex items-center gap-2 hover:text-yellow-600" @click="openModal"
      :title="t('collectionWidget.addToCollection')">
      <CopyPlus class="w-4 h-4" />
    </button>

    <ModalComponent :show="showModal" :title="t('collectionWidget.modalTitle')" @close="closeModal">
      <!-- Header -->
      <template #header>
        <h3 class="text-xl font-bold">
          {{ t('collectionWidget.modalTitle') }}
        </h3>
      </template>

      <!-- Loading State -->
      <LoadingSpinner v-if="isLoading" class="py-4" />

      <!-- Collections List -->
      <div v-else>
        <!-- Create New Collection -->
        <IconButton v-if="collections.length > 0" button-classes="w-full btn-aqua-emerald mb-4"
          :label="t('collectionWidget.createNew')" @click="showCreateForm = true" />

        <!-- Empty State -->
        <div v-if="collections.length === 0" class="px-3 py-4 text-center">
          <p class="text-sm text-gray-500 mb-2">
            {{ t('collectionWidget.noCollections') }}
          </p>
          <IconButton button-classes="btn-aqua-emerald mt-4 mx-auto" :label="t('collectionWidget.createFirst')"
            @click="showCreateForm = true" />
        </div>

        <!-- Collections -->
        <div v-else class="max-h-64 overflow-y-auto space-y-1">
          <button v-for="collection in collections" :key="collection.collection_id"
            :disabled="isAddingTo === collection.collection_id"
            class="w-full px-3 py-2 text-left text-sm rounded-md flex items-center justify-between group transition-colors"
            :class="{
              'bg-indigo-100 hover:bg-indigo-200':
                selectedCollectionId === collection.collection_id,
              'hover:bg-gray-100': selectedCollectionId !== collection.collection_id,
            }" @click="addToCollection(collection.collection_id)">
            <div class="flex items-center gap-2">
              <span class="text-gray-700">{{ collection.name }}</span>
              <span class="text-xs text-gray-500">{{ t('collectionWidget.itemsCount', { count: collection.item_count })
                }}</span>
            </div>
            <span v-if="isAddingTo === collection.collection_id" class="text-indigo-600 animate-spin text-sm">↻</span>
            <span v-else class="text-gray-400 invisible group-hover:visible">{{ selectedCollectionId ===
              collection.collection_id ? t('collectionWidget.selected') : t('collectionWidget.select') }}</span>
          </button>
        </div>
      </div>

      <!-- Create Collection Form -->
      <div v-if="showCreateForm" class="border-t mt-2 pt-2">
        <form class="space-y-3" @submit.prevent="createAndAddToCollection">
          <div>
            <label class="block text-xs font-medium text-gray-700 mb-1">{{ t('collectionWidget.collectionNameLabel')
              }}</label>
            <input v-model="newCollection.name" type="text" required class="w-full input-field">
          </div>
          <div>
            <label class="block text-xs font-medium text-gray-700 mb-1">{{ t('collectionWidget.descriptionLabel')
              }}</label>
            <textarea v-model="newCollection.description" rows="2" class="textarea-field" />
          </div>
          <div class="flex items-center space-x-2">
            <input id="is_public" v-model="newCollection.is_public" type="checkbox" class="checkbox-toggle">
            <label for="is_public" class="text-xs text-gray-700">
              {{ t('collectionWidget.makePublic') }}
            </label>
          </div>
          <div class="mt-2 flex flex-col gap-2">
            <!-- Progress bar when creating collection -->
            <div v-if="isCreating" class="w-full h-1.5 bg-gray-200 rounded-full overflow-hidden">
              <div class="h-full w-1/3 bg-indigo-500 rounded-full progress-indeterminate" />
            </div>
            <div class="flex justify-end gap-2">
              <button type="button" class="btn-cancel" :disabled="isCreating" @click="showCreateForm = false">
                {{ t('collectionWidget.cancel') }}
              </button>
              <button type="submit" :disabled="isCreating" class="btn-create">
                {{ isCreating ? t('collectionDetail.saving') : t('collectionWidget.createAndAdd') }}
              </button>
            </div>
          </div>
        </form>
      </div>

      <!-- Notes Input -->
      <div v-if="showNotesInput" class="border-t mt-2 pt-2">
        <label class="block text-xs font-medium text-gray-700 mb-1">{{ t('collectionWidget.notesLabel') }}</label>
        <textarea v-model="notes" rows="2" :placeholder="t('collectionWidget.notesPlaceholder')"
          class="textarea-field" />
        <div class="mt-2 flex flex-col gap-2">
          <!-- Progress bar when saving -->
          <div v-if="isAddingTo === selectedCollectionId" class="w-full h-1.5 bg-gray-200 rounded-full overflow-hidden">
            <div class="h-full w-1/3 bg-indigo-500 rounded-full progress-indeterminate" />
          </div>
          <div class="flex justify-end gap-2">
            <button class="btn-cancel" :disabled="isAddingTo === selectedCollectionId" @click="cancelAddWithNotes">
              {{ t('collectionWidget.cancel') }}
            </button>
            <button class="btn-insert" :disabled="isAddingTo === selectedCollectionId" @click="confirmAddWithNotes">
              {{ t('collectionWidget.addToCollectionButton') }}
            </button>
          </div>
        </div>
      </div>
    </ModalComponent>

    <!-- Success ToastFloat -->
    <ToastFloat :show="showSuccessToast" :message="t('collectionWidget.addedSuccess')" type="success" />
  </div>
</template>

<script setup>
import { ref, watch, onMounted, onUnmounted, computed } from 'vue';
import { useI18n } from 'vue-i18n';

import { getCollections, addCollectionItem, api } from '@/api';
import IconButton from '@/components/icons/IconButton.vue';
import LoadingSpinner from '@/components/LoadingSpinner.vue';
import ModalComponent from '@/components/ModalComponent.vue';
import ToastFloat from '@/components/ToastFloat.vue';
import { CopyPlus } from 'lucide-vue-next';

const { t } = useI18n();

const props = defineProps({
  definitionId: {
    type: Number,
    required: true,
  },
  word: {
    type: String,
    required: true,
  },
  externalCollections: {
    type: Array,
    default: () => [],
  },
})

const collections = ref([])
const isLoading = ref(false)
const showModal = ref(false)
const showCreateForm = ref(false)
const showNotesInput = ref(false)
const isCreating = ref(false)
const isAddingTo = ref(null)
const selectedCollectionId = ref(null)
const notes = ref('')
const showSuccessToast = ref(false)

const newCollection = ref({
  name: '',
  description: '',
  is_public: true,
})

const emit = defineEmits(['collection-updated'])

const fetchCollections = async () => {
  try {
    const collectionsResponse = await getCollections()
    collections.value = collectionsResponse.data.collections
    emit('collection-updated', collections.value)
  } catch (error) {
    console.error('Error fetching collections:', error)
  }
}

const openModal = async () => {
  showModal.value = true
  if (collections.value.length === 0) {
    isLoading.value = true
  }
  await fetchCollections()
  isLoading.value = false
}

const closeModal = () => {
  showModal.value = false
  showCreateForm.value = false
  showNotesInput.value = false
  notes.value = ''
  selectedCollectionId.value = null
}

const createAndAddToCollection = async () => {
  if (isCreating.value) return
  isCreating.value = true

  try {
    // Send the correctly formatted request data
    const response = await api.post('/collections', {
      name: newCollection.value.name,
      description: newCollection.value.description || undefined,
      is_public: newCollection.value.is_public,
    })

    const collectionId = response.data.collection_id

    await addCollectionItem(collectionId, {
      definition_id: props.definitionId,
      notes: notes.value,
    })

    // Reset form
    newCollection.value = { name: '', description: '', is_public: true }
    showCreateForm.value = false

    // Refresh collections list
    const collectionsResponse = await getCollections()
    collections.value = collectionsResponse.data.collections
    emit('collection-updated', collections.value)
  } catch (error) {
    console.error('Error creating collection:', error)
  } finally {
    isCreating.value = false
  }
}

// Handle adding to collection
const addToCollection = async (collectionId) => {
  selectedCollectionId.value = collectionId
  showNotesInput.value = true
}

// Confirm adding with notes
const confirmAddWithNotes = async () => {
  if (!selectedCollectionId.value) return

  isAddingTo.value = selectedCollectionId.value

  try {
    await addCollectionItem(selectedCollectionId.value, {
      definition_id: props.definitionId,
      notes: notes.value,
      auto_progress: true,
    })

    // Update the collection count locally
    const updatedCollection = collections.value.find(
      (c) => c.collection_id === selectedCollectionId.value
    )
    if (updatedCollection) {
      updatedCollection.item_count++
      // Emit the updated collections array
      const collectionsResponse = await getCollections()
      collections.value = collectionsResponse.data.collections
      emit('collection-updated', collections.value)
    }

    showSuccessToast.value = true
    setTimeout(() => {
      showSuccessToast.value = false
    }, 3000)

    // Reset state
    showNotesInput.value = false
    notes.value = ''

    // Refresh collections to update counts
    await fetchCollections()
  } catch (error) {
    console.error('Error adding to collection:', error)
  } finally {
    isAddingTo.value = null
    selectedCollectionId.value = null
  }
}

// Cancel adding with notes
const cancelAddWithNotes = () => {
  showNotesInput.value = false
  notes.value = ''
  selectedCollectionId.value = null
}

// Close dropdown when clicking outside
const handleClickOutside = (event) => {
  if (!event.target.closest('.collection-widget')) {
    showCreateForm.value = false
    showNotesInput.value = false
  }
}

// Lifecycle hooks
watch(
  () => props.externalCollections,
  (newCollections) => {
    if (newCollections && newCollections.length > 0) {
      collections.value = newCollections
    }
  },
  { deep: true }
)

onMounted(() => {
  const newCollections = props.externalCollections
  if (newCollections && newCollections.length > 0) {
    collections.value = newCollections
  }

  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
/* Styles remain exactly the same */
</style>
