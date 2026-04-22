<template>
  <ModalComponent :show="showModal" :title="t('addAllToCollection.modalTitle')" @close="closeModal">
    <template #header>
      <h3 class="text-xl font-bold">{{ t('addAllToCollection.modalTitle') }}</h3>
    </template>

    <AlertComponent type="warning" class="mb-3">
      {{ t('addAllToCollection.warningMessage') }}
    </AlertComponent>

    <LoadingSpinner v-if="isLoading" variant="inline" class="py-4" />

    <div v-else>
      <div v-if="collections.length > 0" class="flex justify-center mb-4">
        <IconButton
          button-classes="ui-btn--create"
          :label="t('collectionWidget.createNew')"
          @click="showCreateForm = true"
        />
      </div>

      <div v-if="collections.length === 0" class="px-3 py-4 text-center">
        <p class="text-sm text-gray-500 mb-2">{{ t('collectionWidget.noCollections') }}</p>
        <IconButton
          button-classes="ui-btn--create mt-4 mx-auto"
          :label="t('collectionWidget.createFirst')"
          @click="showCreateForm = true"
        />
      </div>

      <div v-else class="max-h-64 overflow-y-auto space-y-1">
        <button
          v-for="collection in collections"
          :key="collection.collection_id"
          :disabled="isSubmitting"
          class="w-full px-3 py-2 text-left text-sm rounded-md flex items-center justify-between group transition-colors"
          :class="{
            'bg-indigo-100 hover:bg-indigo-200':
              selectedCollectionId === collection.collection_id,
            'hover:bg-gray-100': selectedCollectionId !== collection.collection_id,
          }"
          @click="selectCollection(collection.collection_id)"
        >
          <div class="min-w-0 flex-1 flex flex-wrap items-baseline gap-x-2 gap-y-0.5 text-left">
            <span class="text-sm text-gray-700 break-words">{{ collection.name }}</span>
            <span class="shrink-0 italic text-xs text-gray-500 whitespace-nowrap">
              {{ t('collectionWidget.itemsCount', { count: collection.item_count }) }}
            </span>
          </div>
          <span class="text-gray-400 invisible group-hover:visible">
            {{
              selectedCollectionId === collection.collection_id
                ? t('collectionWidget.selected')
                : t('collectionWidget.select')
            }}
          </span>
        </button>
      </div>
    </div>

    <div v-if="showCreateForm" class="border-t mt-2 pt-2">
      <form class="space-y-3" @submit.prevent="createCollectionAndSelect">
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">
            {{ t('collectionWidget.collectionNameLabel') }}
          </label>
          <input v-model="newCollection.name" type="text" required class="w-full input-field" />
        </div>

        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">
            {{ t('collectionWidget.descriptionLabel') }}
          </label>
          <textarea
            v-model="newCollection.description"
            rows="2"
            class="textarea-field"
          />
        </div>

        <div class="flex items-center space-x-2">
          <input
            id="addall_is_public"
            v-model="newCollection.is_public"
            type="checkbox"
            class="checkbox-toggle"
          />
          <label for="addall_is_public" class="text-xs text-gray-700">
            {{ t('collectionWidget.makePublic') }}
          </label>
        </div>

        <div class="mt-2 flex flex-col gap-2">
          <div
            v-if="isCreating"
            class="w-full h-1.5 bg-gray-200 rounded-full overflow-hidden"
          >
            <div class="h-full w-1/3 bg-indigo-500 rounded-full progress-indeterminate" />
          </div>

          <div class="flex justify-end gap-2">
            <button
              type="button"
              class="ui-btn--cancel"
              :disabled="isCreating"
              @click="showCreateForm = false"
            >
              {{ t('collectionWidget.cancel') }}
            </button>
            <button type="submit" :disabled="isCreating" class="ui-btn--create">
              {{
                isCreating ? t('collectionDetail.saving') : t('addAllToCollection.createAndSelect')
              }}
            </button>
          </div>
        </div>
      </form>
    </div>

    <div v-if="showConfirm" class="border-t mt-2 pt-2">
      <label class="block text-xs font-medium text-gray-700 mb-1">
        {{ t('collectionWidget.notesLabel') }}
      </label>
      <textarea
        v-model="notes"
        rows="2"
        :placeholder="t('collectionWidget.notesPlaceholder')"
        class="textarea-field"
      />

      <div class="mt-2 flex flex-col gap-2">
        <p v-if="progressText" class="text-xs text-gray-500">{{ progressText }}</p>
        <div
          v-if="isSubmitting"
          class="w-full h-1.5 bg-gray-200 rounded-full overflow-hidden"
        >
          <div class="h-full w-1/3 bg-indigo-500 rounded-full progress-indeterminate" />
        </div>

        <div class="flex justify-end gap-2">
          <button
            class="ui-btn--cancel"
            :disabled="isSubmitting"
            @click="cancelConfirm"
          >
            {{ t('collectionWidget.cancel') }}
          </button>
          <button
            class="ui-btn--insert"
            :disabled="isSubmitting"
            @click="confirmAddAll"
          >
            {{ t('addAllToCollection.confirmButton') }}
          </button>
        </div>
      </div>
    </div>
  </ModalComponent>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import {
  api,
  bulkAddDefinitionsToCollection,
  getCollections,
} from '@/api'
import AlertComponent from '@/components/AlertComponent.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import { IconButton } from '@packages/ui'
import { useSuccessToast } from '@/composables/useSuccessToast'

const { t } = useI18n()
const { showSuccess } = useSuccessToast()

interface CollectionRow {
  collection_id: number
  name: string
  item_count: number
}

/**
 * Caller provides a loader that returns every definition ID matching the current
 * dictionary search (across all pages). We intentionally fetch IDs lazily on confirm
 * so closing the modal costs nothing if the user abandons.
 */
type LoadAllProgress = (current: number, expectedTotal: number) => void

const props = defineProps<{
  /** When true, parent wants the modal open. Two-way via `update:modelValue`. */
  modelValue: boolean
  externalCollections?: CollectionRow[]
  /**
   * Caller-supplied loader that walks **every page** of the current search and
   * returns all matching definition ids. We pass a progress callback so the modal
   * can display "Collected X / Y" while the loader is paginating.
   */
  loadAllDefinitionIds: (onProgress?: LoadAllProgress) => Promise<number[]>
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', v: boolean): void
  (e: 'collection-updated', collections: CollectionRow[]): void
  (e: 'added', info: { added: number; skipped: number; invalid: number }): void
}>()

const collections = ref<CollectionRow[]>([])
const isLoading = ref(false)
const isSubmitting = ref(false)
const isCreating = ref(false)
const showCreateForm = ref(false)
const showConfirm = ref(false)
const selectedCollectionId = ref<number | null>(null)
const notes = ref('')
const progressText = ref('')

const newCollection = ref({
  name: '',
  description: '',
  is_public: true,
})

const showModal = ref(false)

function resetTransientState() {
  showCreateForm.value = false
  showConfirm.value = false
  selectedCollectionId.value = null
  notes.value = ''
  progressText.value = ''
  newCollection.value = { name: '', description: '', is_public: true }
}

watch(
  () => props.modelValue,
  async (open) => {
    showModal.value = open
    if (open) {
      if (props.externalCollections && props.externalCollections.length > 0) {
        collections.value = [...props.externalCollections]
      } else {
        isLoading.value = true
        await fetchCollections()
        isLoading.value = false
      }
    } else {
      resetTransientState()
    }
  },
  { immediate: true }
)

const fetchCollections = async () => {
  try {
    const res = await getCollections()
    collections.value = res.data.collections
    emit('collection-updated', collections.value)
  } catch (e) {
    console.error('Error fetching collections:', e)
  }
}

const closeModal = () => {
  emit('update:modelValue', false)
}

const selectCollection = (collectionId: number) => {
  selectedCollectionId.value = collectionId
  showConfirm.value = true
}

const cancelConfirm = () => {
  showConfirm.value = false
  selectedCollectionId.value = null
  notes.value = ''
}

const createCollectionAndSelect = async () => {
  if (isCreating.value) return
  isCreating.value = true
  try {
    const res = await api.post('/collections', {
      name: newCollection.value.name,
      description: newCollection.value.description || undefined,
      is_public: newCollection.value.is_public,
    })
    const collectionId = res.data.collection_id as number
    newCollection.value = { name: '', description: '', is_public: true }
    showCreateForm.value = false
    await fetchCollections()
    selectCollection(collectionId)
  } catch (e) {
    console.error('Error creating collection:', e)
  } finally {
    isCreating.value = false
  }
}

const confirmAddAll = async () => {
  if (!selectedCollectionId.value || isSubmitting.value) return
  isSubmitting.value = true
  progressText.value = t('addAllToCollection.collectingIds')

  let totalAdded = 0
  let totalSkipped = 0
  let totalInvalid = 0

  try {
    const ids = await props.loadAllDefinitionIds((current, expected) => {
      progressText.value = expected
        ? t('addAllToCollection.collectingProgress', { current, total: expected })
        : t('addAllToCollection.collectingIds')
    })
    if (ids.length === 0) {
      showSuccess(t('addAllToCollection.nothingToAdd'))
      emit('update:modelValue', false)
      return
    }

    // Chunk defensively so very large result sets don't create huge single requests.
    const CHUNK = 2000
    const collectionId = selectedCollectionId.value
    const notesPayload = notes.value.trim() ? notes.value : undefined
    for (let i = 0; i < ids.length; i += CHUNK) {
      const chunk = ids.slice(i, i + CHUNK)
      progressText.value = t('addAllToCollection.addingProgress', {
        current: Math.min(i + chunk.length, ids.length),
        total: ids.length,
      })
      const res = await bulkAddDefinitionsToCollection(collectionId, chunk, notesPayload)
      totalAdded += res.data.added ?? 0
      totalSkipped += res.data.skipped ?? 0
      totalInvalid += (res.data.invalid_definition_ids || []).length
    }

    showSuccess(
      t('addAllToCollection.success', {
        added: totalAdded,
        skipped: totalSkipped,
      })
    )
    emit('added', { added: totalAdded, skipped: totalSkipped, invalid: totalInvalid })
    await fetchCollections()
    emit('update:modelValue', false)
  } catch (e) {
    console.error('Error adding all to collection:', e)
  } finally {
    isSubmitting.value = false
    progressText.value = ''
  }
}
</script>
