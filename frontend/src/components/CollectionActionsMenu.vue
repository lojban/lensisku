<template>
  <ToolbarSelectDropdown
    v-if="auth.state.isLoggedIn"
    trigger-icon="ellipsis"
    trigger-class="dropdown-action-trigger"
  >
    <template #label>{{ t('collectionDetail.actions') }}</template>
    <ToolbarSelectDropdownItem
      v-if="isOwner"
      class="text-gray-700 hover:bg-gray-100"
      @click="openEditCollectionModal"
    >
      <Edit3 class="h-4 w-4 shrink-0" aria-hidden="true" />
      {{ t('collectionDetail.editCollectionInfo') }}
    </ToolbarSelectDropdownItem>
    <ToolbarSelectDropdownItem
      v-if="isOwner"
      class="text-gray-700 hover:bg-gray-100"
      @click="onAddItem"
    >
      <PlusCircle class="h-4 w-4 shrink-0" aria-hidden="true" />
      {{ t('collectionDetail.addItemButton') }}
    </ToolbarSelectDropdownItem>
    <ToolbarSelectDropdownItem
      v-if="isOwner"
      class="text-gray-700 hover:bg-gray-100"
      @click="router.push(`/collections/${collectionId}/custom-text-bulk`)"
    >
      <FileText class="h-4 w-4 shrink-0" aria-hidden="true" />
      {{ t('collectionDetail.bulkEditCustomText') }}
    </ToolbarSelectDropdownItem>
    <ToolbarSelectDropdownItem
      v-if="canExportFilteredItems"
      class="text-blue-600 hover:bg-blue-50"
      @click="goToCollectionExport"
    >
      <FileExportIcon class="h-4 w-4 shrink-0" aria-hidden="true" />
      {{ t('collectionDetail.exportFilteredItems') }}
    </ToolbarSelectDropdownItem>
    <ToolbarSelectDropdownItem
      v-if="isOwner"
      class="text-blue-600 hover:bg-blue-50"
      @click="triggerJsonImport"
    >
      <FileImportIcon class="h-4 w-4 shrink-0" aria-hidden="true" />
      {{ t('collectionDetail.importFullButton') }}
    </ToolbarSelectDropdownItem>
    <ToolbarSelectDropdownItem
      class="text-indigo-600 hover:bg-indigo-50"
      @click="handleCloneCollection"
    >
      <Copy class="h-4 w-4 shrink-0" aria-hidden="true" />
      {{ t('collectionDetail.cloneCollection') }}
    </ToolbarSelectDropdownItem>
    <ToolbarSelectDropdownItem
      v-if="isOwner"
      class="text-green-600 hover:bg-green-50"
      @click="openMergeModal"
    >
      <GitMerge class="h-4 w-4 shrink-0" aria-hidden="true" />
      {{ t('collectionDetail.mergeCollections') }}
    </ToolbarSelectDropdownItem>
    <ToolbarSelectDropdownItem
      v-if="isOwner"
      class="text-teal-600 hover:bg-teal-50"
      @click="showMediaBulkZipModal = true"
    >
      <Images class="h-4 w-4 shrink-0" aria-hidden="true" />
      {{ t('collectionCustomTextBulk.mediaBulkZipButton') }}
    </ToolbarSelectDropdownItem>
    <ToolbarSelectDropdownItem
      v-if="isOwner"
      class="text-red-600 hover:bg-red-50"
      @click="handleDelete"
    >
      <Trash2 class="h-4 w-4 shrink-0" aria-hidden="true" />
      {{ t('collectionDetail.deleteCollection') }}
    </ToolbarSelectDropdownItem>
  </ToolbarSelectDropdown>

  <FileInput ref="jsonImportInput" accept=".json" class="hidden" @change="handleJsonFileSelect" />

  <ModalComponent
    :show="showEditModal"
    :title="t('collectionDetail.editCollectionTitle')"
    @close="cancelEditCollectionModal()"
  >
    <form @submit.prevent="performUpdateCollection">
      <div class="space-y-4">
        <div v-if="isOwner" class="flex flex-col items-center mb-2">
          <div class="relative group w-28 h-28">
            <div v-show="isEditCollectionImageLoading" class="collection-edit-logo-placeholder">
              <LoadingSpinner variant="inline" />
            </div>
            <CollectionCoverLightbox
              v-if="editCollectionImageUrl"
              v-show="!isEditCollectionImageLoading"
              :image-url="editCollectionImageUrl"
              :alt="t('collectionDetail.coverImageAlt', { name: editForm.name })"
              :aria-label="t('collectionDetail.coverLightboxDialog', { name: editForm.name })"
              :close-aria-label="t('collectionDetail.coverLightboxClose')"
            >
              <div class="collection-edit-logo">
                <div
                  :key="editCollectionImageUrl || 'none'"
                  class="collection-cover-thumb"
                  role="img"
                  :aria-label="t('collectionDetail.coverImageAlt', { name: editForm.name })"
                  :style="{ backgroundImage: `url(${editCollectionImageUrl})` }"
                />
              </div>
            </CollectionCoverLightbox>
            <div
              v-show="!isEditCollectionImageLoading && !editCollectionImageUrl"
              class="collection-edit-logo-placeholder"
            >
              <PlayingCardsFanIcon class="h-14 w-14" />
            </div>

            <div
              v-if="isCollectionImageUploading"
              class="absolute inset-0 rounded-xl bg-black bg-opacity-50 flex items-center justify-center"
            >
              <svg class="w-16 h-16 transform -rotate-90" viewBox="0 0 100 100">
                <circle cx="50" cy="50" r="45" fill="none" stroke="#4B5563" stroke-width="8" />
                <circle
                  cx="50"
                  cy="50"
                  r="45"
                  fill="none"
                  stroke="#3B82F6"
                  stroke-width="8"
                  :stroke-dasharray="collectionImageCircumference"
                  :stroke-dashoffset="
                    collectionImageCircumference -
                    (collectionImageUploadProgress / 100) * collectionImageCircumference
                  "
                  class="transition-all duration-300"
                />
              </svg>
              <span class="absolute text-white text-sm font-medium">
                {{ Math.round(collectionImageUploadProgress) }}%
              </span>
            </div>

            <div
              v-if="!isCollectionImageUploading"
              class="absolute -bottom-4 left-1/2 transform -translate-x-1/2 flex gap-3 pointer-events-none"
            >
              <label
                class="cursor-pointer p-2 bg-white border border-gray-300 rounded-full text-blue-600 hover:bg-blue-50 transition-all shadow-md pointer-events-auto"
                :title="t('collectionDetail.collectionCoverUploadTitle')"
              >
                <FileInput
                  accept="image/*"
                  class="hidden"
                  @change="handleCollectionCoverFileChange"
                />
                <Camera class="h-5 w-5" />
              </label>
              <button
                v-if="collection?.has_cover_image"
                type="button"
                class="p-2 bg-white border border-gray-300 rounded-full text-red-600 hover:bg-red-50 transition-all shadow-md pointer-events-auto"
                :title="t('collectionDetail.collectionCoverRemoveTitle')"
                @click.stop="handleCollectionCoverRemove"
              >
                <Trash2 class="h-5 w-5" />
              </button>
            </div>
          </div>

          <div class="h-6" />

          <p v-if="collectionImageUploadError" class="mt-2 text-sm text-red-600">
            {{ collectionImageUploadError }}
          </p>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            {{ t('collectionDetail.nameLabel') }}
          </label>
          <Input v-model="editForm.name" type="text" required class="input-field w-full" />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            {{ t('collectionDetail.descriptionLabel') }}
          </label>
          <Textarea v-model="editForm.description" rows="3" class="textarea-field" />
        </div>

        <div class="flex items-center">
          <Checkbox
            :id="`edit_is_public_${collectionId}`"
            v-model="editForm.is_public"
            class="checkbox-toggle"
          />
          <label :for="`edit_is_public_${collectionId}`" class="ml-2 text-sm text-gray-700">
            {{ t('collectionDetail.makePublicLabel') }}
          </label>
        </div>

        <fieldset class="course-settings rounded-md border border-gray-200 p-3">
          <legend class="px-1 text-sm font-medium text-gray-700">
            {{ t('collectionDetail.courseSettingsLegend') }}
          </legend>
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div class="flex min-w-0 flex-1 flex-col">
              <label :for="`edit_challenge_mode_${collectionId}`" class="text-sm text-gray-700">
                {{ t('collectionDetail.challengeModeLabel') }}
              </label>
              <select
                :id="`edit_challenge_mode_${collectionId}`"
                v-model="courseSettings.challenge_mode"
                class="input-field mt-1 w-full text-sm"
                :disabled="isCourseSettingsLoading"
              >
                <option value="off">{{ t('collectionDetail.challengeModeOff') }}</option>
                <option value="fixed">{{ t('collectionDetail.challengeModeFixed') }}</option>
                <option value="auto">{{ t('collectionDetail.challengeModeAuto') }}</option>
              </select>
            </div>
            <Button
              variant="toolbar"
              type="button"
              class="shrink-0"
              :disabled="isApplyingMix"
              @click="applyMixNow"
            >
              {{
                isApplyingMix
                  ? t('collectionDetail.applyingMix')
                  : courseSettings.challenge_mode === 'auto'
                    ? t('collectionDetail.convertCourseToAuto')
                    : t('collectionDetail.applyMixNow')
              }}
            </Button>
          </div>
        </fieldset>
      </div>

      <div class="mt-6 flex justify-end gap-3">
        <Button variant="cancel" type="button" @click="cancelEditCollectionModal()">
          {{ t('collectionDetail.cancel') }}
        </Button>
        <Button variant="edit" type="submit" :disabled="isSubmitting || isCollectionImageUploading">
          {{ isSubmitting ? t('collectionDetail.saving') : t('collectionDetail.saveChanges') }}
        </Button>
      </div>
    </form>
  </ModalComponent>

  <CollectionMediaBulkZipModal
    v-model="showMediaBulkZipModal"
    :collection-id="collectionId"
    @success="onMediaBulkZipSuccess"
  />

  <ModalComponent
    :show="showMergeModal"
    :title="t('collectionDetail.mergeCollectionTitle')"
    @close="showMergeModal = false"
  >
    <div v-if="isLoadingCollections" class="flex justify-center py-4">
      <LoadingSpinner variant="inline" size="sm" />
    </div>

    <form v-else class="space-y-4" @submit.prevent="performMerge">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">
          {{ t('collectionDetail.selectCollectionToMergeLabel') }}
        </label>
        <Select
          v-model="selectedCollectionToMerge"
          searchable
          :search-placeholder="t('filters.searchOptions')"
          :empty-filter-label="t('filters.noMatches')"
          select-class="input-field w-full"
          required
          :options="mergeCollectionOptions"
        />
      </div>
      <div class="flex items-center space-x-2">
        <Checkbox
          :id="`createNew_${collectionId}`"
          v-model="createNewCollection"
          class="checkbox-toggle"
        />
        <label :for="`createNew_${collectionId}`" class="text-sm text-gray-700">
          {{ t('collectionDetail.createNewFromMergeLabel') }}
        </label>
      </div>
      <div v-if="createNewCollection">
        <label class="block text-sm font-medium text-gray-700 mb-1">
          {{ t('collectionDetail.newCollectionNameLabel') }}
        </label>
        <Input
          v-model="newMergedCollectionName"
          type="text"
          required
          class="input-field w-full"
          :placeholder="t('collectionDetail.newCollectionNamePlaceholder')"
        />
      </div>
      <div class="flex justify-end space-x-3 pt-4">
        <Button variant="cancel" type="button" @click="showMergeModal = false">
          {{ t('collectionDetail.cancel') }}
        </Button>
        <Button variant="edit" type="submit" :disabled="!isValidMerge || isMerging">
          {{ isMerging ? t('collectionDetail.merging') : t('collectionDetail.mergeButton') }}
        </Button>
      </div>
    </form>
  </ModalComponent>

  <DeleteConfirmationModal
    :show="showDeleteCollectionConfirm"
    :title="t('collectionDetail.confirmDeleteTitle')"
    class="z-[65]"
    :message="t('collectionDetail.confirmDeleteMessage')"
    :is-deleting="isDeletingCollection"
    @confirm="performDeleteCollection"
    @cancel="showDeleteCollectionConfirm = false"
  />
</template>

<script setup lang="ts">
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import { Camera, Copy, Edit3, FileText, GitMerge, Images, PlusCircle, Trash2 } from '@lucide/vue'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import {
  cloneCollection,
  deleteCollection,
  getCollection,
  getCollectionImage,
  getCollections,
  importCollectionFull,
  mergeCollection,
  removeCollectionImage,
  updateCollection,
  getCourseSettings,
  updateCourseSettings,
  applyChallengeMixToCollection,
  convertCollectionToAuto,
  updateCollectionImage,
} from '@/api'
import CollectionMediaBulkZipModal from '@/components/CollectionMediaBulkZipModal.vue'
import DeleteConfirmationModal from '@/components/DeleteConfirmation.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSuccessToast } from '@/composables/useSuccessToast'
import { getApiErrorMessage, uploadImageErrorMessage } from '@/utils/apiError'
import { combinedFiltersToQuery, compactQuery } from '@/utils/routeQuery'
import {
  Button,
  Checkbox,
  CollectionCoverLightbox,
  FileExportIcon,
  FileImportIcon,
  FileInput,
  Input,
  PlayingCardsFanIcon,
  Select,
  Textarea,
  ToolbarSelectDropdown,
  ToolbarSelectDropdownItem,
} from '@packages/ui'

export interface CollectionActionsMenuModel {
  name: string
  description?: string | null
  is_public: boolean
  has_cover_image?: boolean
}

const props = defineProps<{
  collectionId: string | number
  collection: CollectionActionsMenuModel | null
  isOwner: boolean
  /**
   * When set, "Export" uses this query (collection page filters).
   * Otherwise the whole collection is exported.
   */
  exportQuery?: Record<string, string>
  /** Open the add-item flow on this page instead of navigating to the collection. */
  inlineAddItem?: boolean
}>()

const emit = defineEmits<{
  updated: [collection: CollectionActionsMenuModel]
  /** Items or media changed; parent lists should reload. */
  refresh: []
  'add-item': []
}>()

const { t } = useI18n()
const router = useRouter()
const auth = useAuth()
const { showError, clearError } = useError()
const { showSuccess } = useSuccessToast()

const editForm = ref({
  name: '',
  description: '',
  is_public: false,
})

const showEditModal = ref(false)
const courseSettings = ref<{ challenge_mode: 'off' | 'fixed' | 'auto' }>({ challenge_mode: 'off' })
const initialMode = ref<'off' | 'fixed' | 'auto'>('off')
const isCourseSettingsLoading = ref(false)
const isApplyingMix = ref(false)

async function loadCourseSettings() {
  isCourseSettingsLoading.value = true
  try {
    const { data } = await getCourseSettings(props.collectionId)
    const mode = data.challenge_mode || (data.auto_challenge_mix ? 'fixed' : 'off')
    courseSettings.value = { challenge_mode: mode }
    initialMode.value = mode
  } catch (err) {
    console.error('Error loading course settings:', err)
  } finally {
    isCourseSettingsLoading.value = false
  }
}

async function applyMixNow() {
  if (isApplyingMix.value) return
  isApplyingMix.value = true
  try {
    if (courseSettings.value.challenge_mode === 'auto') {
      await convertCollectionToAuto(props.collectionId)
      showSuccess(t('collectionDetail.convertAutoDone'))
    } else {
      await applyChallengeMixToCollection(props.collectionId)
      showSuccess(t('collectionDetail.applyMixDone'))
    }
  } catch (err) {
    showError(
      getApiErrorMessage(err) ||
        t(
          courseSettings.value.challenge_mode === 'auto'
            ? 'collectionDetail.convertAutoError'
            : 'collectionDetail.applyMixError'
        )
    )
  } finally {
    isApplyingMix.value = false
  }
}
const showMediaBulkZipModal = ref(false)
const showMergeModal = ref(false)
const showDeleteCollectionConfirm = ref(false)
const isSubmitting = ref(false)
const isDeletingCollection = ref(false)
const isImportingJson = ref(false)
const isCloning = ref(false)
const isCollectionImageUploading = ref(false)
const collectionImageUploadError = ref('')
const collectionImageUploadProgress = ref(0)
const collectionImageCircumference = computed(() => 2 * Math.PI * 45)
const isEditCollectionImageLoading = ref(false)
const editCollectionImageUrl = ref<string | null>(null)

const selectedCollectionToMerge = ref<string | number>('')
const createNewCollection = ref(false)
const newMergedCollectionName = ref('')
const isLoadingCollections = ref(false)
const isMerging = ref(false)
const availableCollections = ref<{ collection_id: number; name: string; item_count: number }[]>([])

const jsonImportInput = ref<{ input: HTMLInputElement | null } | null>(null)

const canExportFilteredItems = computed(
  () => props.collection && (props.collection.is_public || props.isOwner)
)

const isValidMerge = computed(() => {
  if (!selectedCollectionToMerge.value) return false
  if (createNewCollection.value && !newMergedCollectionName.value.trim()) return false
  return String(selectedCollectionToMerge.value) !== String(props.collectionId)
})

const mergeCollectionOptions = computed(() => [
  { value: '', label: t('collectionDetail.selectCollectionPlaceholder') },
  ...availableCollections.value.map((c) => ({
    value: c.collection_id,
    label: `${c.name} (${t('collectionDetail.itemsCount', { count: c.item_count })}) `,
  })),
])

function syncEditForm(source: CollectionActionsMenuModel) {
  editForm.value = {
    name: source.name,
    description: source.description || '',
    is_public: source.is_public,
  }
}

function cancelEditCollectionModal() {
  showEditModal.value = false
  collectionImageUploadError.value = ''
}

function openEditCollectionModal() {
  if (props.collection) syncEditForm(props.collection)
  showEditModal.value = true
  loadCourseSettings()
  collectionImageUploadError.value = ''
  refreshEditCollectionCoverUrl()
}

function handleEditCollectionImageLoad() {
  isEditCollectionImageLoading.value = false
}

function handleEditCollectionImageError() {
  isEditCollectionImageLoading.value = false
  editCollectionImageUrl.value = null
}

function refreshEditCollectionCoverUrl() {
  if (props.collection?.has_cover_image) {
    isEditCollectionImageLoading.value = true
    const url = `${getCollectionImage(props.collectionId)}?t=${Date.now()}`
    const probe = new Image()
    probe.onload = () => {
      editCollectionImageUrl.value = url
      handleEditCollectionImageLoad()
    }
    probe.onerror = () => {
      handleEditCollectionImageError()
    }
    probe.src = url
  } else {
    editCollectionImageUrl.value = null
    isEditCollectionImageLoading.value = false
  }
}

async function publishCollection(next?: CollectionActionsMenuModel) {
  const data = next ?? (await getCollection(props.collectionId)).data
  emit('updated', data)
  return data
}

async function handleCollectionCoverFileChange(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  const input = event.target as HTMLInputElement
  if (input) input.value = ''
  if (!file) return

  collectionImageUploadError.value = ''
  collectionImageUploadProgress.value = 0

  if (!file.type.startsWith('image/')) {
    collectionImageUploadError.value = t('profile.uploadError.invalidType')
    return
  }
  if (file.size > 5 * 1024 * 1024) {
    collectionImageUploadError.value = t('profile.uploadError.tooLarge')
    return
  }

  try {
    isCollectionImageUploading.value = true
    const reader = new FileReader()
    reader.onload = async (e) => {
      let progressInterval: ReturnType<typeof setInterval> | undefined
      try {
        const raw = e.target?.result
        const base64Data = typeof raw === 'string' ? (raw.split(',')[1] ?? '') : ''
        const imageData = { data: base64Data, mime_type: file.type }

        editCollectionImageUrl.value = URL.createObjectURL(file)
        isEditCollectionImageLoading.value = false

        let progress = 0
        progressInterval = setInterval(() => {
          progress += 10
          if (progress > 90 && progressInterval) clearInterval(progressInterval)
          collectionImageUploadProgress.value = Math.min(progress, 90)
        }, 100)

        await updateCollectionImage(props.collectionId, imageData)
        if (progressInterval) clearInterval(progressInterval)
        collectionImageUploadProgress.value = 100

        const meta = await publishCollection()
        syncEditForm(meta)
        refreshEditCollectionCoverUrl()
        showSuccess(t('collectionDetail.collectionCoverUploadSuccess'))

        setTimeout(() => {
          isCollectionImageUploading.value = false
          collectionImageUploadProgress.value = 0
        }, 500)
      } catch (err: unknown) {
        if (progressInterval) clearInterval(progressInterval)
        console.error('Collection cover upload error:', err)
        collectionImageUploadError.value = uploadImageErrorMessage(err, t)
        isCollectionImageUploading.value = false
        isEditCollectionImageLoading.value = false
        collectionImageUploadProgress.value = 0
      }
    }
    reader.onerror = () => {
      collectionImageUploadError.value = t('profile.uploadError.readError')
      isCollectionImageUploading.value = false
      isEditCollectionImageLoading.value = false
      collectionImageUploadProgress.value = 0
    }
    reader.readAsDataURL(file)
  } catch {
    collectionImageUploadError.value = t('profile.uploadError.uploadFailed')
    isCollectionImageUploading.value = false
    collectionImageUploadProgress.value = 0
  }
}

async function handleCollectionCoverRemove() {
  if (!confirm(t('collectionDetail.collectionCoverRemoveConfirm'))) return

  try {
    isCollectionImageUploading.value = true
    collectionImageUploadError.value = ''
    await removeCollectionImage(props.collectionId)
    const meta = await publishCollection()
    syncEditForm(meta)
    editCollectionImageUrl.value = null
    isEditCollectionImageLoading.value = false
    showSuccess(t('collectionDetail.collectionCoverRemoveSuccess'))
  } catch (err: unknown) {
    collectionImageUploadError.value =
      getApiErrorMessage(err) || t('collectionDetail.collectionCoverRemoveError')
  } finally {
    isCollectionImageUploading.value = false
  }
}

async function performUpdateCollection() {
  if (isSubmitting.value) return
  isSubmitting.value = true

  try {
    const response = await updateCollection(props.collectionId, editForm.value)
    if (courseSettings.value.challenge_mode !== initialMode.value) {
      await updateCourseSettings(props.collectionId, {
        challenge_mode: courseSettings.value.challenge_mode,
      })
      initialMode.value = courseSettings.value.challenge_mode
    }
    emit('updated', response.data)
    showEditModal.value = false
  } catch (error) {
    console.error('Error updating collection:', error)
  } finally {
    isSubmitting.value = false
  }
}

function onAddItem() {
  if (props.inlineAddItem) {
    emit('add-item')
    return
  }
  router.push({
    path: `/collections/${props.collectionId}`,
    query: { addItem: '1' },
  })
}

function goToCollectionExport() {
  router.push({
    path: '/export/search',
    query:
      props.exportQuery ??
      compactQuery({
        mode: 'dictionary',
        collection_only: '1',
        ...combinedFiltersToQuery({
          selectedLanguages: [],
          selectedCollections: [Number(props.collectionId)],
          usernames: [],
          excludeUsernames: [],
          selmaho: '',
          word_type: [],
          source_langid: 1,
          searchInPhrases: true,
          includePopularCollections: false,
          isExpanded: false,
        }),
      }),
  })
}

async function handleCloneCollection() {
  if (isCloning.value) return
  isCloning.value = true
  try {
    const response = await cloneCollection(props.collectionId)
    router.push(`/collections/${response.data.collection_id}`)
  } catch (error) {
    console.error('Error cloning collection:', error)
  } finally {
    isCloning.value = false
  }
}

function handleDelete() {
  showDeleteCollectionConfirm.value = true
}

async function performDeleteCollection() {
  isDeletingCollection.value = true
  try {
    await deleteCollection(props.collectionId)
    router.push('/library')
  } catch (error: unknown) {
    console.error('Error deleting collection:', error)
    showError(getApiErrorMessage(error) || 'Failed to delete collection')
  } finally {
    isDeletingCollection.value = false
    showDeleteCollectionConfirm.value = false
  }
}

function openMergeModal() {
  showMergeModal.value = true
  loadAvailableCollections()
}

async function loadAvailableCollections() {
  isLoadingCollections.value = true
  try {
    const response = await getCollections()
    availableCollections.value = response.data.collections.filter(
      (c) => c.collection_id !== Number(props.collectionId)
    )
  } catch (e) {
    showError('Failed to load collections')
    console.error('Error loading collections:', e)
  } finally {
    isLoadingCollections.value = false
  }
}

async function performMerge() {
  if (!isValidMerge.value || isMerging.value) return

  isMerging.value = true
  clearError()

  try {
    await mergeCollection({
      source_collection_id: Number(selectedCollectionToMerge.value),
      target_collection_id: Number(props.collectionId),
      new_collection_name: createNewCollection.value ? newMergedCollectionName.value : undefined,
    })

    if (createNewCollection.value) {
      router.push('/library')
    } else {
      await publishCollection()
      emit('refresh')
    }

    showMergeModal.value = false
    selectedCollectionToMerge.value = ''
    createNewCollection.value = false
    newMergedCollectionName.value = ''

    showSuccess(t('collectionDetail.mergeSuccess'))
  } catch (err: unknown) {
    showError(getApiErrorMessage(err) || 'Failed to merge collections')
  } finally {
    isMerging.value = false
  }
}

async function onMediaBulkZipSuccess() {
  try {
    await publishCollection()
  } catch (e) {
    console.error('Failed to refresh collection after bulk images:', e)
  }
  emit('refresh')
}

function triggerJsonImport() {
  jsonImportInput.value?.input?.click()
}

async function handleJsonFileSelect(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  isImportingJson.value = true
  clearError()

  try {
    const fileContent = await file.text()
    const jsonData = JSON.parse(fileContent)

    if (!jsonData.collection || !Array.isArray(jsonData.items)) {
      showError(t('collectionDetail.importFullFormatError'))
      return
    }

    const payload = {
      collection: {
        name: jsonData.collection.name,
        description: jsonData.collection.description ?? null,
        is_public: jsonData.collection.is_public ?? true,
      },
      items: jsonData.items,
      levels: Array.isArray(jsonData.levels) ? jsonData.levels : [],
    }

    const response = await importCollectionFull(payload)
    const {
      collection: newCollection,
      imported_count,
      skipped_count,
      levels_created,
      warnings,
    } = response.data

    let message =
      t('collectionDetail.importFullSuccess', {
        name: newCollection.name,
        imported: imported_count,
        skipped: skipped_count,
        levels: levels_created,
      }) ||
      `Created "${newCollection.name}" with ${imported_count} items, ${levels_created} levels.`
    if (warnings?.length) {
      message += '<br/><br/>' + warnings.slice(0, 5).join('<br/>')
      if (warnings.length > 5) message += `<br/>... ${warnings.length - 5} more`
    }
    showSuccess(message, 4000)
    router.push(`/collections/${newCollection.collection_id}`)
  } catch (error: unknown) {
    showError(getApiErrorMessage(error) || t('collectionDetail.importJsonError'))
  } finally {
    isImportingJson.value = false
    if (jsonImportInput.value?.input) jsonImportInput.value.input.value = ''
  }
}
</script>
