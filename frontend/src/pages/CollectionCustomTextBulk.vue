<template>

  <div class="flex flex-col min-h-[calc(100vh-6rem)] pb-32 md:pb-8">

    <div class="mb-4 flex w-full flex-col gap-3">

      <div class="flex w-full flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">

        <div class="min-w-0 w-full flex-1">

          <div class="flex items-center gap-2 text-gray-500 italic text-sm mb-1">
             <List class="w-5 h-5 shrink-0" aria-hidden="true" /> <span>{{
              t('collectionCustomTextBulk.pageHint')
            }}</span
            >
          </div>

          <h1 class="text-xl sm:text-2xl font-bold text-gray-800">
             {{ collection?.name || '…' }}
          </h1>

        </div>
        <div
          class="flex w-full min-w-0 shrink-0 flex-row flex-wrap items-center justify-between gap-3 sm:w-auto sm:justify-start"
        >
          <div class="flex flex-row items-center gap-0" role="group">
            <RouterLink
              :to="`/collections/${numericCollectionId}`"
              class="btn-aqua-zinc btn-aqua-group-item inline-flex items-center gap-2 text-sm md:flex-none"
            >
              <ArrowLeft class="w-4 h-4 shrink-0" aria-hidden="true" />
              {{ t('collectionCustomTextBulk.backToCollection') }}
            </RouterLink>
            <button
              v-if="!isLoading && isOwner"
              type="button"
              class="btn-aqua-white btn-aqua-group-item inline-flex items-center gap-2 text-sm md:flex-none"
              :disabled="isSaving || isImporting"
              @click="showImportModal = true"
            >
              <FileDown class="h-4 w-4 shrink-0" aria-hidden="true" />
              {{ isImporting ? t('collectionCustomTextBulk.importing') : t('collectionCustomTextBulk.importButton') }}
            </button>
          </div>
          <Dropdown v-if="!isLoading && isOwner" :trigger-label="t('collectionCustomTextBulk.exportMenuLabel')">
            <button
              type="button"
              class="w-full px-4 py-2 text-left text-sm text-cyan-600 hover:bg-cyan-50 flex items-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed"
              :disabled="isSaving || isImporting"
              @click="exportListAsCsv"
            >
              <FileUp class="h-4 w-4 shrink-0" aria-hidden="true" />
              {{ t('collectionCustomTextBulk.exportCsv') }}
            </button>
            <button
              type="button"
              class="w-full px-4 py-2 text-left text-sm text-emerald-600 hover:bg-emerald-50 flex items-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed"
              :disabled="isSaving || isImporting"
              @click="exportListAsTsv"
            >
              <FileUp class="h-4 w-4 shrink-0" aria-hidden="true" />
              {{ t('collectionCustomTextBulk.exportTsv') }}
            </button>
          </Dropdown>
        </div>
      </div>

      <p
        class="w-full text-sm text-amber-800 bg-amber-50 border border-amber-200 rounded-md px-3 py-2"
      >
         {{ t('collectionCustomTextBulk.disclaimer') }}
      </p>

    </div>
     <LoadingSpinner v-if="isLoading" class="py-12" />
    <div v-else-if="!isOwner" class="text-center py-12 text-gray-600">
       {{ t('collectionCustomTextBulk.ownerOnly') }}
    </div>
     <template v-else
      >
      <ModalComponent
        :show="showImportModal"
        :title="t('collectionCustomTextBulk.importButton')"
        @close="showImportModal = false"
      >
        <p class="mb-4 text-sm text-gray-600">
          {{ t('collectionCustomTextBulk.importHint') }}
        </p>
        <FileDropzone
          accept=".csv,.tsv,text/csv,text/tab-separated-values"
          :choose-file-text="t('fileDropzone.chooseFile')"
          :or-drag-drop-text="t('fileDropzone.orDragDrop')"
          :types-note-text="t('fileDropzone.acceptsCsvTsv')"
          :dropzone-aria-label="t('fileDropzone.ariaLabel')"
          :input-aria-label="t('collectionCustomTextBulk.importAria')"
          :disabled="isSaving || isImporting"
          :validate-file="isLikelyCsvOrTsvFile"
          @select="onImportFileSelected"
          @reject="onImportFileInvalid"
        />
      </ModalComponent>

      <div class="flex flex-col gap-4 flex-1 min-h-0">

        <div class="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center sm:justify-between">
          <p v-if="rows.length === 0" class="text-sm text-gray-600">
            {{ t('collectionCustomTextBulk.empty') }}
          </p>
          <p v-else class="text-sm text-gray-600">
            {{ t('collectionCustomTextBulk.rowCount', { count: rows.length }) }}
          </p>
        </div>
         <!-- Desktop: table -->
        <div
          class="hidden md:block overflow-x-auto border border-gray-200 rounded-lg bg-white shadow-sm"
        >

          <table class="min-w-full text-sm border-collapse">

            <thead>

              <tr class="bg-gray-100 border-b border-gray-200 text-left">

                <th class="px-2 py-2 font-semibold text-gray-700 min-w-[12rem]">
                   {{ t('collectionCustomTextBulk.colFront') }}
                </th>

                <th class="px-2 py-2 font-semibold text-gray-700 min-w-[12rem]">
                   {{ t('collectionCustomTextBulk.colBack') }}
                </th>

                <th class="px-2 py-2 font-semibold text-gray-700 w-24 text-right whitespace-nowrap">
                  {{ t('collectionCustomTextBulk.colActions') }}
                </th>

              </tr>

            </thead>

            <tbody>

              <tr
                v-for="(row, idx) in rows"
                :key="row.item_id"
                class="border-b border-gray-100 hover:bg-gray-50/80"
              >

                <td class="px-2 py-1 align-top">
                   <textarea
                    v-model="row.free_content_front"
                    rows="1"
                    class="bulk-sheet-input js-bulk-auto-text"
                    :aria-label="t('collectionCustomTextBulk.colFront')"
                    @input="onBulkTextInput"
                  />
                </td>

                <td class="px-2 py-1 align-top">
                   <textarea
                    v-model="row.free_content_back"
                    rows="1"
                    class="bulk-sheet-input js-bulk-auto-text"
                    :aria-label="t('collectionCustomTextBulk.colBack')"
                    @input="onBulkTextInput"
                  />
                </td>

                <td class="px-2 py-1 align-top text-right">
                  <button
                    type="button"
                    class="inline-flex items-center justify-center rounded-md p-1.5 text-red-600 hover:bg-red-50 disabled:opacity-40"
                    :disabled="isRowActionDisabled"
                    :aria-label="t('collectionCustomTextBulk.deleteRowAria')"
                    :title="t('collectionCustomTextBulk.deleteRow')"
                    @click="deleteSavedRow(idx)"
                  >
                    <Loader2
                      v-if="deletingItemId === row.item_id"
                      class="h-4 w-4 animate-spin"
                      aria-hidden="true"
                    />
                    <Trash2 v-else class="h-4 w-4" aria-hidden="true" />
                  </button>
                </td>

              </tr>

              <tr
                v-for="(draft, dIdx) in newRows"
                :key="draft.id"
                class="border-t-2 border-dashed border-gray-200 bg-emerald-50/40"
              >

                <td class="px-2 py-2 align-top">
                   <textarea
                    v-model="draft.free_content_front"
                    rows="1"
                    class="bulk-sheet-input js-bulk-auto-text"
                    :aria-label="t('collectionCustomTextBulk.newRowFrontAria')"
                    :placeholder="t('collectionCustomTextBulk.newRowFrontPlaceholder')"
                    @input="onBulkTextInput"
                  />
                </td>

                <td class="px-2 py-2 align-top">
                   <textarea
                    v-model="draft.free_content_back"
                    rows="1"
                    class="bulk-sheet-input js-bulk-auto-text"
                    :aria-label="t('collectionCustomTextBulk.newRowBackAria')"
                    :placeholder="t('collectionCustomTextBulk.newRowBackPlaceholder')"
                    @input="onBulkTextInput"
                  />
                </td>

                <td class="px-2 py-2 align-top text-right">
                  <button
                    type="button"
                    class="inline-flex items-center justify-center rounded-md p-1.5 text-red-600 hover:bg-red-50 disabled:opacity-40"
                    :disabled="isRowActionDisabled || !canDeleteDraft(dIdx)"
                    :aria-label="t('collectionCustomTextBulk.deleteDraftAria')"
                    :title="t('collectionCustomTextBulk.deleteRow')"
                    @click="deleteDraftRow(dIdx)"
                  >
                    <Trash2 class="h-4 w-4" aria-hidden="true" />
                  </button>
                </td>

              </tr>

            </tbody>

          </table>

        </div>
         <!-- Mobile: single sheet (table-like, no gaps between rows) -->
        <div
          class="md:hidden rounded-lg border border-gray-200 bg-white shadow-sm [&>div:last-child]:border-b-0 [&>div:last-child]:rounded-b-lg"
        >
          <div
            class="sticky top-14 z-30 flex items-center rounded-t-lg border-b border-gray-200 bg-gray-100 px-2 py-2 text-left text-xs font-semibold text-gray-700 shadow-sm sm:top-12"
          >
            <div class="flex min-w-0 flex-1 gap-2">
              <span class="min-w-0 flex-1 truncate">{{ t('collectionCustomTextBulk.colFront') }}</span>
              <span class="min-w-0 flex-1 truncate">{{ t('collectionCustomTextBulk.colBack') }}</span>
            </div>
            <span class="sr-only">{{ t('collectionCustomTextBulk.colActions') }}</span>
          </div>

          <div
            v-for="(row, idx) in rows"
            :key="row.item_id"
            class="flex items-start border-b border-gray-200 px-2 py-2"
          >
            <div class="flex min-w-0 flex-1 gap-2">
              <textarea
                v-model="row.free_content_front"
                rows="1"
                class="bulk-sheet-input js-bulk-auto-text min-w-0 flex-1"
                :aria-label="t('collectionCustomTextBulk.colFront')"
                @input="onBulkTextInput"
              />
              <textarea
                v-model="row.free_content_back"
                rows="1"
                class="bulk-sheet-input js-bulk-auto-text min-w-0 flex-1"
                :aria-label="t('collectionCustomTextBulk.colBack')"
                @input="onBulkTextInput"
              />
            </div>
            <div class="flex min-h-[2.75rem] shrink-0 items-start pl-1 pt-0.5">
              <button
                type="button"
                class="inline-flex shrink-0 items-center justify-center rounded p-0.5 text-red-600 hover:bg-red-50 disabled:opacity-40"
                :disabled="isRowActionDisabled"
                :aria-label="t('collectionCustomTextBulk.deleteRowAria')"
                :title="t('collectionCustomTextBulk.deleteRow')"
                @click="deleteSavedRow(idx)"
              >
                <Loader2
                  v-if="deletingItemId === row.item_id"
                  class="h-3.5 w-3.5 animate-spin"
                  aria-hidden="true"
                />
                <Trash2 v-else class="h-3.5 w-3.5" aria-hidden="true" />
              </button>
            </div>
          </div>

          <div
            v-for="(draft, dIdx) in newRows"
            :key="draft.id"
            class="flex items-start border-b border-gray-200 bg-emerald-50/50 px-2 py-2"
          >
            <div class="flex min-w-0 flex-1 gap-2">
              <textarea
                v-model="draft.free_content_front"
                rows="1"
                class="bulk-sheet-input js-bulk-auto-text min-w-0 flex-1"
                :aria-label="
                  newRows.length > 1
                    ? `${t('collectionCustomTextBulk.newRowSectionTitle')} (${dIdx + 1}) — ${t('collectionCustomTextBulk.colFront')}`
                    : `${t('collectionCustomTextBulk.newRowSectionTitle')}: ${t('collectionCustomTextBulk.colFront')}`
                "
                :placeholder="t('collectionCustomTextBulk.newRowFrontPlaceholder')"
                @input="onBulkTextInput"
              />
              <textarea
                v-model="draft.free_content_back"
                rows="1"
                class="bulk-sheet-input js-bulk-auto-text min-w-0 flex-1"
                :aria-label="
                  newRows.length > 1
                    ? `${t('collectionCustomTextBulk.newRowSectionTitle')} (${dIdx + 1}) — ${t('collectionCustomTextBulk.colBack')}`
                    : `${t('collectionCustomTextBulk.newRowSectionTitle')}: ${t('collectionCustomTextBulk.colBack')}`
                "
                :placeholder="t('collectionCustomTextBulk.newRowBackPlaceholder')"
                @input="onBulkTextInput"
              />
            </div>
            <div class="flex min-h-[2.75rem] shrink-0 items-start pl-1 pt-0.5">
              <button
                type="button"
                class="inline-flex shrink-0 items-center justify-center rounded p-0.5 text-red-600 hover:bg-red-50 disabled:opacity-40"
                :disabled="isRowActionDisabled || !canDeleteDraft(dIdx)"
                :aria-label="t('collectionCustomTextBulk.deleteDraftAria')"
                :title="t('collectionCustomTextBulk.deleteRow')"
                @click="deleteDraftRow(dIdx)"
              >
                <Trash2 class="h-3.5 w-3.5" aria-hidden="true" />
              </button>
            </div>
          </div>
        </div>

      </div>
       </template
    > <!-- Save bar: sticky on mobile -->
    <div
      v-if="isOwner && !isLoading"
      class="fixed bottom-6 left-0 right-0 z-20 max-md:pr-20 md:static md:mt-6 p-4 bg-white/95 md:bg-transparent border-t md:border-t-0 border-gray-200 backdrop-blur-sm flex justify-end gap-3 safe-area-pb"
    >
       <button type="button" class="btn-cancel" :disabled="isSaving || !isDirty" @click="resetRows">
         {{ t('collectionCustomTextBulk.revert') }} </button
      > <button
        type="button"
        class="btn-update min-w-[10rem]"
        :disabled="isSaving || !isDirty"
        :aria-busy="isSaving"
        @click="saveAll"
      >
         <span class="inline-flex h-5 w-5 shrink-0 items-center justify-center" aria-hidden="true"
          > <Loader2 v-if="isSaving" class="h-5 w-5 animate-spin" /> <Save v-else class="h-5 w-5" />
          </span
        > {{
          isSaving ? t('collectionCustomTextBulk.saving') : t('collectionCustomTextBulk.saveAll')
        }} </button
      >
    </div>

  </div>

</template>

<script setup lang="ts">
import { ArrowLeft, FileDown, FileUp, List, Loader2, Save, Trash2 } from 'lucide-vue-next'
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  addCollectionItem,
  bulkUpdateCustomTextItems,
  getCollection,
  listCustomTextBulkItems,
  removeCollectionItem,
} from '@/api'
import FileDropzone from '@/components/FileDropzone.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import { Dropdown } from '@packages/ui'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSuccessToast } from '@/composables/useSuccessToast'
import { useSeoHead } from '@/composables/useSeoHead'
import { isLikelyCsvOrTsvFile } from '@/utils/acceptCsvTsvFile'
import {
  buildBulkCustomTextCsv,
  buildBulkCustomTextTsv,
  downloadTextFile,
} from '@/utils/exportBulkCustomText'
import { mergeBulkImportRows, parseCsvOrTsvFile } from '@/utils/parseDelimitedImport'

const { t } = useI18n()
const router = useRouter()
const auth = useAuth()
const { showError, clearError } = useError()
const { showSuccess } = useSuccessToast()

const props = defineProps({
  collectionId: {
    type: [String, Number],
    required: true,
  },
})

const numericCollectionId = computed(() => Number(props.collectionId))

function createEmptyDraft() {
  return {
    id:
      typeof crypto !== 'undefined' && crypto.randomUUID
        ? crypto.randomUUID()
        : `draft-${Date.now()}-${Math.random().toString(36).slice(2)}`,
    free_content_front: '',
    free_content_back: '',
  }
}

const collection = ref(null)
const isLoading = ref(true)
const rows = ref([])
/** Draft rows for new items; last row is always kept empty once the previous row has content. */
const newRows = ref([createEmptyDraft()])
const snapshotJson = ref('')
const isSaving = ref(false)
const isImporting = ref(false)
const deletingItemId = ref<number | null>(null)
const showImportModal = ref(false)

const MAX_IMPORT_ROWS = 2000

const isOwner = computed(() => collection.value?.owner?.username === auth.state.username)

const isRowActionDisabled = computed(
  () => isSaving.value || isImporting.value || deletingItemId.value !== null
)

const hasNewRowContent = computed(() =>
  newRows.value.some((r) => r.free_content_front.trim() !== '' || r.free_content_back.trim() !== '')
)

/** When the last draft row gets any text, append another empty draft so the user can keep adding. */
function ensureTrailingEmptyDraft() {
  const list = newRows.value
  if (list.length === 0) {
    newRows.value = [createEmptyDraft()]
    return
  }
  const last = list[list.length - 1]
  const lastIsEmpty = !last.free_content_front.trim() && !last.free_content_back.trim()
  if (!lastIsEmpty) {
    newRows.value = [...list, createEmptyDraft()]
  }
}

watch(newRows, ensureTrailingEmptyDraft, { deep: true })

/** Grow height with content; cap at max-height (10rem) then scroll inside. */
function fitTextareaHeight(el) {
  if (!(el instanceof HTMLTextAreaElement)) return
  el.style.height = 'auto'
  const cs = getComputedStyle(el)
  const maxPx =
    cs.maxHeight && cs.maxHeight !== 'none' && !Number.isNaN(parseFloat(cs.maxHeight))
      ? parseFloat(cs.maxHeight)
      : 160
  const minPx =
    cs.minHeight && cs.minHeight !== '0px' && !Number.isNaN(parseFloat(cs.minHeight))
      ? parseFloat(cs.minHeight)
      : 44
  // scrollHeight can be too small before layout; never below one comfortable line + padding
  const target = Math.max(minPx, Math.min(el.scrollHeight, maxPx))
  el.style.height = `${target}px`
}

function onBulkTextInput(e) {
  const el = e.target
  if (el instanceof HTMLTextAreaElement) fitTextareaHeight(el)
}

function canDeleteDraft(dIdx: number) {
  const list = newRows.value
  if (list.length > 1) return true
  const d = list[dIdx]
  if (!d) return false
  return d.free_content_front.trim() !== '' || d.free_content_back.trim() !== ''
}

async function deleteSavedRow(idx: number) {
  const row = rows.value[idx] as { item_id: number } | undefined
  if (!row || deletingItemId.value !== null) return
  if (!window.confirm(t('collectionCustomTextBulk.deleteSavedConfirm'))) return
  deletingItemId.value = row.item_id
  clearError()
  try {
    await removeCollectionItem(props.collectionId, row.item_id)
    rows.value = rows.value.filter((_, i) => i !== idx)
    captureSnapshot()
    await nextTick()
    refitAllBulkTextareas()
  } catch (e: unknown) {
    const ax = e as { response?: { data?: { error?: string } }; message?: string }
    showError(ax.response?.data?.error || ax.message || t('collectionCustomTextBulk.deleteError'))
  } finally {
    deletingItemId.value = null
  }
}

function deleteDraftRow(dIdx: number) {
  if (isRowActionDisabled.value || !canDeleteDraft(dIdx)) return
  const draft = newRows.value[dIdx]
  if (!draft) return
  const hasContent =
    draft.free_content_front.trim() !== '' || draft.free_content_back.trim() !== ''
  if (hasContent && !window.confirm(t('collectionCustomTextBulk.deleteDraftConfirm'))) return
  newRows.value = newRows.value.filter((_, i) => i !== dIdx)
}

function refitAllBulkTextareas() {
  if (typeof document === 'undefined') return
  requestAnimationFrame(() => {
    document.querySelectorAll('textarea.js-bulk-auto-text').forEach((el) => {
      fitTextareaHeight(el)
    })
  })
}

watch(
  [rows, newRows, isLoading],
  async () => {
    if (isLoading.value) return
    await nextTick()
    refitAllBulkTextareas()
  },
  { deep: true }
)

const isDirty = computed(() => {
  const rowsChanged = snapshotJson.value !== '' && JSON.stringify(rows.value) !== snapshotJson.value
  return rowsChanged || hasNewRowContent.value
})

function captureSnapshot() {
  snapshotJson.value = JSON.stringify(rows.value)
}

function resetRows() {
  if (!snapshotJson.value) return
  try {
    rows.value = JSON.parse(snapshotJson.value)
  } catch {
    /* ignore */
  }
  newRows.value = [createEmptyDraft()]
}

async function load(silent = false) {
  if (!silent) {
    isLoading.value = true
  }
  clearError()
  try {
    const colRes = await getCollection(props.collectionId)
    collection.value = colRes.data

    const isPublic = collection.value?.is_public
    const ownerName = collection.value?.owner?.username
    if (!isPublic && (!auth.state.isLoggedIn || ownerName !== auth.state.username)) {
      router.push('/collections')
      return
    }

    if (!auth.state.isLoggedIn || ownerName !== auth.state.username) {
      rows.value = []
      newRows.value = [createEmptyDraft()]
      captureSnapshot()
      return
    }

    const bulkRes = await listCustomTextBulkItems(props.collectionId)
    rows.value = (bulkRes.data.items || []).map((r) => ({
      item_id: r.item_id,
      position: r.position,
      free_content_front: r.free_content_front ?? '',
      free_content_back: r.free_content_back ?? '',
    }))
    newRows.value = [createEmptyDraft()]
    captureSnapshot()
  } catch (e) {
    showError(e.response?.data?.error || e.message || 'Failed to load')
  } finally {
    if (!silent) {
      isLoading.value = false
    }
  }
}

async function saveAll() {
  if (!isDirty.value || isSaving.value) return

  const draftsToAdd = newRows.value.filter(
    (r) => r.free_content_front.trim() !== '' || r.free_content_back.trim() !== ''
  )

  isSaving.value = true
  clearError()
  try {
    if (rows.value.length > 0) {
      const payload = {
        items: rows.value.map((r) => ({
          item_id: r.item_id,
          free_content_front: r.free_content_front.trim(),
          free_content_back: r.free_content_back.trim(),
        })),
      }
      await bulkUpdateCustomTextItems(props.collectionId, payload)
    }

    for (const d of draftsToAdd) {
      await addCollectionItem(props.collectionId, {
        free_content_front: d.free_content_front.trim(),
        free_content_back: d.free_content_back.trim(),
      })
    }

    const updatedCount = rows.value.length + draftsToAdd.length
    showSuccess(
      t('collectionCustomTextBulk.saveSuccess', {
        count: updatedCount,
      })
    )

    await load(true)
  } catch (e) {
    showError(e.response?.data?.error || e.message || t('collectionCustomTextBulk.saveError'))
  } finally {
    isSaving.value = false
  }
}

function onImportFileInvalid() {
  showError(t('collectionCustomTextBulk.importWrongFileType'))
}

async function onImportFileSelected(file: File) {
  await processImportFile(file)
}

function getExportableRows(): { free_content_front: string; free_content_back: string }[] {
  const drafts = newRows.value.filter(
    (r) => r.free_content_front.trim() !== '' || r.free_content_back.trim() !== ''
  )
  const saved = rows.value.map((r) => ({
    free_content_front: r.free_content_front,
    free_content_back: r.free_content_back,
  }))
  return [...saved, ...drafts]
}

function exportListAsCsv() {
  const data = getExportableRows()
  const csv = buildBulkCustomTextCsv(data)
  downloadTextFile(
    `collection-${numericCollectionId.value}-custom-text.csv`,
    csv,
    'text/csv'
  )
}

function exportListAsTsv() {
  const data = getExportableRows()
  const tsv = buildBulkCustomTextTsv(data)
  downloadTextFile(
    `collection-${numericCollectionId.value}-custom-text.tsv`,
    tsv,
    'text/tab-separated-values'
  )
}

async function processImportFile(file: File) {
  if (!file) return
  isImporting.value = true
  clearError()
  try {
    const text = await file.text()
    const { rows: parsed } = parseCsvOrTsvFile(text, file.name)
    if (parsed.length === 0) {
      showError(t('collectionCustomTextBulk.importEmpty'))
      return
    }
    if (parsed.length > MAX_IMPORT_ROWS) {
      showError(t('collectionCustomTextBulk.importTooMany', { max: MAX_IMPORT_ROWS }))
      return
    }
    const { rows: mergedRows, newRows: mergedDrafts, stats } = mergeBulkImportRows(
      parsed,
      rows.value,
      newRows.value,
      createEmptyDraft
    )
    rows.value = mergedRows
    newRows.value = mergedDrafts
    showSuccess(
      t('collectionCustomTextBulk.importSuccess', {
        front: stats.replacedByFront,
        back: stats.replacedByBack,
        inserted: stats.inserted,
        skipped: stats.skippedEmpty,
      })
    )
    showImportModal.value = false
    await nextTick()
    refitAllBulkTextareas()
  } catch (err: unknown) {
    const ax = err as { message?: string }
    showError(ax.message || t('collectionCustomTextBulk.importError'))
  } finally {
    isImporting.value = false
  }
}

onMounted(load)

watch(
  () => props.collectionId,
  () => load()
)

useSeoHead({ title: computed(() => t('collectionCustomTextBulk.documentTitle')) })
</script>

<style scoped>
.safe-area-pb {
  padding-bottom: max(1rem, env(safe-area-inset-bottom));
}

/* Input-like multiline fields: auto height via script, hard cap 10rem */
.bulk-sheet-input {
  @apply w-full px-3 py-1.5 text-sm text-gray-700 bg-white border border-gray-300 rounded-md;
  @apply placeholder-gray-400 shadow-inner shadow-slate-200 resize-none overflow-y-auto;
  @apply focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500;
  max-height: 10rem;
  /* At least one line + vertical padding + border (scrollHeight alone can be ~20px otherwise) */
  min-height: 2.75rem;
  line-height: 1.375;
  box-sizing: border-box;
}
</style>

