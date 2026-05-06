<!-- eslint-disable vue/no-mutating-props -- live row/draft objects from parent state -->
<template>
  <div
    v-if="entry.kind === 'saved'"
    ref="rowRootRef"
    class="bulk-virtual-row flex flex-row items-start gap-2 border-b border-gray-300 px-0 py-0 hover:bg-gray-50/80 sm:grid sm:grid-cols-[minmax(12rem,1fr)_minmax(12rem,1fr)_7rem] sm:items-stretch sm:gap-0 sm:border-b sm:border-gray-300"
    :class="{ 'border-b-0': isLast }"
  >
    <div class="min-w-0 flex-1 sm:contents">
      <div class="flex h-full flex-col border-gray-200 sm:border-gray-300 p-0 sm:border-r">
        <textarea
          v-model="entry.row.free_content_front"
          rows="1"
          class="input-field bulk-sheet-input bulk-sheet-input--table js-bulk-auto-text sm:rounded-none"
          :aria-label="t('collectionCustomTextBulk.colFront')"
          @input="onTextareaInput"
        />
      </div>

      <div class="flex h-full flex-col border-gray-200 sm:border-gray-300 p-0 sm:border-r">
        <textarea
          v-model="entry.row.free_content_back"
          rows="1"
          class="input-field bulk-sheet-input bulk-sheet-input--table js-bulk-auto-text sm:rounded-none"
          :aria-label="t('collectionCustomTextBulk.colBack')"
          @input="onTextareaInput"
        />
      </div>
    </div>

    <div
      class="flex min-h-[2.75rem] shrink-0 items-start gap-1 pl-1 pt-0.5 sm:min-h-0 sm:items-center sm:justify-end sm:gap-2 sm:px-2 sm:py-0 sm:pl-2 sm:pt-0"
    >
      <input
        v-if="showBulkCheckbox"
        type="checkbox"
        class="mt-1 h-4 w-4 shrink-0 rounded border-gray-300 text-cyan-600 focus:ring-cyan-500 sm:mt-0"
        :checked="isRowSelected"
        :disabled="isRowActionDisabled"
        :aria-label="t('collectionCustomTextBulk.bulkSelectRowAria')"
        @change="emit('toggle-select')"
      />
      <button
        type="button"
        class="inline-flex items-center justify-center rounded-md p-1.5 text-red-600 hover:bg-red-50 disabled:opacity-40 sm:p-1.5"
        :disabled="isRowActionDisabled"
        :aria-label="t('collectionCustomTextBulk.deleteRowAria')"
        :title="t('collectionCustomTextBulk.deleteRow')"
        @click="emit('delete-saved', entry.idx)"
      >
        <Loader2
          v-if="deletingItemId === entry.row.item_id"
          class="h-4 w-4 animate-spin sm:h-4 sm:w-4"
          aria-hidden="true"
        />
        <Trash2 v-else class="h-3.5 w-3.5 sm:h-4 sm:w-4" aria-hidden="true" />
      </button>
    </div>
  </div>

  <div
    v-else
    ref="rowRootRef"
    class="bulk-virtual-row flex flex-row items-start gap-2 border-b-2 border-gray-400 bg-emerald-50/50 px-0 py-0 sm:grid sm:grid-cols-[minmax(12rem,1fr)_minmax(12rem,1fr)_7rem] sm:items-stretch sm:gap-0 sm:border-b sm:border-gray-300 sm:bg-emerald-50/40"
    :class="{
      'border-t-2 border-gray-400 sm:border-t sm:border-gray-300': showDraftSectionTop,
      'border-b-0': isLast,
    }"
  >
    <div class="min-w-0 flex-1 sm:contents">
      <div class="flex h-full flex-col border-gray-300 p-0 sm:border-r">
        <textarea
          v-model="entry.draft.free_content_front"
          rows="1"
          class="input-field bulk-sheet-input bulk-sheet-input--table js-bulk-auto-text sm:rounded-none"
          :aria-label="
            newRowCount > 1
              ? `${t('collectionCustomTextBulk.newRowSectionTitle')} (${entry.dIdx + 1}) — ${t('collectionCustomTextBulk.colFront')}`
              : `${t('collectionCustomTextBulk.newRowSectionTitle')}: ${t('collectionCustomTextBulk.colFront')}`
          "
          :placeholder="t('collectionCustomTextBulk.newRowFrontPlaceholder')"
          @input="onTextareaInput"
        />
      </div>

      <div class="flex h-full flex-col border-gray-300 p-0 sm:border-r">
        <textarea
          v-model="entry.draft.free_content_back"
          rows="1"
          class="input-field bulk-sheet-input bulk-sheet-input--table js-bulk-auto-text sm:rounded-none"
          :aria-label="
            newRowCount > 1
              ? `${t('collectionCustomTextBulk.newRowSectionTitle')} (${entry.dIdx + 1}) — ${t('collectionCustomTextBulk.colBack')}`
              : `${t('collectionCustomTextBulk.newRowSectionTitle')}: ${t('collectionCustomTextBulk.colBack')}`
          "
          :placeholder="t('collectionCustomTextBulk.newRowBackPlaceholder')"
          @input="onTextareaInput"
        />
      </div>
    </div>

    <div
      class="flex min-h-[2.75rem] shrink-0 items-start gap-1 pl-1 pt-0.5 sm:min-h-0 sm:items-center sm:justify-end sm:gap-2 sm:px-2 sm:py-0 sm:pl-2 sm:pt-0"
    >
      <input
        v-if="showBulkCheckbox"
        type="checkbox"
        class="mt-1 h-4 w-4 shrink-0 rounded border-gray-300 text-cyan-600 focus:ring-cyan-500 sm:mt-0"
        :checked="isRowSelected"
        :disabled="isRowActionDisabled"
        :aria-label="t('collectionCustomTextBulk.bulkSelectRowAria')"
        @change="emit('toggle-select')"
      />
      <button
        type="button"
        class="inline-flex items-center justify-center rounded-md p-1.5 text-red-600 hover:bg-red-50 disabled:opacity-40 sm:p-1.5"
        :disabled="isRowActionDisabled || !canDeleteDraft(entry.dIdx)"
        :aria-label="t('collectionCustomTextBulk.deleteDraftAria')"
        :title="t('collectionCustomTextBulk.deleteRow')"
        @click="emit('delete-draft', entry.dIdx)"
      >
        <Trash2 class="h-3.5 w-3.5 sm:h-4 sm:w-4" aria-hidden="true" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Loader2, Trash2 } from 'lucide-vue-next'
import { computed, nextTick, onMounted, onUpdated, ref } from 'vue'
import { useI18n } from 'vue-i18n'

export type BulkVirtualRow =
  | {
      kind: 'saved'
      row: { item_id: number; free_content_front: string; free_content_back: string }
      idx: number
    }
  | {
      kind: 'draft'
      draft: { id: string; free_content_front: string; free_content_back: string }
      dIdx: number
    }

const props = defineProps<{
  entry: BulkVirtualRow
  isLast: boolean
  showDraftSectionTop: boolean
  newRowCount: number
  isRowActionDisabled: boolean
  deletingItemId: number | null
  canDeleteDraft: (dIdx: number) => boolean
  showRowCheckbox?: boolean
  isRowSelected?: boolean
}>()

/** Empty placeholder drafts (no text on either side) are not selectable. */
const showBulkCheckbox = computed(
  () =>
    !!props.showRowCheckbox &&
    (props.entry.kind === 'saved' ||
      props.entry.draft.free_content_front.trim() !== '' ||
      props.entry.draft.free_content_back.trim() !== '')
)

const emit = defineEmits<{
  input: [e: Event]
  'delete-saved': [idx: number]
  'delete-draft': [dIdx: number]
  'toggle-select': []
}>()

const { t } = useI18n()
const rowRootRef = ref<HTMLElement | null>(null)

function fitTextareaHeight(el: HTMLTextAreaElement) {
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

function refitRowTextareas() {
  const root = rowRootRef.value
  if (!root) return
  root.querySelectorAll<HTMLTextAreaElement>('textarea.js-bulk-auto-text').forEach((el) => {
    fitTextareaHeight(el)
  })
}

function scheduleRowRefit() {
  nextTick(() => {
    requestAnimationFrame(() => {
      refitRowTextareas()
    })
  })
}

function onTextareaInput(e: Event) {
  const target = e.target
  if (target instanceof HTMLTextAreaElement) {
    fitTextareaHeight(target)
  }
  emit('input', e)
}

onMounted(scheduleRowRefit)
onUpdated(scheduleRowRefit)
</script>
