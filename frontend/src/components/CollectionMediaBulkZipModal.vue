<template>
  <ModalComponent
    :show="modelValue"
    :title="t('collectionCustomTextBulk.mediaBulkModalTitle')"
    @close="close"
  >
    <div class="space-y-4 text-sm text-gray-700">
      <p class="font-medium text-gray-900">{{ t('collectionCustomTextBulk.mediaBulkIntro') }}</p>

      <ol class="list-decimal space-y-2 pl-5 text-gray-700">
        <li>{{ t('collectionCustomTextBulk.mediaBulkStep1') }}</li>

        <li>{{ t('collectionCustomTextBulk.mediaBulkStep2') }}</li>

        <li>{{ t('collectionCustomTextBulk.mediaBulkStep3') }}</li>

        <li>{{ t('collectionCustomTextBulk.mediaBulkStep4') }}</li>
      </ol>

      <p class="text-xs font-medium text-gray-800">
        {{ t('collectionCustomTextBulk.mediaBulkManifestExampleCaption') }}
      </p>

      <pre
        class="overflow-x-auto rounded-md border border-gray-200 bg-gray-50 p-3 text-xs text-gray-800 whitespace-pre-wrap"
        >{{ MANIFEST_JSON_EXAMPLE }}</pre
      >

      <p class="text-xs text-gray-600">{{ t('collectionCustomTextBulk.mediaBulkLimits') }}</p>

      <p class="text-xs text-gray-600">
        {{
          t('collectionCustomTextBulk.mediaBulkMultipartHint', {
            id: String(props.collectionId),
          })
        }}
      </p>
      <FileDropzone
        accept=".zip,application/zip"
        :choose-file-text="t('fileDropzone.chooseFile')"
        :or-drag-drop-text="t('fileDropzone.orDragDrop')"
        :types-note-text="t('collectionCustomTextBulk.mediaBulkAcceptZip')"
        :dropzone-aria-label="t('collectionCustomTextBulk.mediaBulkDropAria')"
        :input-aria-label="t('collectionCustomTextBulk.mediaBulkInputAria')"
        :disabled="uploading"
        :validate-file="isLikelyZipFile"
        @select="onZipSelected"
        @reject="onZipRejected"
      />
      <p v-if="file" class="text-sm text-gray-800">
        {{ t('collectionCustomTextBulk.mediaBulkSelected', { name: file.name }) }}
      </p>
    </div>
    <template #footer>
      <div class="flex flex-wrap justify-end gap-2">
        <button type="button" class="ui-btn--neutral-muted" :disabled="uploading" @click="close">
          {{ t('collectionCustomTextBulk.importCancel') }}
        </button>
        <button type="button" class="ui-btn--create" :disabled="!file || uploading" @click="submit">
          {{
            uploading
              ? t('collectionCustomTextBulk.mediaBulkUploading')
              : t('collectionCustomTextBulk.mediaBulkSubmit')
          }}
        </button>
      </div>
    </template>
  </ModalComponent>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import FileDropzone from '@/components/FileDropzone.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import { uploadCollectionMediaBulkZip } from '@/api'
import { useError } from '@/composables/useError'
import { useSuccessToast } from '@/composables/useSuccessToast'

/** Kept in code, not locale files: JSON `{{` breaks vue-i18n message compilation. */
const MANIFEST_JSON_EXAMPLE = `[
  { "filename": "a.webp", "side": "front", "item_id": 123 },
  { "filename": "b.jpg", "side": "back", "position": 4 },
  { "filename": "c.png", "side": "front", "free_content_front": "Hello", "free_content_back": "coi" }
]`

const props = defineProps<{
  /** v-model visibility */
  modelValue: boolean
  collectionId: string | number
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  /** After a successful upload; parent should refresh collection/items. */
  success: []
  /** True while the ZIP is uploading (parent may disable other actions). */
  busy: [value: boolean]
}>()

const { t } = useI18n()
const { showError, clearError } = useError()
const { showSuccess } = useSuccessToast()

const file = ref<File | null>(null)
const uploading = ref(false)

watch(uploading, (v) => emit('busy', v))

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      file.value = null
      clearError()
    }
  }
)

function isLikelyZipFile(f: File): boolean {
  const n = f.name.toLowerCase()
  return (
    n.endsWith('.zip') || f.type === 'application/zip' || f.type === 'application/x-zip-compressed'
  )
}

function onZipSelected(f: File) {
  file.value = f
}

function onZipRejected() {
  showError(t('collectionCustomTextBulk.mediaBulkWrongFileType'))
}

function close() {
  if (uploading.value) return
  emit('update:modelValue', false)
  file.value = null
}

async function submit() {
  const zip = file.value
  if (!zip) return
  uploading.value = true
  clearError()
  try {
    const { data } = await uploadCollectionMediaBulkZip(props.collectionId, zip)
    const d = data as { attached: number; created_items: number; warnings?: string[] }
    let msg = t('collectionCustomTextBulk.mediaBulkSuccess', {
      attached: d.attached,
      created: d.created_items,
    })
    if (d.warnings?.length) {
      msg +=
        '\n\n' +
        d.warnings.slice(0, 8).join('\n') +
        (d.warnings.length > 8 ? `\n… ${d.warnings.length - 8} more` : '')
    }
    uploading.value = false
    showSuccess(msg, 8000)
    emit('update:modelValue', false)
    file.value = null
    emit('success')
  } catch (e: unknown) {
    const ax = e as { response?: { data?: { error?: string } }; message?: string }
    showError(
      ax.response?.data?.error || ax.message || t('collectionCustomTextBulk.mediaBulkError')
    )
  } finally {
    uploading.value = false
  }
}
</script>
