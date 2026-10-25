<template>
  <div class="course-order flex flex-col gap-3" :aria-busy="!isReady">
    <div v-if="!isReady" class="flex justify-center py-10">
      <Loader2 class="h-6 w-6 animate-spin text-gray-400" :aria-label="t('levelHeading.loading')" />
    </div>
    <template v-else>
      <div class="flex min-h-[2.5rem] flex-wrap items-center gap-2">
        <p class="min-w-0 flex-1 text-sm text-gray-600">
          {{ isOwner ? t('courseOrder.ownerHint') : t('courseOrder.readOnlyHint') }}
        </p>
        <span class="min-w-[5rem] text-right text-xs text-gray-500" aria-live="polite">
          {{
            saveState === 'saving'
              ? t('courseOrder.saving')
              : saveState === 'saved'
                ? t('courseOrder.saved')
                : ''
          }}
        </span>
        <Button v-if="isOwner" variant="create" @click="openCreate">
          <template #icon><PlusCircle class="h-4 w-4" /></template>
          {{ t('courseOrder.addLevel') }}
        </Button>
      </div>

      <section
        v-for="(section, sIdx) in sections"
        :key="section.key"
        class="flex flex-col gap-1"
        @dragover.prevent="onDragOverSection"
        @drop.prevent="onDrop(section.levelId, section.cards.length)"
      >
        <LevelHeading
          :level="section.level"
          :index="sIdx"
          :count="section.cards.length"
          :is-owner="isOwner"
          :is-last="sIdx === levels.length - 1"
          :show-reorder="isOwner && levels.length > 1"
          :highlighted="section.level != null && highlightedLevelId === section.level.level_id"
          @dragover.prevent
          @drop.prevent.stop="onDrop(section.levelId, 0)"
          @rename="(name) => section.level && renameLevel(section.level, name)"
          @edit="section.level && openEdit(section.level)"
          @move-up="moveLevel(sIdx, -1)"
          @move-down="moveLevel(sIdx, 1)"
        />
        <p v-if="!section.cards.length" class="px-3 py-2 text-xs text-gray-400">
          {{ t('courseOrder.emptyLevel') }}
        </p>
        <ol class="flex flex-col gap-1">
          <li
            v-for="(card, cIdx) in section.cards"
            :key="`${section.key}-${card.id}`"
            class="course-row flex items-center gap-2 rounded-md border border-gray-200 bg-white px-2 py-1.5"
            :class="{
              'opacity-50': dragging?.flashcardId === card.id && dragging.from === section.levelId,
            }"
            :draggable="isOwner"
            @dragstart="onDragStart($event, card.id, section.levelId)"
            @dragend="dragging = null"
            @dragover.prevent
            @drop.prevent.stop="onDrop(section.levelId, cIdx)"
          >
            <GripVertical
              v-if="isOwner"
              class="hidden h-4 w-4 shrink-0 cursor-grab text-gray-400 sm:block"
              aria-hidden="true"
            />
            <RouterLink
              :to="`/collections/${collectionId}/flashcards?item=${card.item_id}`"
              class="flex min-w-0 flex-1 flex-col sm:flex-row sm:items-baseline sm:gap-2"
            >
              <span class="truncate text-sm font-medium text-gray-800">{{ card.front }}</span>
              <span class="truncate text-xs text-gray-500">{{ card.back }}</span>
            </RouterLink>
            <div v-if="isOwner" class="flex shrink-0 items-center gap-1">
              <IconButton
                button-classes="ui-btn--toolbar"
                :aria-label="t('courseOrder.moveUp')"
                :disabled="sIdx === 0 && cIdx === 0"
                @click="nudge(sIdx, cIdx, -1)"
              >
                <template #icon><ArrowUp class="h-4 w-4" /></template>
              </IconButton>
              <IconButton
                button-classes="ui-btn--toolbar"
                :aria-label="t('courseOrder.moveDown')"
                :disabled="sIdx === sections.length - 1 && cIdx === section.cards.length - 1"
                @click="nudge(sIdx, cIdx, 1)"
              >
                <template #icon><ArrowDown class="h-4 w-4" /></template>
              </IconButton>
              <select
                class="input-field h-8 w-9 sm:w-40"
                :aria-label="t('courseOrder.moveToLevel')"
                :value="''"
                @change="onMoveSelect($event, card.id, section.levelId)"
              >
                <option value="" disabled>{{ t('courseOrder.moveToLevel') }}</option>
                <option
                  v-for="opt in moveTargets(section.levelId)"
                  :key="opt.id"
                  :value="String(opt.id)"
                >
                  {{ opt.label }}
                </option>
              </select>
            </div>
          </li>
        </ol>
      </section>
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

<script setup lang="ts">
import { ArrowDown, ArrowUp, GripVertical, Loader2, PlusCircle } from '@lucide/vue'
import { computed, nextTick, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'

import { getCollectionFlashcardsPublic, saveCourseOrder, updateLevel } from '@/api'
import LevelEditModal from '@/components/levels/LevelEditModal.vue'
import LevelHeading from '@/components/levels/LevelHeading.vue'
import { useCourseLevels, type CourseLevel } from '@/composables/useCourseLevels'
import { useError } from '@/composables/useError'
import { getApiErrorMessage } from '@/utils/apiError'
import { Button, IconButton } from '@packages/ui'

/**
 * "Course order": every card listed in level order with level titles as headings.
 * Owners can drag cards across headings (or use arrows / "Move to level…" on mobile),
 * rename / add / edit levels; everybody else gets a read-only view.
 * Card moves are persisted with ONE batched request (PUT …/course-order).
 */
const props = defineProps<{
  collectionId: number | string
  isOwner: boolean
  /** Level to scroll to and highlight once loaded (from `#level-<id>`). */
  focusLevelId?: number | null
}>()

const emit = defineEmits<{ changed: [] }>()

const { t } = useI18n()
const { showError } = useError()

interface CardRow {
  id: number
  item_id: number
  front: string
  back: string
}

const UNASSIGNED = 0
const course = useCourseLevels(() => props.collectionId)
const levels = course.levels
const cards = ref<CardRow[]>([])
const cardsLoaded = ref(false)
/** Local, editable order: level_id (0 = unassigned) -> flashcard ids. */
const order = ref(new Map<number, number[]>())
const saveState = ref<'idle' | 'saving' | 'saved'>('idle')
const highlightedLevelId = ref<number | null>(null)
const dragging = ref<{ flashcardId: number; from: number } | null>(null)
const showLevelModal = ref(false)
const editingLevel = ref<CourseLevel | null>(null)

const isReady = computed(() => course.isLoaded.value && cardsLoaded.value)
const cardById = computed(() => new Map(cards.value.map((c) => [c.id, c])))

function stripMarkup(text: unknown) {
  return String(text ?? '')
    .replace(/<[^>]+>/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
}

async function loadCards() {
  try {
    const { data } = await getCollectionFlashcardsPublic(props.collectionId)
    const list = (data?.flashcards ?? []) as Array<{ flashcard: Record<string, unknown> }>
    cards.value = list.map(({ flashcard: f }) => ({
      id: Number(f.id),
      item_id: Number(f.item_id),
      front: stripMarkup(f.word ?? f.free_content_front),
      back: stripMarkup(f.definition ?? f.free_content_back),
    }))
  } catch (err) {
    console.error('Error loading cards for course order:', err)
    cards.value = []
  } finally {
    cardsLoaded.value = true
  }
}

function rebuildOrder() {
  const map = new Map<number, number[]>()
  for (const [levelId, ids] of course.cardsByLevel.value) map.set(levelId, [...ids])
  const assigned = course.assignedFlashcardIds.value
  map.set(
    UNASSIGNED,
    cards.value.filter((c) => !assigned.has(c.id)).map((c) => c.id)
  )
  order.value = map
}

const sections = computed(() => {
  const out: Array<{ key: string; levelId: number; level: CourseLevel | null; cards: CardRow[] }> =
    []
  const toRows = (ids: number[]) =>
    ids.map((id) => cardById.value.get(id)).filter((c): c is CardRow => !!c)
  for (const level of levels.value) {
    out.push({
      key: `l${level.level_id}`,
      levelId: level.level_id,
      level,
      cards: toRows(order.value.get(level.level_id) ?? []),
    })
  }
  const unassigned = order.value.get(UNASSIGNED) ?? []
  if (unassigned.length || props.isOwner) {
    out.push({ key: 'unassigned', levelId: UNASSIGNED, level: null, cards: toRows(unassigned) })
  }
  return out
})

function moveTargets(current: number) {
  const opts = levels.value
    .filter((l) => l.level_id !== current)
    .map((l) => ({ id: l.level_id, label: l.name }))
  if (current !== UNASSIGNED) opts.push({ id: UNASSIGNED, label: t('levelHeading.unassigned') })
  return opts
}

/** Move a card between sections locally, then persist only the touched levels. */
async function moveCard(flashcardId: number, from: number, to: number, toIndex: number) {
  const map = new Map(order.value)
  const src = [...(map.get(from) ?? [])]
  const srcIdx = src.indexOf(flashcardId)
  if (srcIdx < 0) return
  src.splice(srcIdx, 1)
  map.set(from, src)
  const dst = from === to ? src : [...(map.get(to) ?? [])]
  let idx = toIndex
  if (from === to && srcIdx < toIndex) idx -= 1
  const existing = dst.indexOf(flashcardId)
  if (existing >= 0) dst.splice(existing, 1)
  dst.splice(Math.max(0, Math.min(idx, dst.length)), 0, flashcardId)
  map.set(to, dst)
  // A card leaving its last level shows up in Unassigned; one entering a level leaves it.
  const unassigned = (map.get(UNASSIGNED) ?? []).filter(
    (id) => id !== flashcardId || to === UNASSIGNED
  )
  const stillAssigned = [...map.entries()].some(
    ([lid, ids]) => lid !== UNASSIGNED && ids.includes(flashcardId)
  )
  if (!stillAssigned && !unassigned.includes(flashcardId)) unassigned.push(flashcardId)
  map.set(UNASSIGNED, unassigned)
  order.value = map

  const touched = [...new Set([from, to])].filter((id) => id !== UNASSIGNED)
  if (!touched.length) return
  await persist({
    levels: touched.map((id) => ({ level_id: id, flashcard_ids: map.get(id) ?? [] })),
  })
}

async function persist(payload: Parameters<typeof saveCourseOrder>[1]) {
  saveState.value = 'saving'
  try {
    await saveCourseOrder(props.collectionId, payload)
    saveState.value = 'saved'
    await course.load()
    rebuildOrder()
    emit('changed')
  } catch (err) {
    saveState.value = 'idle'
    showError(getApiErrorMessage(err) || t('courseOrder.saveError'))
    await course.load()
    rebuildOrder()
  }
}

function nudge(sIdx: number, cIdx: number, dir: -1 | 1) {
  const section = sections.value[sIdx]
  const card = section.cards[cIdx]
  const target = cIdx + dir
  if (target >= 0 && target < section.cards.length) {
    void moveCard(card.id, section.levelId, section.levelId, dir === 1 ? target + 1 : target)
    return
  }
  // Crossing a heading moves the card into the neighbouring level.
  const neighbour = sections.value[sIdx + dir]
  if (!neighbour) return
  void moveCard(card.id, section.levelId, neighbour.levelId, dir === 1 ? 0 : neighbour.cards.length)
}

function onMoveSelect(e: Event, flashcardId: number, from: number) {
  const el = e.target as HTMLSelectElement
  const to = Number(el.value)
  el.value = ''
  const end = (order.value.get(to) ?? []).length
  void moveCard(flashcardId, from, to, end)
}

function onDragStart(e: DragEvent, flashcardId: number, from: number) {
  if (!props.isOwner) return
  dragging.value = { flashcardId, from }
  e.dataTransfer?.setData('text/plain', String(flashcardId))
  if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move'
}

function onDragOverSection(e: DragEvent) {
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
}

function onDrop(to: number, index: number) {
  const d = dragging.value
  dragging.value = null
  if (!d || !props.isOwner) return
  void moveCard(d.flashcardId, d.from, to, index)
}

async function moveLevel(sIdx: number, dir: -1 | 1) {
  const ids = levels.value.map((l) => l.level_id)
  const j = sIdx + dir
  if (j < 0 || j >= ids.length) return
  ;[ids[sIdx], ids[j]] = [ids[j], ids[sIdx]]
  await persist({ levels: [], level_order: ids })
}

async function renameLevel(level: CourseLevel, name: string) {
  try {
    await updateLevel(level.level_id, { name })
    await course.load()
    emit('changed')
  } catch (err) {
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

async function focusLevel(levelId: number) {
  await nextTick()
  const el = document.getElementById(`level-${levelId}`)
  if (!el) return
  el.scrollIntoView({ behavior: 'smooth', block: 'start' })
  highlightedLevelId.value = levelId
  window.setTimeout(() => {
    if (highlightedLevelId.value === levelId) highlightedLevelId.value = null
  }, 2500)
}

onMounted(async () => {
  await Promise.all([course.load(), loadCards()])
  rebuildOrder()
  if (props.focusLevelId) void focusLevel(props.focusLevelId)
})

defineExpose({ focusLevel, reload: onLevelSaved })
</script>

<style scoped>
.course-order {
  min-height: 60vh;
}

.course-row {
  min-height: 2.75rem;
}
</style>
