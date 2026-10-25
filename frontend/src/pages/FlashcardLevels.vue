<template>
  <div class="flex flex-col min-h-[calc(100vh-6rem)]">
    <PageHeader title-as="h2" stack-gap="comfortable" class="shrink-0">
      <template #title>{{ collection?.name }}</template>
      <template #meta-trailing>
        <CollectionActionsMenu
          v-if="isOwner"
          :collection-id="props.collectionId"
          :collection="collection"
          :is-owner="isOwner"
          @updated="onCollectionMenuUpdated"
          @refresh="fetchLevels"
        />
      </template>
      <template #toolbar>
        <div class="flex w-full flex-row flex-wrap items-center justify-between gap-2">
          <RouterLink
            :to="`/collections/${props.collectionId}/flashcards`"
            class="ui-btn--danger-rose"
          >
            {{ t('flashcardLevels.viewFlashcards') }}
          </RouterLink>
          <Button variant="warning-orange" size="lg" @click="startStudy">
            {{ t('flashcardLevels.studyButton') }}
          </Button>
          <IconButton
            v-if="isOwner"
            :label="t('flashcardLevels.createLevel')"
            button-classes="ui-btn--create"
            @click="openCreateLevel"
          />
        </div>
      </template>
    </PageHeader>
    <!-- Loading State -->
    <div v-if="isLoading" class="flex justify-center py-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
    </div>
    <!-- Levels Display -->
    <div v-else class="flex flex-col flex-1 min-h-0 gap-6">
      <!-- Empty State -->
      <div
        v-if="levels.length === 0"
        class="text-center py-12 bg-gray-50 rounded-lg border flex-shrink-0"
      >
        <p class="text-gray-600 mb-4">{{ t('flashcardLevels.noLevels') }}</p>

        <div class="flex justify-center">
          <IconButton
            v-if="isOwner"
            :label="t('flashcardLevels.createFirst')"
            button-classes="ui-btn--create"
            @click="openCreateLevel"
          />
        </div>
      </div>
      <!-- Flow Container -->
      <div v-else class="flex-1 min-h-0 flex flex-col">
        <VueFlow
          v-model="elements"
          :default-viewport="{ zoom: 1 }"
          :min-zoom="0.2"
          :max-zoom="4"
          class="vue-flow-wrapper flex-1 min-h-0"
        >
          <template #node-custom="nodeProps">
            <article
              class="level-card node-content"
              :class="getLevelCardClass(nodeProps.data)"
              :style="getLevelCardStyle(nodeProps.data)"
            >
              <div class="level-card-glow" aria-hidden="true" />
              <header class="level-card-head">
                <span class="level-card-number">{{ nodeProps.data.levelIndex ?? '?' }}</span>
                <div class="level-card-heading">
                  <h3 class="level-card-title">{{ nodeProps.data.name }}</h3>
                  <p v-if="nodeProps.data.description" class="level-card-desc">
                    {{ nodeProps.data.description }}
                  </p>
                </div>
                <span class="level-card-badge">
                  <Lock
                    v-if="
                      !isLevelUnlocked(nodeProps.data) && !nodeProps.data.progress?.is_completed
                    "
                    class="level-card-badge-icon"
                  />
                  <Dumbbell
                    v-else-if="isLevelUnlocked(nodeProps.data) && isLevelFinished90(nodeProps.data)"
                    class="level-card-badge-icon"
                  />
                  <Sprout v-else class="level-card-badge-icon" />
                </span>
              </header>

              <div
                v-if="!isLevelUnlocked(nodeProps.data) && !nodeProps.data.progress?.is_completed"
                class="level-card-meta"
              >
                <p v-if="nodeProps.data.prerequisites?.length" class="level-card-hint">
                  <Lock class="h-4 w-4 shrink-0 opacity-80" />
                  <span>{{ t('flashcardLevels.unlockHint') }}</span>
                </p>
                <p class="level-card-stats">
                  <span>{{ t('flashcardLevels.cardsInLevel') }}</span>
                  <span class="font-semibold">{{ nodeProps.data.card_count || 0 }}</span>
                </p>
              </div>
              <div v-else class="level-card-meta">
                <div class="level-card-progress-head">
                  <Target class="h-3.5 w-3.5 shrink-0 opacity-80" />
                  <span>{{ t('flashcardLevels.progress') }}</span>
                  <span class="level-card-progress-count">
                    {{ nodeProps.data.progress?.cards_completed || 0 }}/{{
                      nodeProps.data.card_count
                    }}
                  </span>
                </div>
                <div class="level-card-progress-track">
                  <div
                    class="level-card-progress-fill"
                    :style="{ width: getProgressWidth(nodeProps.data) }"
                  />
                </div>
                <p class="level-card-stats">
                  <span>{{ t('flashcardLevels.successRate') }}</span>
                  <span class="font-semibold">{{
                    formatSuccessRate(nodeProps.data.progress?.success_rate)
                  }}</span>
                </p>
                <p v-if="nodeProps.data.progress?.is_completed" class="level-card-completed">
                  <Sparkles class="h-3.5 w-3.5" /> {{ t('flashcardLevels.status.completed') }}
                </p>
              </div>

              <div class="level-card-actions">
                <RouterLink
                  v-if="!isOwner"
                  :to="cardsLink(nodeProps.data)"
                  class="level-card-btn level-card-btn-primary"
                >
                  <BookOpen class="h-4 w-4 shrink-0" /> {{ t('flashcardLevels.actions.viewCards') }}
                </RouterLink>
                <RouterLink
                  v-if="
                    nodeProps.data.card_count > 0 &&
                    (isLevelUnlocked(nodeProps.data) || nodeProps.data.progress?.is_completed)
                  "
                  :to="`/collections/${props.collectionId}/flashcards/study?levelId=${nodeProps.data.level_id}`"
                  class="level-card-btn level-card-btn-secondary"
                >
                  <Play class="h-4 w-4 shrink-0" /> {{ t('flashcardLevels.actions.practice') }}
                </RouterLink>
              </div>
              <div v-if="isOwner" class="level-card-actions level-card-actions--owner">
                <Button
                  variant="neutral"
                  type="button"
                  class="level-card-btn level-card-btn-ghost"
                  @click="editLevel(nodeProps.data)"
                >
                  <Settings class="h-4 w-4 shrink-0" /> {{ t('flashcardLevels.actions.edit') }}
                </Button>
                <RouterLink
                  :to="cardsLink(nodeProps.data)"
                  class="level-card-btn level-card-btn-ghost"
                >
                  <ListOrdered class="h-4 w-4 shrink-0" />
                  {{ t('flashcardLevels.actions.manageCards') }}
                </RouterLink>
              </div>
            </article>
          </template>
          <Background :pattern-color="'#aaa'" :gap="8" /> <Controls />
        </VueFlow>
      </div>
    </div>
    <LevelEditModal
      v-if="isOwner"
      :show="showLevelModal"
      :collection-id="props.collectionId"
      :level="editingLevel"
      :levels="levels"
      @close="showLevelModal = false"
      @saved="onLevelChanged"
      @deleted="onLevelChanged"
    />
  </div>
</template>

<script setup lang="ts">
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { VueFlow, useVueFlow } from '@vue-flow/core'
import type { Elements } from '@vue-flow/core'
import {
  Lock,
  Play,
  BookOpen,
  Settings,
  ListOrdered,
  Sparkles,
  Dumbbell,
  Sprout,
  Target,
} from '@lucide/vue'
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import CollectionActionsMenu from '@/components/CollectionActionsMenu.vue'
import LevelEditModal from '@/components/levels/LevelEditModal.vue'
import PageHeader from '@/components/layout/PageHeader.vue'

import { getCollection, getLevels } from '@/api'
import { Button, IconButton } from '@packages/ui'
import { useAuth } from '@/composables/useAuth'
import { useAnonymousProgress, type LevelProgressData } from '@/composables/useAnonymousProgress'
import { useSeoHead } from '@/composables/useSeoHead'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'

const props = defineProps({
  collectionId: {
    type: [String, Number],
    required: true,
    validator: (value) => !isNaN(Number(value)),
  },
})

const auth = useAuth()
const router = useRouter()
const { t } = useI18n()
const {
  getProgress,
  isLevelUnlockedForAnon,
  resolveAndRememberActiveLevelId,
  anonProgressVersion,
} = useAnonymousProgress()

function startStudy() {
  const sorted = [...levels.value].sort((a, b) => a.position - b.position)
  let targetLevelId

  if (auth.state.isLoggedIn) {
    const firstUnlockedIncomplete = sorted.find(
      (level) =>
        (!level.is_locked || !!level.progress?.is_unlocked) && !level.progress?.is_completed
    )
    targetLevelId = firstUnlockedIncomplete?.level_id ?? sorted[0]?.level_id
  } else {
    targetLevelId =
      resolveAndRememberActiveLevelId(props.collectionId, sorted) ?? sorted[0]?.level_id
  }

  if (targetLevelId) {
    router.push(`/collections/${props.collectionId}/flashcards/study?levelId=${targetLevelId}`)
  } else {
    router.push(`/collections/${props.collectionId}/flashcards/study`)
  }
}

// State
const collection = ref(null)
const levels = ref([])
const isLoading = ref(true)
const showLevelModal = ref(false)
const editingLevel = ref(null)

// Computed
const isOwner = computed(() => {
  return auth.state.isLoggedIn && collection.value?.owner?.username === auth.state.username
})

function onCollectionMenuUpdated(next) {
  collection.value = next
}

// Levels with anonymous progress merged in (is_locked, progress) when not logged in
const effectiveLevels = computed(() => {
  const raw = levels.value
  if (auth.state.isLoggedIn || !raw.length) return raw
  void anonProgressVersion.value
  const cid = props.collectionId
  return raw.map((level) => {
    const local = getProgress(cid, level.level_id) as LevelProgressData | null
    const hasLocal = local && (local.cards_completed > 0 || local.total_answers > 0)
    // Guests: unlock from localStorage prerequisites (API has no anon progress).
    const isLocked = !isLevelUnlockedForAnon(cid, level)
    const progress = hasLocal
      ? {
          cards_completed: local.cards_completed ?? 0,
          correct_answers: local.correct_answers ?? 0,
          total_answers: local.total_answers ?? 0,
          is_unlocked: !isLocked,
          is_completed: !!local.completed_at,
        }
      : {
          cards_completed: 0,
          correct_answers: 0,
          total_answers: 0,
          is_unlocked: !isLocked,
          is_completed: false,
        }
    return { ...level, is_locked: isLocked, progress }
  })
})

// Methods
const fetchCollection = async () => {
  try {
    const response = await getCollection(props.collectionId)
    collection.value = response.data
  } catch (error) {
    console.error('Error fetching collection:', error)
  }
}

const fetchLevels = async () => {
  try {
    const response = await getLevels(props.collectionId)
    levels.value = response.data.levels
  } catch (error) {
    console.error('Error fetching levels:', error)
  } finally {
    isLoading.value = false
  }
}

const editLevel = (level) => {
  editingLevel.value = level
  showLevelModal.value = true
}

const openCreateLevel = () => {
  editingLevel.value = null
  showLevelModal.value = true
}

async function onLevelChanged() {
  showLevelModal.value = false
  await fetchLevels()
}

/** Cards of a level are managed (owner) or viewed (others) in the collection's course-order view. */
const cardsLink = (level: { level_id: number }) => ({
  path: `/collections/${props.collectionId}`,
  query: { view: 'course-order' },
  hash: `#level-${level.level_id}`,
})

// Level is playable when backend says not locked (e.g. no prerequisites) or user progress has unlocked_at set
const isLevelUnlocked = (level) => level && (!level.is_locked || !!level.progress?.is_unlocked)

const getLevelCardClass = (level) => {
  if (level?.progress?.is_completed) return 'level-card--completed'
  if (isLevelUnlocked(level)) return 'level-card--unlocked'
  return 'level-card--locked'
}

const getProgressWidth = (level) => {
  if (!level.card_count) return '0%'
  const progress = (level.progress?.cards_completed || 0) / level.card_count
  return `${Math.round(progress * 100)}%`
}

/** Level is “finished” at least 90% (for strength badge). */
const isLevelFinished90 = (level) => {
  if (level?.progress?.is_completed) return true
  if (!level?.card_count) return false
  const p = (level.progress?.cards_completed || 0) / level.card_count
  return p >= 0.9
}

const formatSuccessRate = (rate) => {
  if (!rate) return '0%'
  return `${Math.round(rate * 100)}%`
}

/** Hash level name to a hue 0–360 for consistent per-level color. */
const levelNameToHue = (name) => {
  if (!name || typeof name !== 'string') return 220
  let h = 0
  for (let i = 0; i < name.length; i++) {
    h = (h << 5) - h + name.charCodeAt(i)
    h |= 0
  }
  return Math.abs(h) % 360
}

/** CSS custom properties for card theming by level name (hue) and state. */
const getLevelCardStyle = (level) => {
  const hue = levelNameToHue(level?.name)
  const completed = level?.progress?.is_completed
  const unlocked = isLevelUnlocked(level)
  const locked = !unlocked && !completed
  return {
    '--level-hue': String(hue),
    '--level-sat': locked ? '25%' : '55%',
    '--level-light': completed ? '52%' : unlocked ? '60%' : '75%',
    '--level-bg-from': locked ? `hsl(${hue}, 15%, 96%)` : `hsl(${hue}, 40%, 97%)`,
    '--level-bg-to': locked ? 'hsl(0, 0%, 100%)' : `hsl(${hue}, 25%, 99%)`,
    '--level-accent': locked ? 'hsl(0, 0%, 75%)' : `hsl(${hue}, 55%, 50%)`,
    '--level-glow': locked ? 'transparent' : `hsla(${hue}, 60%, 50%, 0.25)`,
  }
}

const getTreeLevel = (level) => {
  if (!level || !level.prerequisites || !level.prerequisites.length) return 0
  return Math.max(...level.prerequisites.map((p) => getTreeLevel(p))) + 1
}

const elements = ref<Elements>([])
const { setElements, fitView } = useVueFlow()

// Next available unlocked level (first unlocked and not completed) — center and zoom into it on open
const getNextUnlockedLevelId = (levelsData) => {
  if (!levelsData?.length) return null
  const sorted = [...levelsData].sort((a, b) => a.position - b.position)
  const next = sorted.find((l) => isLevelUnlocked(l) && !l.progress?.is_completed)
  return next ? next.level_id.toString() : null
}

// Inset so level cards never touch the flow area borders (hover scale/shadow have room)
const FLOW_PADDING = 12

// Vertical spacing between level cards (enough to avoid overlap with variable card height and hover lift)
const CARD_VERTICAL_GAP = 380

// Convert levels to Vue Flow elements (vertical layout: levels stacked top-to-bottom, depth as horizontal offset)
const convertLevelsToElements = (levelsData) => {
  const sorted = [...levelsData].sort((a, b) => a.position - b.position)
  const nodes = sorted.map((level, index) => ({
    id: level.level_id.toString(),
    type: 'custom',
    position: {
      x: getTreeLevel(level) * 220 + FLOW_PADDING,
      y: index * CARD_VERTICAL_GAP + FLOW_PADDING,
    },
    data: { ...level, levelIndex: index + 1 },
  }))

  const edges = levelsData.flatMap((level) =>
    level.prerequisites.map((prereq) => ({
      id: `e${prereq.level_id}-${level.level_id}`,
      source: prereq.level_id.toString(),
      target: level.level_id.toString(),
      animated: true,
      style: { stroke: '#94a3b8' },
    }))
  )

  return [...nodes, ...edges]
}

// Only run initial scroll once so we never get a second fitView that zooms out to fit all
let initialScrollScheduled = false

// Center viewport on next available unlocked level and zoom into it (runs once after first load)
const scrollToNextUnlocked = (levelsData) => {
  const nodeId = getNextUnlockedLevelId(levelsData)
  if (!nodeId) return // never call fitView() for "all" — that would zoom out and override our zoom-to-node
  if (initialScrollScheduled) return
  initialScrollScheduled = true
  nextTick(() => {
    setTimeout(() => {
      fitView({ nodes: [nodeId], padding: 0.25, duration: 400 })
    }, 150)
  })
}

// Watch for changes in levels (and effectiveLevels for anon) and update Vue Flow elements
watch(
  () => effectiveLevels.value,
  (newLevels) => {
    if (newLevels?.length) {
      const next = convertLevelsToElements(newLevels)
      // Keep v-model `elements` in sync; setElements alone can leave the bound array empty.
      elements.value = next
      setElements(next)
      scrollToNextUnlocked(newLevels)
    } else {
      elements.value = []
      setElements([])
    }
  },
  { immediate: true }
)

useSeoHead({
  title: t('flashcardLevels.title', { collectionName: collection.value?.name || '' }),
})

onMounted(async () => {
  await Promise.all([fetchCollection(), fetchLevels()])
})
</script>

<style scoped>
.vue-flow-wrapper {
  width: 100%;
  height: 100%;
}

/* Internal padding so cards don't touch Vue Flow area borders */
.vue-flow-wrapper :deep(.vue-flow__viewport) {
  padding: 0.5rem;
  /* p-2 equivalent */
  box-sizing: border-box;
}

/* ---- Level card: game-style with hue from level name ---- */
.level-card {
  --level-hue: 220;
  --level-bg-from: hsl(220, 25%, 97%);
  --level-bg-to: hsl(220, 15%, 99%);
  --level-accent: hsl(220, 55%, 50%);
  --level-glow: hsla(220, 60%, 50%, 0.2);

  width: 300px;
  max-width: calc(100vw - 2rem);
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 1rem;
  box-sizing: border-box;
  position: relative;
  overflow: hidden;
  border-radius: 1rem;
  border: 2px solid var(--level-accent);
  background: linear-gradient(165deg, var(--level-bg-from) 0%, var(--level-bg-to) 100%);
  box-shadow:
    0 4px 6px -1px rgba(0, 0, 0, 0.08),
    0 2px 4px -2px rgba(0, 0, 0, 0.05),
    0 0 0 1px rgba(0, 0, 0, 0.03);
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease,
    filter 0.2s ease;
}

.level-card--locked {
  filter: saturate(0.75) brightness(0.98);
  opacity: 0.92;
}

.level-card--unlocked:hover {
  transform: translateY(-3px) scale(1.01);
  box-shadow:
    0 12px 24px -8px rgba(0, 0, 0, 0.15),
    0 0 20px -4px var(--level-glow);
}

.level-card--completed {
  border-width: 2px;
  box-shadow:
    0 8px 20px -6px rgba(0, 0, 0, 0.12),
    0 0 16px -4px var(--level-glow);
}

.level-card-glow {
  position: absolute;
  inset: -20px;
  background: radial-gradient(ellipse 80% 50% at 50% -20%, var(--level-glow), transparent 60%);
  pointer-events: none;
  opacity: 0.6;
}

.level-card-number {
  flex: none;
  width: 2rem;
  height: 2rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--level-accent);
  color: white;
  font-size: 0.8125rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
}

.level-card-badge {
  flex: none;
  width: 2.25rem;
  height: 2.25rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.75rem;
  background: var(--level-accent);
  color: white;
  box-shadow: 0 3px 8px rgba(0, 0, 0, 0.2);
}

.level-card-badge-icon {
  width: 1.125rem;
  height: 1.125rem;
}

.level-card-head {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
}

.level-card-heading {
  flex: 1 1 auto;
  min-width: 0;
}

.level-card-title {
  font-size: 1.0625rem;
  font-weight: 700;
  letter-spacing: -0.02em;
  line-height: 1.3;
  color: #1e293b;
  overflow-wrap: anywhere;
}

.level-card--locked .level-card-title {
  color: #64748b;
}

.level-card-desc {
  margin-top: 0.25rem;
  font-size: 0.8125rem;
  line-height: 1.4;
  color: #475569;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.level-card--locked .level-card-desc {
  color: #94a3b8;
}

.level-card-meta {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.level-card-hint {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8125rem;
  color: #64748b;
}

.level-card-progress-head {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.75rem;
  font-weight: 600;
  color: #475569;
}

.level-card-progress-count {
  margin-left: auto;
  color: #334155;
}

.level-card-progress-track {
  height: 6px;
  width: 100%;
  border-radius: 9999px;
  background: #e2e8f0;
  overflow: hidden;
}

.level-card-progress-fill {
  height: 100%;
  border-radius: 9999px;
  background: var(--level-accent);
  transition: width 0.3s ease;
}

.level-card-stats {
  display: flex;
  justify-content: space-between;
  font-size: 0.6875rem;
  color: #64748b;
}

.level-card-actions {
  position: relative;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

.level-card-actions--owner {
  padding-top: 0.5rem;
  border-top: 1px solid rgba(15, 23, 42, 0.08);
  gap: 0.25rem;
}

.level-card-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.8125rem;
  font-weight: 600;
  border-radius: 0.75rem;
  transition:
    background 0.15s,
    color 0.15s,
    box-shadow 0.15s;
  border: none;
  cursor: pointer;
  text-decoration: none;
}

.level-card-btn-primary {
  background: var(--level-accent);
  color: white;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.level-card-btn-primary:hover {
  filter: brightness(1.08);
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.2);
}

.level-card-btn-secondary {
  background: white;
  color: #334155;
  border: 2px solid #cbd5e1;
}

.level-card-btn-secondary:hover {
  border-color: var(--level-accent);
  background: color-mix(in hsl, var(--level-accent) 12%, white);
  color: #0f172a;
}

.level-card-btn-ghost {
  background: transparent;
  color: #64748b;
}

.level-card-btn-ghost:hover {
  background: #f1f5f9;
  color: #334155;
}

.level-card-completed {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--level-accent);
}
</style>
