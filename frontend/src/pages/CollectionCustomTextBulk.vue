<template>
  <div class="flex min-h-0 flex-1 flex-col pb-3">
    <div class="page-header-shell mb-4 flex w-full flex-col gap-3">
      <!-- Title row: collection name + export menu only -->
      <div class="flex w-full flex-row flex-nowrap items-start justify-between gap-3 sm:gap-4">
        <div class="min-w-0 w-full flex-1">
          <div class="flex items-center gap-2 text-gray-500 italic text-sm mb-1">
            <List class="w-5 h-5 shrink-0" aria-hidden="true" />
            <span>{{ t('collectionCustomTextBulk.pageHint') }}</span>
          </div>

          <h1 class="text-xl sm:text-2xl font-bold text-gray-800">
            {{ collection?.name || '…' }}
          </h1>
        </div>

        <div v-if="!isLoading && isOwner" class="flex shrink-0 justify-end">
          <Dropdown :trigger-label="t('collectionCustomTextBulk.exportMenuLabel')">
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
            <button
              type="button"
              class="w-full px-4 py-2 text-left text-sm text-blue-600 hover:bg-blue-50 flex items-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed"
              :disabled="isSaving || isImporting"
              @click="showImportModal = true"
            >
              <FileDown class="h-4 w-4 shrink-0" aria-hidden="true" />
              {{
                isImporting
                  ? t('collectionCustomTextBulk.importing')
                  : t('collectionCustomTextBulk.importButton')
              }}
            </button>
            <button
              type="button"
              class="w-full px-4 py-2 text-left text-sm text-purple-600 hover:bg-purple-50 flex items-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed"
              :disabled="isSaving || isImporting || mediaBulkBusy"
              @click="showMediaBulkModal = true"
            >
              <Package class="h-4 w-4 shrink-0" aria-hidden="true" />
              {{ t('collectionCustomTextBulk.mediaBulkZipButton') }}
            </button>
          </Dropdown>
        </div>
      </div>
      <!-- Actions: back, import, revert, save -->
      <div class="flex w-full flex-row flex-wrap items-center gap-2 sm:gap-3">
        <div class="btn-group-forced flex flex-row flex-wrap items-center md:gap-y-2" role="group">
          <RouterLink
            :to="`/collections/${numericCollectionId}`"
            class="ui-btn--neutral-muted ui-btn--group-item"
          >
            <ArrowLeft class="w-4 h-4 shrink-0" aria-hidden="true" />
            {{ t('collectionCustomTextBulk.backToCollection') }}
          </RouterLink>
        </div>

        <div
          v-if="!isLoading && isOwner"
          class="btn-group-forced flex flex-row flex-wrap items-center md:gap-y-2"
          role="group"
        >
          <button
            type="button"
            class="ui-btn--neutral-muted ui-btn--group-item inline-flex items-center gap-2"
            :disabled="isSaving || !isDirty"
            @click="resetRows"
          >
            <Undo2 class="h-5 w-5 shrink-0" aria-hidden="true" />
            {{ t('collectionCustomTextBulk.revert') }}
          </button>
          <button
            type="button"
            class="ui-btn--auth-signup ui-btn--group-item inline-flex items-center gap-2"
            :disabled="isSaving || !isDirty"
            :aria-busy="isSaving"
            @click="saveAll"
          >
            <span
              class="inline-flex h-5 w-5 shrink-0 items-center justify-center"
              aria-hidden="true"
            >
              <Loader2 v-if="isSaving" class="h-5 w-5 animate-spin" />
              <SaveChangesIcon v-else class="h-5 w-5" />
            </span>
            {{
              isSaving
                ? t('collectionCustomTextBulk.saving')
                : t('collectionCustomTextBulk.saveAll')
            }}
          </button>
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
    <template v-else>
      <ModalComponent
        :show="showImportModal"
        :title="t('collectionCustomTextBulk.importButton')"
        @close="closeImportModal"
      >
        <p class="mb-4 text-sm text-gray-600">{{ t('collectionCustomTextBulk.importHint') }}</p>

        <div class="mb-4 grid grid-cols-1 gap-3 rounded-md border border-gray-200 bg-gray-50 p-3">
          <label class="inline-flex items-center gap-2 text-sm text-gray-700">
            <input
              v-model="importSkipFirstRow"
              type="checkbox"
              class="h-4 w-4 rounded border-gray-300 text-cyan-600 focus:ring-cyan-500"
            />
            <span>{{ t('collectionCustomTextBulk.importSkipFirstRowLabel') }}</span>
          </label>
          <label class="flex flex-col gap-1 text-sm text-gray-700">
            <span>{{ t('collectionCustomTextBulk.importPersistLanguageFromLabel') }}</span>
            <select v-model="importPersistLanguageFrom" class="input-field">
              <option value="front">
                {{ t('collectionCustomTextBulk.importPersistFromFront') }}
              </option>

              <option value="back">
                {{ t('collectionCustomTextBulk.importPersistFromBack') }}
              </option>
            </select>
          </label>
        </div>
        <FileDropzone
          accept=".csv,.tsv,text/csv,text/tab-separated-values"
          :choose-file-text="t('fileDropzone.chooseFile')"
          :or-drag-drop-text="t('fileDropzone.orDragDrop')"
          :types-note-text="t('fileDropzone.acceptsCsvTsv')"
          :dropzone-aria-label="t('fileDropzone.ariaLabel')"
          :input-aria-label="t('collectionCustomTextBulk.importAria')"
          :disabled="isSaving || isImporting || importPreviewLoading"
          :validate-file="isLikelyCsvOrTsvFile"
          @select="onImportFileSelected"
          @reject="onImportFileInvalid"
        />
        <p v-if="importPreviewLoading" class="mt-3 text-sm text-gray-600">
          {{ t('collectionCustomTextBulk.importPreviewLoading') }}
        </p>

        <div
          v-else-if="importAwaitingFinalConfirm && importPendingFile"
          class="mt-4 space-y-3 rounded-md border border-cyan-200 bg-cyan-50/40 p-3"
        >
          <p class="text-sm font-medium text-gray-800">
            {{ t('collectionCustomTextBulk.importFinalConfirmTitle') }}
          </p>

          <p class="text-sm text-gray-700">
            {{
              t('collectionCustomTextBulk.importFinalConfirmBody', {
                name: importPendingFile.name,
              })
            }}
          </p>

          <p class="text-xs text-gray-600">
            {{ t('collectionCustomTextBulk.importFinalConfirmHint') }}
          </p>
        </div>

        <div v-else-if="importPreviewLines && importColumnConfigs.length" class="mt-4 space-y-2">
          <div class="flex flex-wrap items-end justify-between gap-2">
            <p class="text-sm font-medium text-gray-800">
              {{ t('collectionCustomTextBulk.importPreviewTitle') }}
            </p>

            <p v-if="importPreviewTruncated" class="text-xs text-gray-500">
              {{ t('collectionCustomTextBulk.importPreviewTruncated') }}
            </p>
          </div>

          <p class="text-xs text-gray-600">
            {{ t('collectionCustomTextBulk.importColumnMappingHint') }}
          </p>

          <div class="overflow-x-auto rounded-md border border-gray-200">
            <table class="min-w-full border-collapse text-left text-xs text-gray-800">
              <thead>
                <tr>
                  <th
                    v-for="(col, colIdx) in importColumnConfigs"
                    :key="`import-col-h-${colIdx}`"
                    class="min-w-[10rem] border-b border-gray-200 bg-gray-100 p-2 align-bottom font-normal"
                  >
                    <div
                      class="mb-1 text-[0.65rem] font-semibold uppercase tracking-wide text-gray-500"
                    >
                      {{ t('collectionCustomTextBulk.importColumnHeading', { n: colIdx + 1 }) }}
                    </div>
                    <label class="mb-1 block text-[0.7rem] text-gray-600">
                      {{ t('collectionCustomTextBulk.importColumnLanguageLabel') }}
                    </label>
                    <select
                      v-model.number="col.languageId"
                      class="input-field mb-2 w-full py-1 text-xs"
                    >
                      <option :value="null">
                        {{ t('collectionCustomTextBulk.importLanguageUnset') }}
                      </option>

                      <option
                        v-for="lang in languageOptions"
                        :key="`bulk-col-lang-${colIdx}-${lang.id}`"
                        :value="lang.id"
                      >
                        {{ languageLabel(lang) }}
                      </option>
                    </select>
                    <label class="mb-1 block text-[0.7rem] text-gray-600">
                      {{ t('collectionCustomTextBulk.importColumnRoleLabel') }}
                    </label>
                    <select
                      class="input-field w-full py-1 text-xs"
                      :value="col.role"
                      @change="onImportColumnRoleChange(colIdx, $event)"
                    >
                      <option value="none">
                        {{ t('collectionCustomTextBulk.importColumnRoleNone') }}
                      </option>

                      <option value="front">
                        {{ t('collectionCustomTextBulk.importColumnRoleFront') }}
                      </option>

                      <option value="back">
                        {{ t('collectionCustomTextBulk.importColumnRoleBack') }}
                      </option>
                    </select>
                  </th>
                </tr>
              </thead>

              <tbody>
                <tr
                  v-for="(line, lineIdx) in importPreviewLines"
                  :key="`import-preview-${lineIdx}`"
                >
                  <td
                    v-for="(col, colIdx) in importColumnConfigs"
                    :key="`import-cell-${lineIdx}-${colIdx}`"
                    class="max-w-[14rem] border-b border-gray-100 p-2 align-top text-gray-700"
                  >
                    <span class="line-clamp-3 break-words">{{ line[colIdx] ?? '' }}</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <p v-if="importPendingFile && !importMappingValid" class="text-xs text-red-700">
            {{ t('collectionCustomTextBulk.importMappingInvalid') }}
          </p>
        </div>

        <div v-else-if="!importPreviewLoading" class="mt-3 text-sm text-gray-500">
          {{ t('collectionCustomTextBulk.importChooseFileFirst') }}
        </div>

        <div
          v-if="isImporting && importProgress"
          class="mt-4 space-y-2"
          role="status"
          :aria-label="t('collectionCustomTextBulk.importProgressAria')"
        >
          <div class="h-2 rounded-full bg-gray-200 overflow-hidden">
            <div
              class="h-full bg-cyan-600 transition-[width] duration-150 ease-out"
              :style="{ width: `${importReadPct}%` }"
            />
          </div>

          <p class="text-xs text-gray-600">
            {{ t('collectionCustomTextBulk.importProgressLine', importProgressParams) }}
          </p>
        </div>
        <template #footer>
          <div
            v-if="importAwaitingFinalConfirm"
            class="flex w-full flex-wrap items-center justify-end gap-2"
          >
            <button
              type="button"
              class="ui-btn--neutral-muted"
              :disabled="isImporting"
              @click="backFromImportConfirm"
            >
              {{ t('collectionCustomTextBulk.importBackToMapping') }}
            </button>
            <button
              type="button"
              class="ui-btn--create"
              :disabled="!canExecuteImport"
              :aria-busy="isImporting"
              @click="executeImport"
            >
              {{
                isImporting
                  ? t('collectionCustomTextBulk.importing')
                  : t('collectionCustomTextBulk.importConfirmRun')
              }}
            </button>
          </div>

          <div v-else class="flex w-full flex-wrap items-center justify-end gap-2">
            <button
              type="button"
              class="ui-btn--neutral-muted"
              :disabled="isImporting"
              @click="closeImportModal"
            >
              {{ t('collectionCustomTextBulk.importCancel') }}
            </button>
            <button
              type="button"
              class="ui-btn--create"
              :disabled="!canProceedImportReview"
              @click="proceedToImportConfirm"
            >
              {{ t('collectionCustomTextBulk.importContinueReview') }}
            </button>
          </div>
        </template>
      </ModalComponent>
      <CollectionMediaBulkZipModal
        v-model="showMediaBulkModal"
        :collection-id="numericCollectionId"
        @success="load(true)"
        @busy="onMediaBulkBusy"
      />
      <DeleteConfirmationModal
        :show="pendingDelete !== null"
        :title="deleteModalTitle"
        :message="deleteModalMessage"
        :is-deleting="deleteModalIsDeleting"
        class="z-[65]"
        @confirm="confirmPendingDelete"
        @cancel="cancelPendingDelete"
      />
      <DeleteConfirmationModal
        :show="bulkDeletePending"
        :title="t('collectionCustomTextBulk.bulkDeleteConfirmTitle')"
        :message="
          t('collectionCustomTextBulk.bulkDeleteConfirmMessage', { count: selectedBulkCount })
        "
        :is-deleting="bulkDeleteInProgress"
        class="z-[65]"
        @confirm="confirmBulkDelete"
        @cancel="cancelBulkDelete"
      />
      <div class="flex flex-col gap-4 flex-1 min-h-0">
        <div
          class="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center sm:justify-between"
        >
          <div class="flex w-full flex-row flex-nowrap items-center gap-2 sm:gap-3">
            <div class="relative flex-1 min-w-0 sm:max-w-md">
              <SearchInput
                v-model="bulkFilterQuery"
                :placeholder="t('collectionCustomTextBulk.filterPlaceholder')"
                show-search-icon
                @clear="bulkFilterQuery = ''"
              />
            </div>

            <div v-if="selectedBulkCount > 0" class="flex shrink-0 flex-wrap items-center gap-2">
              <button
                type="button"
                class="ui-btn--neutral-muted"
                :disabled="isRowActionDisabled || bulkDeleteInProgress"
                @click="clearBulkSelection"
              >
                {{ t('collectionCustomTextBulk.bulkClearSelection') }}
              </button>
              <button
                type="button"
                class="ui-btn--delete"
                :disabled="isRowActionDisabled || bulkDeleteInProgress"
                @click="requestBulkDelete"
              >
                {{ t('collectionCustomTextBulk.bulkDeleteSelected', { count: selectedBulkCount }) }}
              </button>
            </div>

            <p
              v-if="rows.length === 0 && !hasNewRowContent"
              class="shrink-0 whitespace-nowrap text-sm text-gray-600"
            >
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
              class="sticky top-0 z-30 grid grid-cols-[minmax(0,1fr)_auto] grid-rows-2 items-center gap-x-2 gap-y-1 border-b border-gray-300 bg-gray-100 px-2 py-2 text-left text-xs font-semibold text-gray-700 sm:hidden"
            >
              <span class="min-w-0 truncate">{{ t('collectionCustomTextBulk.colFront') }}</span>
              <span
                class="row-span-2 flex flex-col items-end justify-center gap-1 whitespace-nowrap text-right"
              >
                <span class="sr-only">{{ t('collectionCustomTextBulk.bulkSelectColumn') }}</span>
                <input
                  ref="bulkSelectAllCheckboxMobileRef"
                  type="checkbox"
                  class="h-4 w-4 rounded border-gray-300 text-cyan-600 focus:ring-cyan-500"
                  :checked="bulkSelectAllChecked"
                  :aria-label="t('collectionCustomTextBulk.bulkSelectAllAria')"
                  @change="onBulkSelectAllChange"
                />
                <span>{{ t('collectionCustomTextBulk.colActions') }}</span>
              </span>
              <span class="min-w-0 truncate">{{ t('collectionCustomTextBulk.colBack') }}</span>
            </div>

            <div
              class="sticky top-0 z-20 hidden min-w-full grid-cols-[minmax(12rem,1fr)_minmax(12rem,1fr)_7rem] border-b border-gray-300 bg-gray-100 px-2 py-2 text-left text-sm font-semibold text-gray-700 sm:grid sm:px-0"
            >
              <div class="min-w-[12rem] border-r border-gray-300 pl-2 pr-2">
                {{ t('collectionCustomTextBulk.colFront') }}
              </div>

              <div class="min-w-[12rem] border-r border-gray-300 pl-2 pr-2">
                {{ t('collectionCustomTextBulk.colBack') }}
              </div>

              <div
                class="flex w-full min-w-[7rem] items-center justify-end gap-2 whitespace-nowrap pl-2 pr-2 text-right"
              >
                <input
                  ref="bulkSelectAllCheckboxRef"
                  type="checkbox"
                  class="h-4 w-4 shrink-0 rounded border-gray-300 text-cyan-600 focus:ring-cyan-500"
                  :checked="bulkSelectAllChecked"
                  :aria-label="t('collectionCustomTextBulk.bulkSelectAllAria')"
                  @change="onBulkSelectAllChange"
                />
                <span>{{ t('collectionCustomTextBulk.colActions') }}</span>
              </div>
            </div>

            <div class="relative w-full" :style="{ height: `${bulkTotalSize}px` }">
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
                  :show-row-checkbox="true"
                  :is-row-selected="isBulkRowSelected(bulkVirtualRows[virtualRow.index])"
                  @input="onBulkTextInput"
                  @delete-saved="requestDeleteSaved"
                  @delete-draft="requestDeleteDraft"
                  @toggle-select="toggleBulkRowSelect(bulkVirtualRows[virtualRow.index])"
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
import { ArrowLeft, FileDown, FileUp, List, Loader2, Package, Undo2 } from 'lucide-vue-next'
import { computed, nextTick, onMounted, ref, watch, watchEffect } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  bulkRemoveCollectionItems,
  bulkUpdateCustomTextItems,
  getLanguages,
  getCollection,
  listCustomTextBulkItems,
  removeCollectionItem,
} from '@/api'
import SaveChangesIcon from '@/components/icons/SaveChangesIcon.vue'
import CollectionMediaBulkZipModal from '@/components/CollectionMediaBulkZipModal.vue'
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
  type BulkDraftRow,
  type BulkTableRow,
  eachBulkImportRowFromFile,
  finalizeBulkImportDraftRows,
  mergeOneBulkImportRow,
  previewBulkImportFile,
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

type LanguageOption = {
  id: number
  tag?: string
  english_name?: string
  real_name?: string
}

function createEmptyDraft(): BulkDraftRow {
  return {
    id:
      typeof crypto !== 'undefined' && crypto.randomUUID
        ? crypto.randomUUID()
        : `draft-${Date.now()}-${Math.random().toString(36).slice(2)}`,
    free_content_front: '',
    free_content_back: '',
    language_id: null as number | null,
  }
}

const collection = ref(null)
const isLoading = ref(true)
const rows = ref<BulkTableRow[]>([])
/** Draft rows for new items; last row is always kept empty once the previous row has content. */
const newRows = ref<BulkDraftRow[]>([createEmptyDraft()])
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
const showMediaBulkModal = ref(false)
const mediaBulkBusy = ref(false)

function onMediaBulkBusy(b: boolean) {
  mediaBulkBusy.value = b
}
const languageOptions = ref<LanguageOption[]>([])
/** Shown while streaming file read + row merge (avoids loading the whole file as one string). */
const importProgress = ref<{
  bytesRead: number
  totalBytes: number
  rowsMerged: number
} | null>(null)
const importSkipFirstRow = ref(false)
const importPersistLanguageFrom = ref<'front' | 'back'>('front')

type ImportColumnRole = 'none' | 'front' | 'back'
type ImportColumnConfig = { role: ImportColumnRole; languageId: number | null }

const importPendingFile = ref<File | null>(null)
const importPreviewLoading = ref(false)
const importPreviewLines = ref<string[][] | null>(null)
const importPreviewTruncated = ref(false)
const importColumnConfigs = ref<ImportColumnConfig[]>([])
/** After mapping preview, user must confirm on a second screen before merge runs. */
const importAwaitingFinalConfirm = ref(false)
const IMPORT_PREVIEW_LINE_COUNT = 25

const bulkSelectAllCheckboxRef = ref<HTMLInputElement | null>(null)
const bulkSelectAllCheckboxMobileRef = ref<HTMLInputElement | null>(null)
/** Stable keys: `s-{item_id}` saved, `d-{draft.id}` drafts */
const selectedBulkKeys = ref<string[]>([])
const bulkDeletePending = ref(false)
const bulkDeleteInProgress = ref(false)

function defaultColumnConfigs(columnCount: number): ImportColumnConfig[] {
  const n = Math.max(1, columnCount)
  return Array.from({ length: n }, (_, i) => ({
    role: (i === 0 ? 'front' : i === 1 ? 'back' : 'none') as ImportColumnRole,
    languageId: null as number | null,
  }))
}

function onImportColumnRoleChange(colIdx: number, ev: Event) {
  const el = ev.target as HTMLSelectElement | null
  const role = (el?.value || 'none') as ImportColumnRole
  const list = importColumnConfigs.value.map((c) => ({ ...c }))
  if (!list[colIdx]) return
  if (role === 'front') {
    for (let i = 0; i < list.length; i++) {
      if (i !== colIdx && list[i].role === 'front') list[i].role = 'none'
    }
  }
  if (role === 'back') {
    for (let i = 0; i < list.length; i++) {
      if (i !== colIdx && list[i].role === 'back') list[i].role = 'none'
    }
  }
  list[colIdx].role = role
  importColumnConfigs.value = list
}

const importMappingValid = computed(() => {
  const fi = importColumnConfigs.value.findIndex((c) => c.role === 'front')
  const bi = importColumnConfigs.value.findIndex((c) => c.role === 'back')
  return fi >= 0 && bi >= 0 && fi !== bi
})

const importFrontColumn = computed(() =>
  importColumnConfigs.value.findIndex((c) => c.role === 'front')
)
const importBackColumn = computed(() =>
  importColumnConfigs.value.findIndex((c) => c.role === 'back')
)

const hasImportColumnConflict = computed(() => {
  const f = importFrontColumn.value
  const b = importBackColumn.value
  return f >= 0 && b >= 0 && f === b
})

const importPersistedLanguageId = computed(() => {
  const idx =
    importPersistLanguageFrom.value === 'front' ? importFrontColumn.value : importBackColumn.value
  if (idx < 0) return null
  return importColumnConfigs.value[idx]?.languageId ?? null
})

function normalizeImportColumnIndex(i: number): number {
  return Math.max(0, i)
}

const canProceedImportReview = computed(
  () =>
    !!importPendingFile.value &&
    importMappingValid.value &&
    !hasImportColumnConflict.value &&
    !importPreviewLoading.value &&
    !isImporting.value
)

const canExecuteImport = computed(
  () =>
    !!importPendingFile.value &&
    importMappingValid.value &&
    !hasImportColumnConflict.value &&
    !importPreviewLoading.value &&
    !isImporting.value
)

function bulkVirtualRowKey(entry: BulkVirtualRow): string {
  return entry.kind === 'saved' ? `s-${entry.row.item_id}` : `d-${entry.draft.id}`
}

function isBulkRowSelectable(entry: BulkVirtualRow): boolean {
  if (entry.kind === 'saved') return true
  return entry.draft.free_content_front.trim() !== '' || entry.draft.free_content_back.trim() !== ''
}

function closeImportModal() {
  showImportModal.value = false
  importPendingFile.value = null
  importPreviewLines.value = null
  importPreviewTruncated.value = false
  importColumnConfigs.value = []
  importAwaitingFinalConfirm.value = false
  importProgress.value = null
}

function proceedToImportConfirm() {
  if (!canProceedImportReview.value) return
  importAwaitingFinalConfirm.value = true
}

function backFromImportConfirm() {
  if (isImporting.value) return
  importAwaitingFinalConfirm.value = false
}

async function executeImport() {
  const file = importPendingFile.value
  if (!file || !canExecuteImport.value) return
  await processImportFile(file)
}

function requestBulkDelete() {
  if (selectedBulkCount.value === 0 || isRowActionDisabled.value) return
  bulkDeletePending.value = true
}

function cancelBulkDelete() {
  if (bulkDeleteInProgress.value) return
  bulkDeletePending.value = false
}

async function confirmBulkDelete() {
  if (selectedBulkCount.value === 0) {
    bulkDeletePending.value = false
    return
  }
  bulkDeleteInProgress.value = true
  clearError()
  try {
    const savedIds = new Set<number>()
    const draftIds = new Set<string>()
    for (const k of selectedBulkKeys.value) {
      if (k.startsWith('s-')) savedIds.add(Number(k.slice(2)))
      else if (k.startsWith('d-')) draftIds.add(k.slice(2))
    }

    if (savedIds.size > 0) {
      const idList = [...savedIds]
      const chunks = chunkForBulkSave(idList, MAX_CUSTOM_TEXT_BULK_ITEMS)
      for (const chunk of chunks) {
        await bulkRemoveCollectionItems(props.collectionId, chunk)
        const chunkSet = new Set(chunk)
        rows.value = rows.value.filter((r) => !chunkSet.has(r.item_id))
      }
    }

    if (draftIds.size > 0) {
      newRows.value = newRows.value.filter((d) => !draftIds.has(d.id))
    }

    selectedBulkKeys.value = []
    captureSnapshot()
    await nextTick()
    refitAllBulkTextareas()
  } catch (e: unknown) {
    const ax = e as { response?: { data?: { error?: string } }; message?: string }
    showError(ax.response?.data?.error || ax.message || t('collectionCustomTextBulk.deleteError'))
  } finally {
    bulkDeleteInProgress.value = false
    bulkDeletePending.value = false
  }
}

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

function languageLabel(lang: LanguageOption): string {
  const title =
    (lang.real_name && lang.real_name.trim()) ||
    (lang.english_name && lang.english_name.trim()) ||
    `#${lang.id}`
  return lang.tag ? `${title} (${lang.tag})` : title
}

const isOwner = computed(() => collection.value?.owner?.username === auth.state.username)

const isRowActionDisabled = computed(
  () => isSaving.value || isImporting.value || mediaBulkBusy.value || deletingItemId.value !== null
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
    .filter(({ draft }) =>
      bulkRowMatchesFilter(draft.free_content_front, draft.free_content_back, q)
    )
})

const filteredScreenRowCount = computed(
  () => filteredSavedRowsForDisplay.value.length + filteredDraftRowsForDisplay.value.length
)

/** Single list for virtualization: saved rows then drafts (same order as before). */
const bulkVirtualRows = computed((): BulkVirtualRow[] => [
  ...filteredSavedRowsForDisplay.value.map((x) => ({
    kind: 'saved' as const,
    row: x.row,
    idx: x.idx,
  })),
  ...filteredDraftRowsForDisplay.value.map((x) => ({
    kind: 'draft' as const,
    draft: x.draft,
    dIdx: x.dIdx,
  })),
])

const bulkSelectableVirtualRows = computed(() => bulkVirtualRows.value.filter(isBulkRowSelectable))

const selectedBulkCount = computed(() => selectedBulkKeys.value.length)

const bulkSelectAllChecked = computed(() => {
  const list = bulkSelectableVirtualRows.value
  if (list.length === 0) return false
  return list.every((r) => selectedBulkKeys.value.includes(bulkVirtualRowKey(r)))
})

function isBulkRowSelected(entry: BulkVirtualRow): boolean {
  return selectedBulkKeys.value.includes(bulkVirtualRowKey(entry))
}

function toggleBulkRowSelect(entry: BulkVirtualRow) {
  const k = bulkVirtualRowKey(entry)
  const cur = selectedBulkKeys.value
  if (cur.includes(k)) {
    selectedBulkKeys.value = cur.filter((x) => x !== k)
  } else {
    selectedBulkKeys.value = [...cur, k]
  }
}

function onBulkSelectAllChange(ev: Event) {
  const el = ev.target as HTMLInputElement | null
  const want = !!el?.checked
  const keys = bulkSelectableVirtualRows.value.map((r) => bulkVirtualRowKey(r))
  if (keys.length === 0) return
  if (want) {
    selectedBulkKeys.value = [...new Set([...selectedBulkKeys.value, ...keys])]
  } else {
    const drop = new Set(keys)
    selectedBulkKeys.value = selectedBulkKeys.value.filter((k) => !drop.has(k))
  }
}

function clearBulkSelection() {
  selectedBulkKeys.value = []
}

watchEffect(() => {
  const list = bulkSelectableVirtualRows.value
  const sel = selectedBulkKeys.value
  let n = 0
  for (const r of list) {
    if (sel.includes(bulkVirtualRowKey(r))) n++
  }
  const ind = list.length > 0 && n > 0 && n < list.length
  for (const el of [bulkSelectAllCheckboxRef.value, bulkSelectAllCheckboxMobileRef.value]) {
    if (el) el.indeterminate = ind
  }
})

watch(
  bulkVirtualRows,
  () => {
    const allowed = new Set(
      bulkVirtualRows.value.filter(isBulkRowSelectable).map(bulkVirtualRowKey)
    )
    selectedBulkKeys.value = selectedBulkKeys.value.filter((k) => allowed.has(k))
  },
  { deep: true }
)

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
  el.style.minHeight = '0'
  el.style.height = '0'
  const cs = getComputedStyle(el)
  const maxPx =
    cs.maxHeight && cs.maxHeight !== 'none' && !Number.isNaN(parseFloat(cs.maxHeight))
      ? parseFloat(cs.maxHeight)
      : 160
  const minPx =
    cs.minHeight && cs.minHeight !== '0px' && !Number.isNaN(parseFloat(cs.minHeight))
      ? parseFloat(cs.minHeight)
      : 44
  const sh = el.scrollHeight
  const target = Math.max(minPx, Math.min(sh, maxPx))
  el.style.minHeight = `${target}px`
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

function requestDeleteSaved(idx: number) {
  const row = rows.value[idx] as { item_id: number } | undefined
  if (!row || deletingItemId.value !== null) return
  pendingDelete.value = { kind: 'saved', idx }
}

function requestDeleteDraft(dIdx: number) {
  if (isRowActionDisabled.value || !canDeleteDraft(dIdx)) return
  const draft = newRows.value[dIdx]
  if (!draft) return
  const hasContent = draft.free_content_front.trim() !== '' || draft.free_content_back.trim() !== ''
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
      languageOptions.value = []
      captureSnapshot()
      return
    }

    try {
      const langsRes = await getLanguages()
      languageOptions.value = Array.isArray(langsRes.data) ? langsRes.data : []
    } catch {
      languageOptions.value = []
    }

    const bulkRes = await listCustomTextBulkItems(props.collectionId)
    rows.value = (bulkRes.data.items || []).map((r) => ({
      item_id: r.item_id,
      position: r.position,
      free_content_front: r.free_content_front ?? '',
      free_content_back: r.free_content_back ?? '',
      language_id: r.language_id ?? null,
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
      language_id: r.language_id ?? null,
    }))
    const newPayloads = draftsToAdd.map((d) => ({
      free_content_front: d.free_content_front.trim(),
      free_content_back: d.free_content_back.trim(),
      language_id: d.language_id ?? null,
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
  importAwaitingFinalConfirm.value = false
  importPendingFile.value = file
  importPreviewLoading.value = true
  importPreviewLines.value = null
  importColumnConfigs.value = []
  clearError()
  try {
    const { lines, truncated } = await previewBulkImportFile(file, IMPORT_PREVIEW_LINE_COUNT)
    importPreviewLines.value = lines
    importPreviewTruncated.value = truncated
    const colCount = Math.max(1, lines[0]?.length ?? 2)
    importColumnConfigs.value = defaultColumnConfigs(colCount)
  } catch (e: unknown) {
    const ax = e as { message?: string }
    showError(ax.message || t('collectionCustomTextBulk.importError'))
    importPendingFile.value = null
    importPreviewLines.value = null
  } finally {
    importPreviewLoading.value = false
  }
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
  downloadTextFile(`collection-${numericCollectionId.value}-custom-text.csv`, csv, 'text/csv')
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
  if (hasImportColumnConflict.value) {
    showError(t('collectionCustomTextBulk.importColumnsMustDiffer'))
    return
  }
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

    for await (const row of eachBulkImportRowFromFile(
      file,
      (p) => {
        importProgress.value = {
          bytesRead: p.bytesRead,
          totalBytes: p.totalBytes,
          rowsMerged: lineCount,
        }
      },
      {
        frontColumnIndex: normalizeImportColumnIndex(importFrontColumn.value),
        backColumnIndex: normalizeImportColumnIndex(importBackColumn.value),
        skipFirstRow: importSkipFirstRow.value,
        languageId: importPersistedLanguageId.value ?? undefined,
      }
    )) {
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
    closeImportModal()

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

useSeoHead({
  title: computed(() => t('collectionCustomTextBulk.documentTitle')),
  robots: 'noindex, nofollow',
})
</script>

<style scoped>
/* Multiline bulk sheet: `input-field` supplies brand chrome; this only fixes single-line assumptions + growth. */
:deep(.bulk-sheet-input) {
  @apply w-full resize-none overflow-y-auto py-2 px-2;
  max-height: 10rem;
  /* At least two line boxes + py-2 so wrapped two-line text does not get an inner scrollbar. */
  min-height: calc(2.75em + 1rem);
  min-height: calc(2 * lh + 1rem);
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
  min-height: calc(2.75em + 1rem);
  min-height: calc(2 * lh + 1rem);
  @apply focus:border-0 hover:border-0;
}
</style>
