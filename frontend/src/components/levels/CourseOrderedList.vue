<template>
  <div class="flex flex-col gap-3">
    <!-- Flat mode: the caller's own order/pagination (search, filters…), no headings. -->
    <template v-if="flat">
      <div v-for="item in items" :key="String(itemKey(item))" class="min-w-0">
        <slot name="item" :item="item" />
      </div>
    </template>
    <div v-else-if="!levelsReady" class="flex justify-center py-12">
      <LoadingSpinner variant="inline" />
    </div>
    <template v-else>
      <div v-if="isOwner" class="course-toolbar flex flex-col gap-1.5">
        <!-- Row 1: mode toggle (left) · save status + Add level (right). Same height in both modes. -->
        <div class="flex min-h-[2.5rem] items-center justify-between gap-2">
          <div
            class="inline-flex shrink-0 rounded-md border border-gray-200 bg-white p-0.5"
            role="group"
            :aria-label="t('courseOrder.modeLabel')"
          >
            <button
              v-for="m in ['learn', 'edit'] as const"
              :key="m"
              type="button"
              class="rounded px-3 py-1 text-xs font-medium transition-colors"
              :class="mode === m ? 'bg-indigo-600 text-white' : 'text-gray-600 hover:bg-gray-100'"
              :aria-pressed="mode === m"
              :data-mode="m"
              @click="setMode(m)"
            >
              {{ t(m === 'edit' ? 'courseOrder.modeEdit' : 'courseOrder.modeLearn') }}
            </button>
          </div>
          <div class="flex shrink-0 items-center gap-2" :class="{ invisible: !editMode }">
            <span class="text-right text-xs text-gray-500" aria-live="polite">{{
              saveState === 'saving'
                ? t('courseOrder.saving')
                : saveState === 'saved'
                  ? t('courseOrder.saved')
                  : ''
            }}</span>
            <Button variant="create" :tabindex="editMode ? undefined : -1" @click="openCreate">
              <template #icon><PlusCircle class="h-4 w-4" /></template>
              {{ t('levelHeading.addLevel') }}
            </Button>
          </div>
        </div>
        <!-- Row 2: full-width hint. Both texts share one grid cell so the row is always as tall
             as the longer one: switching modes never shifts the list (CLS 0). -->
        <div class="grid text-left text-xs leading-snug text-gray-500">
          <p
            class="col-start-1 row-start-1"
            :class="{ invisible: !editMode }"
            :aria-hidden="!editMode"
          >
            {{ t('courseOrder.ownerHint') }}
          </p>
          <p
            class="col-start-1 row-start-1"
            :class="{ invisible: editMode }"
            :aria-hidden="editMode"
          >
            {{ t('courseOrder.readOnlyHint') }}
          </p>
        </div>
      </div>
      <template v-for="entry in pageEntries" :key="entry.key">
        <LevelHeading
          v-if="entry.kind === 'heading'"
          :level="entry.level"
          :index="entry.levelIndex"
          :count="entry.count"
          :is-owner="editMode"
          :is-last="entry.levelIndex === levels.length - 1"
          :show-reorder="editMode && levels.length > 1"
          :continued="entry.continued"
          :anchor="!entry.continued"
          :highlighted="entry.level != null && highlightedLevelId === entry.level.level_id"
          @dragover.prevent
          @drop.prevent="onDrop(entry.sectionId, 0)"
          @rename="(name) => entry.level && renameLevel(entry.level, name)"
          @edit="entry.level && openEdit(entry.level)"
          @move-up="moveLevel(entry.levelIndex, -1)"
          @move-down="moveLevel(entry.levelIndex, 1)"
        />
        <div
          v-else
          class="course-row flex min-w-0 items-stretch gap-1.5 sm:gap-2"
          :class="{
            'opacity-50': dragging?.key === entry.rowKey && dragging.from === entry.sectionId,
          }"
          :data-row-key="entry.rowKey"
          :draggable="editMode && entry.flashcardId != null"
          @dragstart="onDragStart($event, entry.rowKey, entry.sectionId)"
          @dragend="dragging = null"
          @dragover.prevent
          @drop.prevent="onDrop(entry.sectionId, entry.index)"
        >
          <!-- Edit mode only: reorder column (Learn mode gives the card the full width). -->
          <div
            v-if="isOwner && editMode"
            class="course-row__handle flex shrink-0 flex-col items-center justify-start gap-1 pt-1"
          >
            <template v-if="entry.flashcardId != null">
              <GripVertical
                class="hidden h-4 w-4 cursor-grab text-gray-400 sm:block"
                aria-hidden="true"
              />
              <IconButton
                button-classes="ui-btn--toolbar course-row__arrow"
                :aria-label="t('courseOrder.moveUp')"
                :disabled="!canMove(entry, -1)"
                @click="nudge(entry, -1)"
              >
                <template #icon><ArrowUp class="h-4 w-4" /></template>
              </IconButton>
              <IconButton
                button-classes="ui-btn--toolbar course-row__arrow"
                :aria-label="t('courseOrder.moveDown')"
                :disabled="!canMove(entry, 1)"
                @click="nudge(entry, 1)"
              >
                <template #icon><ArrowDown class="h-4 w-4" /></template>
              </IconButton>
            </template>
          </div>
          <div
            class="min-w-0 flex-1"
            :class="{ 'course-row--highlight': highlightedRowKey === entry.rowKey }"
          >
            <slot name="item" :item="entry.item" />
          </div>
        </div>
      </template>
      <PaginationComponent
        v-if="pages.length > 1"
        :current-page="page"
        :total-pages="pages.length"
        :total="rowCount"
        :per-page="pageSize"
        @prev="goToPage(page - 1)"
        @next="goToPage(page + 1)"
      />
    </template>

    <LevelEditModal
      v-if="isOwner"
      :show="showLevelModal"
      :collection-id="collectionId"
      :level="editingLevel"
      :levels="levels"
      @close="showLevelModal = false"
      @saved="onLevelSaved"
      @deleted="onLevelSaved"
    />
  </div>
</template>

<script setup lang="ts" generic="T">
import { ArrowDown, ArrowUp, GripVertical, PlusCircle } from '@lucide/vue'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { saveCourseOrder, updateLevel } from '@/api'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import PaginationComponent from '@/components/PaginationComponent.vue'
import LevelEditModal from '@/components/levels/LevelEditModal.vue'
import LevelHeading from '@/components/levels/LevelHeading.vue'
import { useCourseLevels, type CourseLevel } from '@/composables/useCourseLevels'
import { useError } from '@/composables/useError'
import { useListMode } from '@/composables/useListMode'
import { getApiErrorMessage } from '@/utils/apiError'
import { Button, IconButton } from '@packages/ui'

/**
 * A collection's cards in course order (level position, then card position) interspersed with
 * level heading rows, plus an "Unassigned" section. Owners rename/edit/reorder levels in place
 * and move cards with up/down arrows (crossing a heading moves the card into that level) or drag.
 * Every move is saved with one PUT …/course-order call (optimistic, rolled back on error).
 * `flat` renders the caller's items as-is (used while search/filters break the course order).
 */
const props = withDefaults(
  defineProps<{
    collectionId: number | string
    isOwner: boolean
    /** All items (course mode) or the current page (flat mode). */
    items: T[]
    itemKey: (item: T) => string | number
    flashcardIdOf: (item: T) => number | null | undefined
    flat?: boolean
    pageSize?: number
    /** Level to show/scroll to/highlight once loaded (e.g. from `#level-<id>`). */
    focusLevelId?: number | null
    /** Item (by `itemKey`) to show/scroll to/highlight once loaded (e.g. `?item=`). */
    focusItemKey?: string | number | null
  }>(),
  { flat: false, pageSize: 30, focusLevelId: null, focusItemKey: null }
)

const emit = defineEmits<{ changed: [] }>()

const { t } = useI18n()
const { mode, isEditMode: editMode, setMode } = useListMode(() => props.isOwner)
const { showError } = useError()

const UNASSIGNED = 0
const course = useCourseLevels(() => props.collectionId)
const levels = course.levels
const levelsReady = computed(() => course.isLoaded.value)
const order = ref(new Map<number, number[]>())
const page = ref(1)
const saveState = ref<'idle' | 'saving' | 'saved'>('idle')
const highlightedLevelId = ref<number | null>(null)
const highlightedRowKey = ref<string | null>(null)
const dragging = ref<{ key: string; from: number } | null>(null)
const showLevelModal = ref(false)
const editingLevel = ref<CourseLevel | null>(null)

const itemByFlashcard = computed(() => {
  const map = new Map<number, T>()
  for (const it of props.items) {
    const id = props.flashcardIdOf(it)
    if (id != null && !map.has(id)) map.set(id, it)
  }
  return map
})

/** Items that have no flashcard yet: listed in Unassigned, cannot join a level. */
const itemsWithoutFlashcard = computed(() =>
  props.items.filter((it) => props.flashcardIdOf(it) == null)
)

function rebuildOrder() {
  const map = new Map<number, number[]>()
  for (const [levelId, ids] of course.cardsByLevel.value) map.set(levelId, [...ids])
  const assigned = course.assignedFlashcardIds.value
  const unassigned: number[] = []
  for (const it of props.items) {
    const id = props.flashcardIdOf(it)
    if (id != null && !assigned.has(id) && !unassigned.includes(id)) unassigned.push(id)
  }
  map.set(UNASSIGNED, unassigned)
  order.value = map
}

type HeadingEntry = {
  kind: 'heading'
  key: string
  level: CourseLevel | null
  levelIndex: number
  sectionId: number
  count: number
  continued: boolean
}
type RowEntry = {
  kind: 'row'
  key: string
  rowKey: string
  item: T
  flashcardId: number | null
  sectionId: number
  index: number
  sectionLen: number
  sectionPos: number
}
type Entry = HeadingEntry | RowEntry

const sections = computed(() => {
  const out: Array<{ sectionId: number; level: CourseLevel | null; ids: number[] }> = []
  for (const l of levels.value)
    out.push({ sectionId: l.level_id, level: l, ids: order.value.get(l.level_id) ?? [] })
  out.push({ sectionId: UNASSIGNED, level: null, ids: order.value.get(UNASSIGNED) ?? [] })
  return out
})

const entries = computed<Entry[]>(() => {
  const out: Entry[] = []
  sections.value.forEach((sec, sPos) => {
    const rows: RowEntry[] = []
    sec.ids.forEach((fid) => {
      const item = itemByFlashcard.value.get(fid)
      if (!item) return
      rows.push({
        kind: 'row',
        key: `${sec.sectionId}-f${fid}`,
        rowKey: `f${fid}`,
        item,
        flashcardId: fid,
        sectionId: sec.sectionId,
        index: rows.length,
        sectionLen: 0,
        sectionPos: sPos,
      })
    })
    if (sec.sectionId === UNASSIGNED) {
      for (const it of itemsWithoutFlashcard.value) {
        rows.push({
          kind: 'row',
          key: `0-i${props.itemKey(it)}`,
          rowKey: `i${props.itemKey(it)}`,
          item: it,
          flashcardId: null,
          sectionId: UNASSIGNED,
          index: rows.length,
          sectionLen: 0,
          sectionPos: sPos,
        })
      }
      if (!rows.length && !props.isOwner) return
    }
    const withFc = rows.filter((r) => r.flashcardId != null).length
    rows.forEach((r) => (r.sectionLen = withFc))
    out.push({
      kind: 'heading',
      key: `h${sec.sectionId}`,
      level: sec.level,
      levelIndex: sec.level ? sPos : levels.value.length,
      sectionId: sec.sectionId,
      count: rows.length,
      continued: false,
    })
    out.push(...rows)
  })
  return out
})

const rowCount = computed(() => entries.value.filter((e) => e.kind === 'row').length)

/** Pages of ~pageSize entries; a page starting mid-level repeats that level's heading. */
const pages = computed(() => {
  const result: Entry[][] = []
  let cur: Entry[] = []
  let lastHeading: HeadingEntry | null = null
  for (const e of entries.value) {
    if (e.kind === 'heading') lastHeading = e
    if (cur.length >= props.pageSize && !(e.kind === 'row' && cur.length === 0)) {
      result.push(cur)
      cur = []
      if (e.kind === 'row' && lastHeading) {
        cur.push({ ...lastHeading, key: `${lastHeading.key}-c${result.length}`, continued: true })
      }
    }
    cur.push(e)
  }
  if (cur.length) result.push(cur)
  return result
})

const pageEntries = computed(() => pages.value[Math.min(page.value, pages.value.length) - 1] ?? [])

watch(
  () => pages.value.length,
  (n) => {
    if (page.value > n) page.value = Math.max(1, n)
  }
)

function goToPage(p: number) {
  page.value = Math.max(1, Math.min(p, pages.value.length))
  void nextTick(() =>
    document.querySelector('.course-row, .level-heading')?.scrollIntoView({ block: 'start' })
  )
}

function canMove(e: RowEntry, dir: -1 | 1) {
  if (e.sectionId === UNASSIGNED) return dir === -1 && levels.value.length > 0 && e.index === 0
  const target = e.index + dir
  if (target >= 0 && target < e.sectionLen) return true
  return dir === -1 ? e.sectionPos > 0 : true
}

function nudge(e: RowEntry, dir: -1 | 1) {
  if (e.flashcardId == null) return
  const target = e.index + dir
  if (e.sectionId !== UNASSIGNED && target >= 0 && target < e.sectionLen) {
    void moveCard(e.flashcardId, e.sectionId, e.sectionId, dir === 1 ? target + 1 : target)
    return
  }
  const neighbour = sections.value[e.sectionPos + dir]
  if (!neighbour) return
  void moveCard(
    e.flashcardId,
    e.sectionId,
    neighbour.sectionId,
    dir === 1 ? 0 : neighbour.ids.length
  )
}

async function moveCard(fid: number, from: number, to: number, toIndex: number) {
  const before = order.value
  const map = new Map(order.value)
  const src = [...(map.get(from) ?? [])]
  const srcIdx = src.indexOf(fid)
  if (srcIdx < 0) return
  src.splice(srcIdx, 1)
  map.set(from, src)
  const dst = from === to ? src : [...(map.get(to) ?? [])]
  let idx = toIndex
  if (from === to && srcIdx < toIndex) idx -= 1
  const dup = dst.indexOf(fid)
  if (dup >= 0) dst.splice(dup, 1)
  dst.splice(Math.max(0, Math.min(idx, dst.length)), 0, fid)
  map.set(to, dst)
  const stillAssigned = [...map.entries()].some(
    ([lid, ids]) => lid !== UNASSIGNED && ids.includes(fid)
  )
  const un = (map.get(UNASSIGNED) ?? []).filter((id) => id !== fid || to === UNASSIGNED)
  if (!stillAssigned && !un.includes(fid)) un.unshift(fid)
  map.set(UNASSIGNED, un)
  order.value = map
  highlightedRowKey.value = `f${fid}`

  const touched = [...new Set([from, to])].filter((id) => id !== UNASSIGNED)
  if (!touched.length) return
  saveState.value = 'saving'
  try {
    await saveCourseOrder(props.collectionId, {
      levels: touched.map((id) => ({ level_id: id, flashcard_ids: map.get(id) ?? [] })),
    })
    saveState.value = 'saved'
    // Refresh counts/memberships quietly; the optimistic order already matches.
    await course.load()
    emit('changed')
  } catch (err) {
    order.value = before
    saveState.value = 'idle'
    showError(getApiErrorMessage(err) || t('courseOrder.saveError'))
  }
}

function onDragStart(e: DragEvent, key: string, from: number) {
  if (!editMode.value || !key.startsWith('f')) return
  dragging.value = { key, from }
  e.dataTransfer?.setData('text/plain', key)
  if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move'
}

function onDrop(to: number, index: number) {
  const d = dragging.value
  dragging.value = null
  if (!d || !editMode.value) return
  void moveCard(Number(d.key.slice(1)), d.from, to, index)
}

async function moveLevel(i: number, dir: -1 | 1) {
  const ids = levels.value.map((l) => l.level_id)
  const j = i + dir
  if (j < 0 || j >= ids.length) return
  const before = levels.value
  ;[ids[i], ids[j]] = [ids[j], ids[i]]
  levels.value = ids.map((id) => before.find((l) => l.level_id === id) as CourseLevel)
  saveState.value = 'saving'
  try {
    await saveCourseOrder(props.collectionId, { levels: [], level_order: ids })
    saveState.value = 'saved'
    await course.load()
    emit('changed')
  } catch (err) {
    levels.value = before
    saveState.value = 'idle'
    showError(getApiErrorMessage(err) || t('courseOrder.saveError'))
  }
}

async function renameLevel(level: CourseLevel, name: string) {
  const old = level.name
  level.name = name
  try {
    await updateLevel(level.level_id, { name })
    emit('changed')
  } catch (err) {
    level.name = old
    showError(getApiErrorMessage(err) || t('flashcardLevels.saveError'))
  }
}

function openCreate() {
  editingLevel.value = null
  showLevelModal.value = true
}

function openEdit(level: CourseLevel) {
  editingLevel.value = level
  showLevelModal.value = true
}

async function onLevelSaved() {
  showLevelModal.value = false
  await course.load()
  rebuildOrder()
  emit('changed')
}

function flash(target: 'level' | 'row', key: number | string) {
  if (target === 'level') highlightedLevelId.value = Number(key)
  else highlightedRowKey.value = String(key)
  window.setTimeout(() => {
    if (target === 'level' && highlightedLevelId.value === Number(key))
      highlightedLevelId.value = null
    if (target === 'row' && highlightedRowKey.value === String(key)) highlightedRowKey.value = null
  }, 2500)
}

/** Height of sticky/fixed chrome at the top of the viewport (header, toolbars). */
function stickyTopOffset() {
  let bottom = 0
  for (const el of Array.from(
    document.querySelectorAll<HTMLElement>('header, [data-sticky-top]')
  )) {
    const cs = getComputedStyle(el)
    if (cs.position !== 'sticky' && cs.position !== 'fixed') continue
    const r = el.getBoundingClientRect()
    if (r.top <= 1 && r.bottom > bottom && r.height < window.innerHeight / 2) bottom = r.bottom
  }
  return bottom + 12
}

function scrollToEl(el: HTMLElement, smooth: boolean) {
  const top = el.getBoundingClientRect().top + window.scrollY - stickyTopOffset()
  window.scrollTo({ top: Math.max(0, top), behavior: smooth ? 'smooth' : 'auto' })
}

const frame = () => new Promise<void>((r) => requestAnimationFrame(() => r()))

/** Show the page holding a level heading, scroll to it (below sticky chrome) and highlight it. */
async function focusLevel(levelId: number, smooth = false) {
  // Wait (briefly) for the cards to arrive so the target does not move after we scroll.
  for (let i = 0; i < 30 && props.items.length === 0; i++) {
    await new Promise((r) => setTimeout(r, 100))
  }
  const p = pages.value.findIndex((pg) =>
    pg.some((e) => e.kind === 'heading' && !e.continued && e.sectionId === levelId)
  )
  if (p < 0) return
  page.value = p + 1
  await nextTick()
  await frame()
  const el = document.getElementById(`level-${levelId}`)
  if (!el) return
  scrollToEl(el, smooth)
  flash('level', levelId)
  // Images/cards above may still settle: correct once more if the heading moved.
  setTimeout(() => {
    const again = document.getElementById(`level-${levelId}`)
    if (again && Math.abs(again.getBoundingClientRect().top - stickyTopOffset()) > 4) {
      scrollToEl(again, false)
    }
  }, 700)
}

/** Same-page `#level-<id>` links (incl. re-clicking the current hash) and manual hash edits. */
function onDocClick(ev: MouseEvent) {
  if (props.flat || ev.defaultPrevented || ev.button !== 0 || ev.metaKey || ev.ctrlKey) return
  const a = (ev.target as HTMLElement | null)?.closest?.(
    'a[href*="#level-"]'
  ) as HTMLAnchorElement | null
  if (!a) return
  const url = new URL(a.href, window.location.href)
  if (url.pathname !== window.location.pathname) return
  const m = /^#level-(\d+)$/.exec(url.hash)
  if (m && url.hash === window.location.hash) void focusLevel(Number(m[1]), true)
}
function onHashChange() {
  const m = /^#level-(\d+)$/.exec(window.location.hash)
  if (m && !props.flat && course.isLoaded.value) void focusLevel(Number(m[1]), true)
}
onMounted(() => {
  document.addEventListener('click', onDocClick, true)
  window.addEventListener('hashchange', onHashChange)
})
onBeforeUnmount(() => {
  document.removeEventListener('click', onDocClick, true)
  window.removeEventListener('hashchange', onHashChange)
})

/** Show the page holding an item (first occurrence), scroll to it and highlight it. */
async function focusItem(match: (item: T) => boolean) {
  for (let p = 0; p < pages.value.length; p++) {
    const e = pages.value[p].find((x) => x.kind === 'row' && match(x.item)) as RowEntry | undefined
    if (!e) continue
    page.value = p + 1
    await nextTick()
    document
      .querySelector(`[data-row-key="${e.rowKey}"]`)
      ?.scrollIntoView({ behavior: 'smooth', block: 'center' })
    flash('row', e.rowKey)
    return true
  }
  return false
}

async function load() {
  await course.load()
  rebuildOrder()
  if (props.focusLevelId) void focusLevel(props.focusLevelId)
  else if (props.focusItemKey != null) {
    const key = String(props.focusItemKey)
    void focusItem((it) => String(props.itemKey(it)) === key)
  }
}

watch(
  () => props.focusLevelId,
  (id) => {
    if (id && course.isLoaded.value && !props.flat) void focusLevel(id, true)
  }
)

watch(
  () => props.items,
  () => {
    if (!props.flat && course.isLoaded.value) rebuildOrder()
  }
)

onMounted(() => {
  if (!props.flat) void load()
})
watch(
  () => props.flat,
  (flat) => {
    if (!flat) void load()
  }
)

defineExpose({ load, focusLevel, focusItem, isLoaded: course.isLoaded })
</script>

<style scoped>
.course-row--highlight {
  border-radius: 0.5rem;
  box-shadow: 0 0 0 3px rgb(99 102 241 / 0.5);
  transition: box-shadow 0.3s;
}

/* Square, fixed-size reorder arrows sized to their column (never overlap the card):
   40px touch targets on phones, 32px with a pointer-sized layout. */
.course-row__handle {
  width: 2.5rem;
}
.course-row__handle :deep(.course-row__arrow) {
  width: 2.5rem;
  min-width: 2.5rem;
  height: 2.5rem;
  padding: 0;
  justify-content: center;
}
@media (min-width: 640px) {
  .course-row__handle {
    width: 2rem;
  }
  .course-row__handle :deep(.course-row__arrow) {
    width: 2rem;
    min-width: 2rem;
    height: 2rem;
  }
}
</style>
