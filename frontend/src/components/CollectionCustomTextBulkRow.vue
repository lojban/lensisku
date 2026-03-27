<!-- eslint-disable vue/no-mutating-props -- live row/draft objects from parent state -->
<template>
  <div
    v-if="entry.kind === 'saved'"
    ref="rowRootRef"
    class="bulk-virtual-row flex flex-row items-start gap-2 border-b border-gray-300 px-2 py-2 hover:bg-gray-50/80 md:grid md:grid-cols-[minmax(12rem,1fr)_minmax(12rem,1fr)_6rem] md:items-stretch md:gap-0 md:px-0 md:py-0"
    :class="{ 'border-b-0': isLast }"
  >
    <div class="min-w-0 flex-1 md:contents">
      <div class="flex h-full flex-col border-gray-300 p-0 md:border-r">
        <textarea
          v-model="entry.row.free_content_front"
          rows="1"
          class="input-field bulk-sheet-input bulk-sheet-input--table js-bulk-auto-text md:rounded-none"
          :aria-label="t('collectionCustomTextBulk.colFront')"
          @input="onTextareaInput"
        />
      </div>
      <div class="flex h-full flex-col border-gray-300 p-0 md:border-r">
        <textarea
          v-model="entry.row.free_content_back"
          rows="1"
          class="input-field bulk-sheet-input bulk-sheet-input--table js-bulk-auto-text md:rounded-none"
          :aria-label="t('collectionCustomTextBulk.colBack')"
          @input="onTextareaInput"
        />
      </div>
    </div>
    <div
      class="flex min-h-[2.75rem] shrink-0 items-start pl-1 pt-0.5 md:min-h-0 md:items-center md:justify-end md:px-2 md:py-0 md:pl-2 md:pt-0"
    >
      <button
        type="button"
        class="inline-flex items-center justify-center rounded-md p-1.5 text-red-600 hover:bg-red-50 disabled:opacity-40 md:p-1.5"
        :disabled="isRowActionDisabled"
        :aria-label="t('collectionCustomTextBulk.deleteRowAria')"
        :title="t('collectionCustomTextBulk.deleteRow')"
        @click="emit('delete-saved', entry.idx)"
      >
        <Loader2
          v-if="deletingItemId === entry.row.item_id"
          class="h-4 w-4 animate-spin md:h-4 md:w-4"
          aria-hidden="true"
        />
        <Trash2 v-else class="h-3.5 w-3.5 md:h-4 md:w-4" aria-hidden="true" />
      </button>
    </div>
  </div>

  <div
    v-else
    ref="rowRootRef"
    class="bulk-virtual-row flex flex-row items-start gap-2 border-b border-gray-300 bg-emerald-50/50 px-2 py-2 md:grid md:grid-cols-[minmax(12rem,1fr)_minmax(12rem,1fr)_6rem] md:items-stretch md:gap-0 md:bg-emerald-50/40 md:px-0 md:py-0"
    :class="{
      'border-t border-gray-300': showDraftSectionTop,
      'border-b-0': isLast,
    }"
  >
    <div class="min-w-0 flex-1 md:contents">
      <div class="flex h-full flex-col border-gray-300 p-0 md:border-r">
        <textarea
          v-model="entry.draft.free_content_front"
          rows="1"
          class="input-field bulk-sheet-input bulk-sheet-input--table js-bulk-auto-text md:rounded-none"
          :aria-label="
            newRowCount > 1
              ? `${t('collectionCustomTextBulk.newRowSectionTitle')} (${entry.dIdx + 1}) — ${t('collectionCustomTextBulk.colFront')}`
              : `${t('collectionCustomTextBulk.newRowSectionTitle')}: ${t('collectionCustomTextBulk.colFront')}`
          "
          :placeholder="t('collectionCustomTextBulk.newRowFrontPlaceholder')"
          @input="onTextareaInput"
        />
      </div>
      <div class="flex h-full flex-col border-gray-300 p-0 md:border-r">
        <textarea
          v-model="entry.draft.free_content_back"
          rows="1"
          class="input-field bulk-sheet-input bulk-sheet-input--table js-bulk-auto-text md:rounded-none"
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
      class="flex min-h-[2.75rem] shrink-0 items-start pl-1 pt-0.5 md:min-h-0 md:items-center md:justify-end md:px-2 md:py-0 md:pl-2 md:pt-0"
    >
      <button
        type="button"
        class="inline-flex items-center justify-center rounded-md p-1.5 text-red-600 hover:bg-red-50 disabled:opacity-40 md:p-1.5"
        :disabled="isRowActionDisabled || !canDeleteDraft(entry.dIdx)"
        :aria-label="t('collectionCustomTextBulk.deleteDraftAria')"
        :title="t('collectionCustomTextBulk.deleteRow')"
        @click="emit('delete-draft', entry.dIdx)"
      >
        <Trash2 class="h-3.5 w-3.5 md:h-4 md:w-4" aria-hidden="true" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Loader2, Trash2 } from 'lucide-vue-next'
import { nextTick, onMounted, onUpdated, ref } from 'vue'
import { useI18n } from 'vue-i18n'

export type BulkVirtualRow =
  | { kind: 'saved'; row: { item_id: number; free_content_front: string; free_content_back: string }; idx: number }
  | { kind: 'draft'; draft: { id: string; free_content_front: string; free_content_back: string }; dIdx: number }

defineProps<{
  entry: BulkVirtualRow
  isLast: boolean
  showDraftSectionTop: boolean
  newRowCount: number
  isRowActionDisabled: boolean
  deletingItemId: number | null
  canDeleteDraft: (dIdx: number) => boolean
}>()

const emit = defineEmits<{
  input: [e: Event]
  'delete-saved': [idx: number]
  'delete-draft': [dIdx: number]
}>()

const { t } = useI18n()
const rowRootRef = ref<HTMLElement | null>(null)

function fitTextareaHeight(el: HTMLTextAreaElement) {
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
  const target = Math.max(minPx, Math.min(el.scrollHeight, maxPx))
  el.style.minHeight = `${target}px`
  el.style.height = 'auto'
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
