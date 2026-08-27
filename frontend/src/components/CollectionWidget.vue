<template>
  <div class="collection-widget">
    <!-- Add to Collection Button -->
    <Button
      variant="empty"
      class="flex items-center gap-2 hover:text-yellow-600"
      :title="t('collectionWidget.addToCollection')"
      @click.stop="openModal"
    >
      <StarPlus class="w-4 h-4" />
    </Button>
    <ModalComponent :show="showModal" :title="t('collectionWidget.modalTitle')" @close="closeModal">
      <!-- Header -->
      <template #header>
        <h3 class="text-xl font-bold">{{ t('collectionWidget.modalTitle') }}</h3>
      </template>
      <!-- Loading State -->
      <LoadingSpinner v-if="isLoading" variant="inline" class="py-4" />
      <!-- Collections List -->

      <div v-else>
        <!-- Create New Collection -->
        <div v-if="collections.length > 0" class="flex justify-center mb-4">
          <IconButton
            button-classes="ui-btn--create"
            :label="t('collectionWidget.createNew')"
            @click="showCreateForm = true"
          />
        </div>
        <!-- Empty State -->
        <div v-if="collections.length === 0" class="px-3 py-4 text-center">
          <p class="text-sm text-gray-500 mb-2">{{ t('collectionWidget.noCollections') }}</p>
          <IconButton
            button-classes="ui-btn--create mt-4 mx-auto"
            :label="t('collectionWidget.createFirst')"
            @click="showCreateForm = true"
          />
        </div>
        <!-- Collections -->
        <div v-else class="max-h-64 overflow-y-auto overflow-x-hidden space-y-1">
          <Button
            v-for="collection in sortedCollections"
            :key="collection.collection_id"
            variant="plain"
            :disabled="isAddingTo === collection.collection_id"
            class="w-full min-w-0 px-3 py-2 text-left text-sm rounded-md flex items-center justify-between gap-2 group transition-colors [&>span]:!whitespace-normal [&>span]:min-w-0 [&>span]:w-full [&>span]:justify-between"
            :class="{
              'bg-indigo-100 hover:bg-indigo-200':
                selectedCollectionId === collection.collection_id,
              'hover:bg-gray-100': selectedCollectionId !== collection.collection_id,
            }"
            @click="addToCollection(collection.collection_id)"
          >
            <div class="min-w-0 flex-1 flex flex-wrap items-baseline gap-x-2 gap-y-0.5 text-left">
              <span class="text-sm text-gray-700 break-words">{{ collection.name }}</span>
              <span
                v-if="containingIds.has(collection.collection_id)"
                class="badge badge-muted shrink-0"
              >
                {{ t('collectionWidget.alreadyIncluded') }}
              </span>
              <span class="shrink-0 italic text-xs text-gray-500 whitespace-nowrap">
                {{ t('collectionWidget.itemsCount', { count: collection.item_count }) }}</span
              >
            </div>
            <span
              v-if="isAddingTo === collection.collection_id"
              class="shrink-0 text-indigo-600 animate-spin text-sm"
              >↻</span
            >
            <span v-else class="shrink-0 text-gray-400 invisible group-hover:visible">{{
              selectedCollectionId === collection.collection_id
                ? t('collectionWidget.selected')
                : t('collectionWidget.select')
            }}</span>
          </Button>
        </div>
      </div>
      <!-- Create Collection Form -->
      <div v-if="showCreateForm" class="border-t mt-2 pt-2">
        <form class="space-y-3" @submit.prevent="createAndAddToCollection">
          <div>
            <label class="block text-xs font-medium text-gray-700 mb-1">{{
              t('collectionWidget.collectionNameLabel')
            }}</label>
            <Input v-model="newCollection.name" type="text" required class="w-full input-field" />
          </div>

          <div>
            <label class="block text-xs font-medium text-gray-700 mb-1">{{
              t('collectionWidget.descriptionLabel')
            }}</label>
            <Textarea v-model="newCollection.description" rows="2" class="textarea-field" />
          </div>

          <div class="flex items-center space-x-2">
            <Checkbox id="is_public" v-model="newCollection.is_public" class="checkbox-toggle" />
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
              <Button
                variant="cancel"
                type="button"
                :disabled="isCreating"
                @click="showCreateForm = false"
              >
                {{ t('collectionWidget.cancel') }}
              </Button>
              <Button variant="create" type="submit" :disabled="isCreating">
                {{ isCreating ? t('collectionDetail.saving') : t('collectionWidget.createAndAdd') }}
              </Button>
            </div>
          </div>
        </form>
      </div>
      <!-- Notes Input -->
      <div v-if="showNotesInput" class="border-t mt-2 pt-2">
        <label class="block text-xs font-medium text-gray-700 mb-1">{{
          t('collectionWidget.notesLabel')
        }}</label>
        <Textarea
          v-model="notes"
          rows="2"
          :placeholder="t('collectionWidget.notesPlaceholder')"
          class="textarea-field"
        />
        <div class="mt-2 flex flex-col gap-2">
          <!-- Progress bar when saving -->
          <div
            v-if="isAddingTo === selectedCollectionId"
            class="w-full h-1.5 bg-gray-200 rounded-full overflow-hidden"
          >
            <div class="h-full w-1/3 bg-indigo-500 rounded-full progress-indeterminate" />
          </div>

          <div class="flex justify-end gap-2">
            <Button
              variant="cancel"
              :disabled="isAddingTo === selectedCollectionId"
              @click="cancelAddWithNotes"
            >
              {{ t('collectionWidget.cancel') }}
            </Button>
            <Button
              variant="insert"
              :disabled="isAddingTo === selectedCollectionId"
              @click="confirmAddWithNotes"
            >
              {{ t('collectionWidget.addToCollectionButton') }}
            </Button>
          </div>
        </div>
      </div>
    </ModalComponent>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted, type PropType } from 'vue'
import { useI18n } from 'vue-i18n'

import { addCollectionItem, api, getCollectionMembership } from '@/api'
import { Button, Checkbox, IconButton, Input, Textarea } from '@packages/ui'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import {
  useCollectionsCache,
  type CachedCollection,
} from '@/composables/useCollectionsCache'
import { useSuccessToast } from '@/composables/useSuccessToast'
import { StarPlus } from '@lucide/vue'

const { t } = useI18n()
const { showSuccess } = useSuccessToast()
const {
  collections,
  hasLoaded,
  preload,
  refresh,
  setCollections,
} = useCollectionsCache()

const props = defineProps({
  definitionId: {
    type: Number,
    default: null,
  },
  itemId: {
    type: Number,
    default: null,
  },
  word: {
    type: String,
    required: true,
  },
  definitionText: {
    type: String,
    default: '',
  },
  sourceLangId: {
    type: Number,
    default: null,
  },
  targetLangId: {
    type: Number,
    default: null,
  },
  /** When true, membership is valsi+definition+languages (collection-item search hits). */
  isCollectionItem: {
    type: Boolean,
    default: false,
  },
  /** Optional seed from a parent; shared cache is the source of truth once loaded. */
  externalCollections: {
    type: Array as PropType<CachedCollection[]>,
    default: () => [],
  },
})

const isLoading = ref(false)
const showModal = ref(false)
const showCreateForm = ref(false)
const showNotesInput = ref(false)
const isCreating = ref(false)
const isAddingTo = ref<number | null>(null)
const selectedCollectionId = ref<number | null>(null)
const notes = ref('')

const newCollection = ref({
  name: '',
  description: '',
  is_public: true,
})

const emit = defineEmits<{
  (e: 'collection-updated', collections: CachedCollection[]): void
}>()

const containingIds = ref<Set<number>>(new Set())

const sortedCollections = computed(() => {
  const ids = containingIds.value
  return [...collections.value].sort((a, b) => {
    const aIn = ids.has(a.collection_id) ? 0 : 1
    const bIn = ids.has(b.collection_id) ? 0 : 1
    return aIn - bIn
  })
})

const addItemPayload = (extra?: Record<string, unknown>) => {
  const notesValue = notes.value
  // Collection-item cards without a dictionary id are forked as custom text.
  if (props.isCollectionItem && !props.definitionId) {
    return {
      free_content_front: props.word,
      free_content_back: props.definitionText || '',
      language_id: props.targetLangId || undefined,
      notes: notesValue,
      ...extra,
    }
  }
  return {
    definition_id: props.definitionId,
    notes: notesValue,
    ...extra,
  }
}

const refreshMembership = async () => {
  const body: { definition_id?: number; item_id?: number } = {}
  if (props.isCollectionItem && props.itemId) {
    body.item_id = props.itemId
  } else if (props.definitionId) {
    body.definition_id = props.definitionId
  } else {
    containingIds.value = new Set()
    return
  }
  try {
    const response = await getCollectionMembership(body)
    const ids = (response.data?.collection_ids || []) as number[]
    containingIds.value = new Set(ids)
  } catch (error) {
    console.error('Error checking collection membership:', error)
  }
}

const seedFromExternal = (list: CachedCollection[] | undefined) => {
  if (!list || list.length === 0) return
  if (!hasLoaded.value || collections.value.length === 0) {
    setCollections(list)
  }
}

const refreshAndEmit = async () => {
  const list = await refresh()
  emit('collection-updated', list)
  return list
}

const openModal = () => {
  showModal.value = true
  // Instant open from cache; spinner only when we have nothing to show yet.
  const needsSpinner = !hasLoaded.value && collections.value.length === 0
  if (needsSpinner) {
    isLoading.value = true
  }
  // Always revalidate in the background so list updates in place if it changed.
  void Promise.all([refreshAndEmit(), refreshMembership()]).finally(() => {
    isLoading.value = false
  })
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
    const response = await api.post('/collections', {
      name: newCollection.value.name,
      description: newCollection.value.description || undefined,
      is_public: newCollection.value.is_public,
    })

    const collectionId = response.data.collection_id

    await addCollectionItem(collectionId, addItemPayload())

    newCollection.value = { name: '', description: '', is_public: true }
    showCreateForm.value = false

    await refreshAndEmit()
    await refreshMembership()
  } catch (error) {
    console.error('Error creating collection:', error)
  } finally {
    isCreating.value = false
  }
}

const addToCollection = (collectionId: number) => {
  selectedCollectionId.value = collectionId
  showNotesInput.value = true
}

const confirmAddWithNotes = async () => {
  if (!selectedCollectionId.value) return

  isAddingTo.value = selectedCollectionId.value

  try {
    await addCollectionItem(selectedCollectionId.value, addItemPayload({ auto_progress: true }))

    const updatedCollection = collections.value.find(
      (c) => c.collection_id === selectedCollectionId.value
    )
    if (updatedCollection) {
      updatedCollection.item_count++
    }

    await refreshAndEmit()
    await refreshMembership()

    showSuccess(t('collectionWidget.addedSuccess'))

    showNotesInput.value = false
    notes.value = ''
  } catch (error) {
    console.error('Error adding to collection:', error)
  } finally {
    isAddingTo.value = null
    selectedCollectionId.value = null
  }
}

const cancelAddWithNotes = () => {
  showNotesInput.value = false
  notes.value = ''
  selectedCollectionId.value = null
}

const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as HTMLElement | null
  if (!target?.closest('.collection-widget')) {
    showCreateForm.value = false
    showNotesInput.value = false
  }
}

watch(
  () => props.externalCollections,
  (newCollections) => {
    seedFromExternal(newCollections as CachedCollection[])
  },
  { deep: true, immediate: true }
)

onMounted(() => {
  seedFromExternal(props.externalCollections as CachedCollection[])
  void preload()
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
/* Styles remain exactly the same */
</style>
