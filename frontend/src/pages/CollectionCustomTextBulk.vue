<template>

  <div class="flex min-h-0 flex-1 flex-col pb-3">

    <div class="mb-4 flex w-full flex-col gap-3">

      <!-- Title row: collection name + export menu only -->
      <div class="flex w-full flex-row flex-nowrap items-start justify-between gap-3 sm:gap-4">

        <div class="min-w-0 w-full flex-1">

          <div class="flex items-center gap-2 text-gray-500 italic text-sm mb-1">
            <List class="w-5 h-5 shrink-0" aria-hidden="true" /> <span>{{
              t('collectionCustomTextBulk.pageHint')
              }}</span>
          </div>

          <h1 class="text-xl sm:text-2xl font-bold text-gray-800">
            {{ collection?.name || '…' }}
          </h1>

        </div>
        <div v-if="!isLoading && isOwner" class="flex shrink-0 justify-end">
          <Dropdown :trigger-label="t('collectionCustomTextBulk.exportMenuLabel')">
            <button type="button"
              class="w-full px-4 py-2 text-left text-sm text-cyan-600 hover:bg-cyan-50 flex items-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed"
              :disabled="isSaving || isImporting" @click="exportListAsCsv">
              <FileUp class="h-4 w-4 shrink-0" aria-hidden="true" />
              {{ t('collectionCustomTextBulk.exportCsv') }}
            </button>
            <button type="button"
              class="w-full px-4 py-2 text-left text-sm text-emerald-600 hover:bg-emerald-50 flex items-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed"
              :disabled="isSaving || isImporting" @click="exportListAsTsv">
              <FileUp class="h-4 w-4 shrink-0" aria-hidden="true" />
              {{ t('collectionCustomTextBulk.exportTsv') }}
            </button>
          </Dropdown>
        </div>
      </div>

      <!-- Actions: back, import, revert, save -->
      <div class="flex w-full flex-row flex-wrap items-center gap-2 sm:gap-3">
        <div class="flex flex-row items-center gap-0" role="group">
          <RouterLink :to="`/collections/${numericCollectionId}`"
            class="btn-aqua-zinc btn-aqua-group-item">
            <ArrowLeft class="w-4 h-4 shrink-0" aria-hidden="true" />
            {{ t('collectionCustomTextBulk.backToCollection') }}
          </RouterLink>
          <button v-if="!isLoading && isOwner" type="button"
            class="btn-aqua-emerald btn-aqua-group-item"
            :disabled="isSaving || isImporting" @click="showImportModal = true">
            <FileDown class="h-4 w-4 shrink-0" aria-hidden="true" />
            {{ isImporting ? t('collectionCustomTextBulk.importing') : t('collectionCustomTextBulk.importButton') }}
          </button>
        </div>
        <div v-if="!isLoading && isOwner" class="flex flex-row flex-wrap items-center gap-0" role="group">
          <button type="button"
            class="btn-aqua-zinc btn-aqua-group-item inline-flex items-center gap-2"
            :disabled="isSaving || !isDirty" @click="resetRows">
            <Undo2 class="h-5 w-5 shrink-0" aria-hidden="true" />
            {{ t('collectionCustomTextBulk.revert') }}
          </button>
          <button type="button"
            class="btn-aqua-teal btn-aqua-group-item inline-flex items-center gap-2"
            :disabled="isSaving || !isDirty" :aria-busy="isSaving" @click="saveAll">
            <span class="inline-flex h-5 w-5 shrink-0 items-center justify-center" aria-hidden="true">
              <Loader2 v-if="isSaving" class="h-5 w-5 animate-spin" />
              <SaveChangesIcon v-else class="h-5 w-5" />
            </span>
            {{ isSaving ? t('collectionCustomTextBulk.saving') : t('collectionCustomTextBulk.saveAll') }}
          </button>
        </div>
      </div>

      <p class="w-full text-sm text-amber-800 bg-amber-50 border border-amber-200 rounded-md px-3 py-2">
        {{ t('collectionCustomTextBulk.disclaimer') }}
      </p>

    </div>
    <LoadingSpinner v-if="isLoading" class="py-12" />
    <div v-else-if="!isOwner" class="text-center py-12 text-gray-600">
      {{ t('collectionCustomTextBulk.ownerOnly') }}
    </div>
    <template v-else>
      <ModalComponent :show="showImportModal" :title="t('collectionCustomTextBulk.importButton')"
        @close="showImportModal = false">
        <p class="mb-4 text-sm text-gray-600">
          {{ t('collectionCustomTextBulk.importHint') }}
        </p>
        <FileDropzone accept=".csv,.tsv,text/csv,text/tab-separated-values"
          :choose-file-text="t('fileDropzone.chooseFile')" :or-drag-drop-text="t('fileDropzone.orDragDrop')"
          :types-note-text="t('fileDropzone.acceptsCsvTsv')" :dropzone-aria-label="t('fileDropzone.ariaLabel')"
          :input-aria-label="t('collectionCustomTextBulk.importAria')" :disabled="isSaving || isImporting"
          :validate-file="isLikelyCsvOrTsvFile" @select="onImportFileSelected" @reject="onImportFileInvalid" />
        <div v-if="isImporting && importProgress" class="mt-4 space-y-2" role="status"
          :aria-label="t('collectionCustomTextBulk.importProgressAria')">
          <div class="h-2 rounded-full bg-gray-200 overflow-hidden">
            <div class="h-full bg-cyan-600 transition-[width] duration-150 ease-out"
              :style="{ width: `${importReadPct}%` }" />
          </div>
          <p class="text-xs text-gray-600">
            {{ t('collectionCustomTextBulk.importProgressLine', importProgressParams) }}
          </p>
        </div>
      </ModalComponent>

      <DeleteConfirmationModal
        :show="pendingDelete !== null"
        :title="deleteModalTitle"
        :message="deleteModalMessage"
        :is-deleting="deleteModalIsDeleting"
        class="z-[65]"
        @confirm="confirmPendingDelete"
        @cancel="cancelPendingDelete"
      />

      <div class="flex flex-col gap-4 flex-1 min-h-0">

        <div class="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center sm:justify-between">
          <div class="flex w-full flex-row flex-nowrap items-center gap-2 sm:gap-3">
            <div class="relative flex-1 min-w-0 sm:max-w-md">
              <SearchInput
                v-model="bulkFilterQuery"
                :placeholder="t('collectionCustomTextBulk.filterPlaceholder')"
                show-search-icon
                @clear="bulkFilterQuery = ''"
              />
            </div>
            <p v-if="rows.length === 0 && !hasNewRowContent" class="shrink-0 whitespace-nowrap text-sm text-gray-600">
              {{ t('collectionCustomTextBulk.empty') }}
            </p>
            <p
              v-else-if="isBulkFilterActive && filteredScreenRowCount === 0"
              class="shrink-0 whitespace-nowrap text-sm text-gray-600"
            >
              {{ t('collectionCustomTextBulk.filterNoMatches') }}
            </p>
            <p v-else class="shrink-0 whitespace-nowrap text-sm text-gray-600">
              <template v-if="isBulkFilterActive">
                {{
                  t('collectionCustomTextBulk.filterShowing', {
                    visible: filteredScreenRowCount,
                    total: screenRowCount,
                  })
                }}
              </template>
              <template v-else>
                {{
                  rows.length === screenRowCount
                    ? t('collectionCustomTextBulk.rowCountSavedOnly', { saved: rows.length })
                    : t('collectionCustomTextBulk.rowCountSavedAndScreen', {
                        saved: rows.length,
                        screen: screenRowCount,
                      })
                }}
              </template>
            </p>
          </div>
        </div>
        <!-- Virtualized sheet (desktop + mobile share one list; only visible rows mount). -->
        <div
          class="flex min-h-0 flex-col overflow-hidden rounded-lg border border-gray-300 bg-white shadow-sm"
        >
          <div
            ref="bulkScrollParentRef"
            class="bulk-scroll-viewport min-h-0 max-h-[70vh] overflow-auto overflow-x-auto"
          >
            <div
              class="sticky top-0 z-30 grid grid-cols-[minmax(0,1fr)_auto] grid-rows-2 items-center gap-x-2 gap-y-1 border-b border-gray-300 bg-gray-100 px-2 py-2 text-left text-xs font-semibold text-gray-700 md:hidden"
            >
              <span class="min-w-0 truncate">{{ t('collectionCustomTextBulk.colFront') }}</span>
              <span class="row-span-2 whitespace-nowrap justify-self-end text-right">
                {{ t('collectionCustomTextBulk.colActions') }}
              </span>
              <span class="min-w-0 truncate">{{ t('collectionCustomTextBulk.colBack') }}</span>
            </div>
            <div
              class="sticky top-0 z-20 hidden min-w-full grid-cols-[minmax(12rem,1fr)_minmax(12rem,1fr)_6rem] border-b border-gray-300 bg-gray-100 px-2 py-2 text-left text-sm font-semibold text-gray-700 md:grid md:px-0"
            >
              <div class="min-w-[12rem] border-r border-gray-300 pl-2 pr-2">{{ t('collectionCustomTextBulk.colFront') }}</div>
              <div class="min-w-[12rem] border-r border-gray-300 pl-2 pr-2">{{ t('collectionCustomTextBulk.colBack') }}</div>
              <div class="w-24 whitespace-nowrap pl-2 pr-2 text-right">{{ t('collectionCustomTextBulk.colActions') }}</div>
            </div>
            <div
              class="relative w-full"
              :style="{ height: `${bulkTotalSize}px` }"
            >
              <div
                v-for="virtualRow in bulkVirtualItems"
                :key="String(virtualRow.key)"
                :ref="bulkVirtualizer.measureElement"
                :data-index="virtualRow.index"
                class="absolute left-0 top-0 w-full"
                :style="{ transform: `translateY(${virtualRow.start}px)` }"
              >
                <CollectionCustomTextBulkRow
                  v-if="bulkVirtualRows[virtualRow.index]"
                  :entry="bulkVirtualRows[virtualRow.index]"
                  :is-last="virtualRow.index === bulkVirtualRows.length - 1"
                  :show-draft-section-top="isFirstDraftAfterSaved(virtualRow.index)"
                  :new-row-count="newRows.length"
                  :is-row-action-disabled="isRowActionDisabled"
                  :deleting-item-id="deletingItemId"
                  :can-delete-draft="canDeleteDraft"
                  @input="onBulkTextInput"
                  @delete-saved="requestDeleteSaved"
                  @delete-draft="requestDeleteDraft"
                />
              </div>
            </div>
          </div>
        </div>

      </div>
    </template>

  </div>

</template>

<script setup lang="ts">
import { ArrowLeft, FileDown, FileUp, List, Loader2, Undo2 } from 'lucide-vue-next'
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  bulkUpdateCustomTextItems,
  getCollection,
  listCustomTextBulkItems,
  removeCollectionItem,
} from '@/api'
import SaveChangesIcon from '@/components/icons/SaveChangesIcon.vue'
import FileDropzone from '@/components/FileDropzone.vue'
import CollectionCustomTextBulkRow, {
  type BulkVirtualRow,
} from '@/components/CollectionCustomTextBulkRow.vue'
import DeleteConfirmationModal from '@/components/DeleteConfirmation.vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import SearchInput from '@/components/SearchInput.vue'
import SuccessToastImportReloadHint from '@/components/SuccessToastImportReloadHint.vue'
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
import {
  eachBulkImportRowFromFile,
  finalizeBulkImportDraftRows,
  mergeOneBulkImportRow,
  stripTrailingEmptyDrafts,
  type BulkMergeStats,
} from '@/utils/parseDelimitedImport'
import { measureElement, useVirtualizer } from '@tanstack/vue-virtual'

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
/** Pending delete shown in DeleteConfirmationModal (saved row or draft row). */
type PendingDelete =
  | { kind: 'saved'; idx: number }
  | { kind: 'draft'; dIdx: number; isEmpty: boolean }
const pendingDelete = ref<PendingDelete | null>(null)
const showImportModal = ref(false)
/** Shown while streaming file read + row merge (avoids loading the whole file as one string). */
const importProgress = ref<{
  bytesRead: number
  totalBytes: number
  rowsMerged: number
} | null>(null)

const importReadPct = computed(() => {
  const p = importProgress.value
  if (!p || p.totalBytes <= 0) return 0
  return Math.min(100, Math.round((p.bytesRead / p.totalBytes) * 100))
})

const importProgressParams = computed(() => {
  const p = importProgress.value
  if (!p) return { pct: 0, rows: 0 }
  return { pct: importReadPct.value, rows: p.rowsMerged }
})

const MAX_IMPORT_ROWS = 2000
/** Must match `MAX_CUSTOM_TEXT_BULK_ITEMS` in `src/collections/service.rs` (bulk PUT body limit). */
const MAX_CUSTOM_TEXT_BULK_ITEMS = 500

function chunkForBulkSave<T>(arr: T[], chunkSize: number): T[][] {
  const out: T[][] = []
  for (let i = 0; i < arr.length; i += chunkSize) {
    out.push(arr.slice(i, i + chunkSize))
  }
  return out
}

const isOwner = computed(() => collection.value?.owner?.username === auth.state.username)

const isRowActionDisabled = computed(
  () => isSaving.value || isImporting.value || deletingItemId.value !== null
)

const deleteModalTitle = computed(() => {
  const p = pendingDelete.value
  if (!p) return ''
  if (p.kind === 'saved') return t('collectionCustomTextBulk.deleteSavedConfirmTitle')
  return t('collectionCustomTextBulk.deleteDraftConfirmTitle')
})

const deleteModalMessage = computed(() => {
  const p = pendingDelete.value
  if (!p) return ''
  if (p.kind === 'saved') return t('collectionCustomTextBulk.deleteSavedConfirm')
  if (p.isEmpty) return t('collectionCustomTextBulk.deleteDraftEmptyConfirm')
  return t('collectionCustomTextBulk.deleteDraftConfirm')
})

const deleteModalIsDeleting = computed(
  () => pendingDelete.value?.kind === 'saved' && deletingItemId.value !== null
)

const hasNewRowContent = computed(() =>
  newRows.value.some((r) => r.free_content_front.trim() !== '' || r.free_content_back.trim() !== '')
)

/** Saved rows plus unsaved drafts that have content (empty placeholder row is not counted). */
const screenRowCount = computed(() => {
  const draftWithContent = newRows.value.filter(
    (r) => r.free_content_front.trim() !== '' || r.free_content_back.trim() !== ''
  ).length
  return rows.value.length + draftWithContent
})

const bulkFilterQuery = ref('')

function bulkRowMatchesFilter(front: string, back: string, q: string): boolean {
  const needle = q.trim().toLowerCase()
  if (!needle) return true
  const hay = `${front ?? ''}\n${back ?? ''}`.toLowerCase()
  return hay.includes(needle)
}

const isBulkFilterActive = computed(() => bulkFilterQuery.value.trim() !== '')

/** Saved rows visible under the current filter (indices are positions in `rows`). */
const filteredSavedRowsForDisplay = computed(() => {
  const q = bulkFilterQuery.value
  return rows.value
    .map((row, idx) => ({ row, idx }))
    .filter(({ row }) => bulkRowMatchesFilter(row.free_content_front, row.free_content_back, q))
})

/** Draft rows visible under the current filter (indices are positions in `newRows`). */
const filteredDraftRowsForDisplay = computed(() => {
  const q = bulkFilterQuery.value
  return newRows.value
    .map((draft, dIdx) => ({ draft, dIdx }))
    .filter(({ draft }) => bulkRowMatchesFilter(draft.free_content_front, draft.free_content_back, q))
})

const filteredScreenRowCount = computed(
  () => filteredSavedRowsForDisplay.value.length + filteredDraftRowsForDisplay.value.length
)

/** Single list for virtualization: saved rows then drafts (same order as before). */
const bulkVirtualRows = computed((): BulkVirtualRow[] => [
  ...filteredSavedRowsForDisplay.value.map((x) => ({ kind: 'saved' as const, row: x.row, idx: x.idx })),
  ...filteredDraftRowsForDisplay.value.map((x) => ({ kind: 'draft' as const, draft: x.draft, dIdx: x.dIdx })),
])

const bulkScrollParentRef = ref<HTMLElement | null>(null)

const bulkVirtualizer = useVirtualizer(
  computed(() => ({
    count: bulkVirtualRows.value.length,
    getScrollElement: () => bulkScrollParentRef.value,
    estimateSize: () => 88,
    overscan: 12,
    getItemKey: (index: number) => {
      const item = bulkVirtualRows.value[index]
      if (!item) return index
      return item.kind === 'saved' ? `s-${item.row.item_id}` : `d-${item.draft.id}`
    },
    measureElement,
  }))
)

const bulkVirtualItems = computed(() => bulkVirtualizer.value.getVirtualItems())
const bulkTotalSize = computed(() => bulkVirtualizer.value.getTotalSize())

function isFirstDraftAfterSaved(index: number) {
  const list = bulkVirtualRows.value
  const cur = list[index]
  if (!cur || cur.kind !== 'draft') return false
  const prev = list[index - 1]
  return !prev || prev.kind === 'saved'
}

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
  el.style.minHeight = 'auto'
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
  el.style.minHeight = `${target}px`
  el.style.height = 'auto'
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

function requestDeleteSaved(idx: number) {
  const row = rows.value[idx] as { item_id: number } | undefined
  if (!row || deletingItemId.value !== null) return
  pendingDelete.value = { kind: 'saved', idx }
}

function requestDeleteDraft(dIdx: number) {
  if (isRowActionDisabled.value || !canDeleteDraft(dIdx)) return
  const draft = newRows.value[dIdx]
  if (!draft) return
  const hasContent =
    draft.free_content_front.trim() !== '' || draft.free_content_back.trim() !== ''
  pendingDelete.value = { kind: 'draft', dIdx, isEmpty: !hasContent }
}

function cancelPendingDelete() {
  if (deletingItemId.value !== null) return
  pendingDelete.value = null
}

async function confirmPendingDelete() {
  const p = pendingDelete.value
  if (!p) return
  if (p.kind === 'draft') {
    if (isRowActionDisabled.value || !canDeleteDraft(p.dIdx)) {
      pendingDelete.value = null
      return
    }
    newRows.value = newRows.value.filter((_, i) => i !== p.dIdx)
    pendingDelete.value = null
    return
  }
  await performDeleteSavedAtIndex(p.idx)
}

async function performDeleteSavedAtIndex(idx: number) {
  const row = rows.value[idx] as { item_id: number } | undefined
  if (!row || deletingItemId.value !== null) return
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
    pendingDelete.value = null
  }
}

const BULK_REFIT_CHUNK = 72
const BULK_REFIT_CHUNK_THRESHOLD = 96

function refitBulkTextareasChunked(nodes: Iterable<HTMLTextAreaElement>) {
  const arr = Array.from(nodes)
  let i = 0
  const step = () => {
    const end = Math.min(i + BULK_REFIT_CHUNK, arr.length)
    for (; i < end; i++) {
      fitTextareaHeight(arr[i])
    }
    if (i < arr.length) {
      requestAnimationFrame(step)
    }
  }
  requestAnimationFrame(step)
}

function refitAllBulkTextareas() {
  if (typeof document === 'undefined') return
  requestAnimationFrame(() => {
    const nodes = document.querySelectorAll<HTMLTextAreaElement>('textarea.js-bulk-auto-text')
    if (nodes.length >= BULK_REFIT_CHUNK_THRESHOLD) {
      refitBulkTextareasChunked(nodes)
    } else {
      nodes.forEach((el) => fitTextareaHeight(el))
    }
  })
}

watch(
  [rows, newRows, isLoading, bulkFilterQuery],
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
    const itemPayloads = rows.value.map((r) => ({
      item_id: r.item_id,
      free_content_front: r.free_content_front.trim(),
      free_content_back: r.free_content_back.trim(),
    }))
    const newPayloads = draftsToAdd.map((d) => ({
      free_content_front: d.free_content_front.trim(),
      free_content_back: d.free_content_back.trim(),
    }))
    const itemChunks = chunkForBulkSave(itemPayloads, MAX_CUSTOM_TEXT_BULK_ITEMS)
    const newChunks = chunkForBulkSave(newPayloads, MAX_CUSTOM_TEXT_BULK_ITEMS)

    for (const chunk of itemChunks) {
      await bulkUpdateCustomTextItems(props.collectionId, {
        items: chunk,
        new_items: [],
      })
    }
    for (const chunk of newChunks) {
      await bulkUpdateCustomTextItems(props.collectionId, {
        items: [],
        new_items: chunk,
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

const MERGE_UI_YIELD_EVERY = 64

async function processImportFile(file: File) {
  if (!file) return
  isImporting.value = true
  importProgress.value = { bytesRead: 0, totalBytes: file.size, rowsMerged: 0 }
  clearError()
  try {
    const mergedRows = rows.value.map((r) => ({ ...r }))
    let mergedDrafts = stripTrailingEmptyDrafts(newRows.value.map((d) => ({ ...d })))
    const stats: BulkMergeStats = {
      replacedByFront: 0,
      replacedByBack: 0,
      inserted: 0,
      skippedEmpty: 0,
    }
    let lineCount = 0
    let mergeSteps = 0

    for await (const row of eachBulkImportRowFromFile(file, (p) => {
      importProgress.value = {
        bytesRead: p.bytesRead,
        totalBytes: p.totalBytes,
        rowsMerged: lineCount,
      }
    })) {
      lineCount++
      if (lineCount > MAX_IMPORT_ROWS) {
        showError(t('collectionCustomTextBulk.importTooMany', { max: MAX_IMPORT_ROWS }))
        return
      }
      mergeOneBulkImportRow(row, mergedRows, mergedDrafts, createEmptyDraft, stats)
      mergeSteps++
      if (mergeSteps >= MERGE_UI_YIELD_EVERY) {
        mergeSteps = 0
        importProgress.value = {
          bytesRead: importProgress.value!.bytesRead,
          totalBytes: importProgress.value!.totalBytes,
          rowsMerged: lineCount,
        }
        await new Promise<void>((resolve) => {
          requestAnimationFrame(() => resolve())
        })
      }
    }

    if (lineCount === 0) {
      showError(t('collectionCustomTextBulk.importEmpty'))
      return
    }

    mergedDrafts = finalizeBulkImportDraftRows(mergedDrafts, createEmptyDraft)

    isImporting.value = false
    importProgress.value = null

    showSuccess(
      t('collectionCustomTextBulk.importSuccess', {
        front: stats.replacedByFront,
        back: stats.replacedByBack,
        inserted: stats.inserted,
        skipped: stats.skippedEmpty,
      }),
      {
        duration: 0,
        extraComponent: SuccessToastImportReloadHint,
      }
    )
    showImportModal.value = false

    await nextTick()
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => resolve())
    })

    rows.value = mergedRows
    newRows.value = mergedDrafts
  } catch (err: unknown) {
    const ax = err as { message?: string }
    showError(ax.message || t('collectionCustomTextBulk.importError'))
  } finally {
    isImporting.value = false
    importProgress.value = null
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
/* Multiline bulk sheet: `input-field` supplies brand chrome; this only fixes single-line assumptions + growth. */
:deep(.bulk-sheet-input) {
  @apply w-full resize-none overflow-y-auto;
  max-height: 10rem;
  min-height: 2.75rem;
  line-height: 1.375;
  box-sizing: border-box;
  /* input-field: h-8, rounded-full, whitespace-nowrap — not valid for auto-growing textarea */
  @apply h-auto whitespace-normal;
  /* Neutralize input-field focus ring / blue border and hover border shift (sheet stays flat while editing). */
  @apply transition-none focus:z-auto focus:ring-0 focus:border-gray-300 hover:border-gray-300;
}

/* Desktop table: flush cell, borderless field, tighter block */
:deep(.bulk-sheet-input.bulk-sheet-input--table) {
  @apply rounded-none border-0;
  min-height: 2rem;
  @apply focus:border-0 hover:border-0;
}

</style>
