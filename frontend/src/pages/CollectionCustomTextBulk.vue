<template>
  <div class="flex flex-col min-h-[calc(100vh-6rem)] pb-24 md:pb-8">
    <div class="mb-4 flex w-full flex-col gap-3">
      <div class="flex w-full flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2 text-gray-500 italic text-sm mb-1">
            <List class="w-5 h-5 shrink-0" aria-hidden="true" />
            <span>{{ t('collectionCustomTextBulk.pageHint') }}</span>
          </div>
          <h1 class="text-xl sm:text-2xl font-bold text-gray-800">
            {{ collection?.name || '…' }}
          </h1>
        </div>
        <RouterLink :to="`/collections/${numericCollectionId}`" class="btn-aqua-white shrink-0 self-start">
          <ArrowLeft class="w-4 h-4" />
          {{ t('collectionCustomTextBulk.backToCollection') }}
        </RouterLink>
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
      <div class="flex flex-col gap-4 flex-1 min-h-0">
        <p v-if="rows.length === 0" class="text-sm text-gray-600">
          {{ t('collectionCustomTextBulk.empty') }}
        </p>
        <p v-else class="text-sm text-gray-600">
          {{ t('collectionCustomTextBulk.rowCount', { count: rows.length }) }}
        </p>

        <!-- Desktop: table -->
        <div class="hidden md:block overflow-x-auto border border-gray-200 rounded-lg bg-white shadow-sm">
          <table class="min-w-full text-sm border-collapse">
            <thead>
              <tr class="bg-gray-100 border-b border-gray-200 text-left">
                <th class="px-2 py-2 font-semibold text-gray-700 w-12 whitespace-nowrap">
                  #
                </th>
                <th class="px-2 py-2 font-semibold text-gray-700 min-w-[12rem]">
                  {{ t('collectionCustomTextBulk.colFront') }}
                </th>
                <th class="px-2 py-2 font-semibold text-gray-700 min-w-[12rem]">
                  {{ t('collectionCustomTextBulk.colBack') }}
                </th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="(row, idx) in rows"
                :key="row.item_id"
                class="border-b border-gray-100 hover:bg-gray-50/80"
              >
                <td class="px-2 py-1 align-top text-gray-500 tabular-nums">
                  {{ idx + 1 }}
                </td>
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
              </tr>
              <tr
                v-for="(draft, dIdx) in newRows"
                :key="draft.id"
                class="border-t-2 border-dashed border-gray-200 bg-emerald-50/40"
              >
                <td class="px-2 py-2 align-top text-emerald-700 font-medium whitespace-nowrap">
                  {{ t('collectionCustomTextBulk.newRowMarker') }}{{ newRows.length > 1 ? ` ${dIdx + 1}` : '' }}
                </td>
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
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Mobile: stacked cards -->
        <div class="md:hidden space-y-4">
          <div
            v-for="(row, idx) in rows"
            :key="row.item_id"
            class="border border-gray-200 rounded-lg p-3 bg-white shadow-sm space-y-2"
          >
            <div class="text-xs font-medium text-gray-500">
              {{ t('collectionCustomTextBulk.cardLabel', { n: idx + 1 }) }}
            </div>
            <div>
              <label class="block text-xs font-medium text-gray-600 mb-1">{{ t('collectionCustomTextBulk.colFront') }}</label>
              <textarea
                v-model="row.free_content_front"
                rows="1"
                class="bulk-sheet-input js-bulk-auto-text w-full"
                @input="onBulkTextInput"
              />
            </div>
            <div>
              <label class="block text-xs font-medium text-gray-600 mb-1">{{ t('collectionCustomTextBulk.colBack') }}</label>
              <textarea
                v-model="row.free_content_back"
                rows="1"
                class="bulk-sheet-input js-bulk-auto-text w-full"
                @input="onBulkTextInput"
              />
            </div>
          </div>

          <div
            v-for="(draft, dIdx) in newRows"
            :key="draft.id"
            class="border-2 border-dashed border-emerald-200 rounded-lg p-3 bg-emerald-50/50 space-y-2"
          >
            <div class="text-xs font-medium text-emerald-800">
              {{ t('collectionCustomTextBulk.newRowSectionTitle') }}{{ newRows.length > 1 ? ` (${dIdx + 1})` : '' }}
            </div>
            <div>
              <label class="block text-xs font-medium text-gray-600 mb-1">{{ t('collectionCustomTextBulk.colFront') }}</label>
              <textarea
                v-model="draft.free_content_front"
                rows="1"
                class="bulk-sheet-input js-bulk-auto-text w-full"
                :placeholder="t('collectionCustomTextBulk.newRowFrontPlaceholder')"
                @input="onBulkTextInput"
              />
            </div>
            <div>
              <label class="block text-xs font-medium text-gray-600 mb-1">{{ t('collectionCustomTextBulk.colBack') }}</label>
              <textarea
                v-model="draft.free_content_back"
                rows="1"
                class="bulk-sheet-input js-bulk-auto-text w-full"
                :placeholder="t('collectionCustomTextBulk.newRowBackPlaceholder')"
                @input="onBulkTextInput"
              />
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Save bar: sticky on mobile -->
    <div
      v-if="isOwner && !isLoading"
      class="fixed bottom-0 left-0 right-0 md:static md:mt-6 p-4 bg-white/95 md:bg-transparent border-t md:border-t-0 border-gray-200 backdrop-blur-sm z-20 flex justify-end gap-3 safe-area-pb"
    >
      <button
        type="button"
        class="btn-cancel"
        :disabled="isSaving || !isDirty"
        @click="resetRows"
      >
        {{ t('collectionCustomTextBulk.revert') }}
      </button>
      <button
        type="button"
        class="btn-update min-w-[10rem]"
        :disabled="isSaving || !isDirty"
        :aria-busy="isSaving"
        @click="saveAll"
      >
        <span class="inline-flex h-5 w-5 shrink-0 items-center justify-center" aria-hidden="true">
          <Loader2 v-if="isSaving" class="h-5 w-5 animate-spin" />
          <Save v-else class="h-5 w-5" />
        </span>
        {{ isSaving ? t('collectionCustomTextBulk.saving') : t('collectionCustomTextBulk.saveAll') }}
      </button>
    </div>
  </div>
</template>

<script setup>
import { ArrowLeft, List, Loader2, Save } from 'lucide-vue-next'
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  addCollectionItem,
  bulkUpdateCustomTextItems,
  getCollection,
  listCustomTextBulkItems,
} from '@/api'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSuccessToast } from '@/composables/useSuccessToast'
import { useSeoHead } from '@/composables/useSeoHead'

const { t, locale } = useI18n()
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

const isOwner = computed(
  () => collection.value?.owner?.username === auth.state.username,
)

const hasNewRowContent = computed(() =>
  newRows.value.some(
    (r) =>
      r.free_content_front.trim() !== '' || r.free_content_back.trim() !== '',
  ),
)

/** When the last draft row gets any text, append another empty draft so the user can keep adding. */
function ensureTrailingEmptyDraft() {
  const list = newRows.value
  if (list.length === 0) {
    newRows.value = [createEmptyDraft()]
    return
  }
  const last = list[list.length - 1]
  const lastIsEmpty
    = !last.free_content_front.trim() && !last.free_content_back.trim()
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
  const maxPx
    = cs.maxHeight && cs.maxHeight !== 'none' && !Number.isNaN(parseFloat(cs.maxHeight))
      ? parseFloat(cs.maxHeight)
      : 160
  const minPx
    = cs.minHeight && cs.minHeight !== '0px' && !Number.isNaN(parseFloat(cs.minHeight))
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
  { deep: true },
)

const isDirty = computed(() => {
  const rowsChanged
    = snapshotJson.value !== ''
      && JSON.stringify(rows.value) !== snapshotJson.value
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
    if (
      !isPublic
      && (!auth.state.isLoggedIn || ownerName !== auth.state.username)
    ) {
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
    (r) =>
      r.free_content_front.trim() !== '' || r.free_content_back.trim() !== '',
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
      }),
    )

    await load(true)
  } catch (e) {
    showError(e.response?.data?.error || e.message || t('collectionCustomTextBulk.saveError'))
  } finally {
    isSaving.value = false
  }
}

onMounted(load)

watch(
  () => props.collectionId,
  () => load(),
)

useSeoHead(
  { title: computed(() => t('collectionCustomTextBulk.documentTitle')) },
  locale.value,
)
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
