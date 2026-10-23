<template>
  <LingoLayout fill>
    <!-- Wait for collection + levels before painting the feed (avoids header/body CLS). -->
    <div v-if="isLoading" class="flex min-h-0 flex-1 items-center justify-center py-12">
      <div
        class="h-10 w-10 animate-spin rounded-full border-2 border-green-500 border-t-transparent"
        role="status"
        :aria-label="t('components.loadingSpinner.loadingDefault')"
      />
    </div>

    <LingoFeedLayout v-else :show-aside="false" class="min-h-0 flex-1">
      <template #header>
        <!-- div not header: global App.vue `header` stripes belong on `__header-band`. -->
        <div class="lingo-levels-header">
          <RouterLink
            to="/lingo/courses"
            class="lingo-icon-btn lingo-icon-btn--muted"
            :aria-label="t('flashcardLevels.backToCourses')"
          >
            <ArrowLeft class="h-5 w-5" />
          </RouterLink>
          <div class="flex shrink-0 items-center gap-2">
            <button
              type="button"
              class="lingo-btn lingo-btn--secondary lingo-btn--sm"
              @click="startStudy"
            >
              <Dumbbell class="h-4 w-4 shrink-0" />
              <span>{{ t('flashcardLevels.studyButton') }}</span>
            </button>
            <RouterLink
              :to="reelsPath"
              class="lingo-btn lingo-btn--ghost lingo-btn--sm"
              :aria-label="t('collectionDetail.viewAsTiktoknu')"
            >
              <GalleryVerticalIcon class="h-4 w-4 shrink-0" aria-hidden="true" />
              <span class="hidden sm:inline">{{ t('collectionDetail.viewAsTiktoknu') }}</span>
            </RouterLink>
            <LingoUserProgress />
            <RouterLink
              v-if="isOwner"
              :to="editPath"
              class="lingo-btn lingo-btn--ghost lingo-btn--sm"
              :aria-label="t('flashcardLevels.editCourse')"
            >
              <Pencil class="h-4 w-4 shrink-0" />
              <span class="hidden sm:inline">{{ t('flashcardLevels.editCourse') }}</span>
            </RouterLink>
          </div>
        </div>
      </template>

      <div class="lingo-levels-root relative flex w-full flex-col items-center pb-16">
        <LingoUnitBanner
          :title="collection?.name || ''"
          :description="collection?.description || ''"
          :continue-label="t('flashcardLevels.continueLabel')"
          @continue="startStudy"
        />
        <LingoGuestSaveAlert class="mt-3 w-full" :return-to="levelsReturnTo" />
        <div class="relative mt-4 flex w-full flex-col items-center">
          <LingoLessonButton
            v-for="(level, i) in displayLevels"
            :key="level.level_id"
            :index="i"
            :total-count="displayLevels.length - 1"
            :level-id="level.level_id"
            :locked="!isLevelUnlocked(level) && !level.progress?.is_completed"
            :current="level === activeLevel"
            :completed="!!level.progress?.is_completed"
            :percentage="
              activeLevel && level.level_id === activeLevel.level_id ? activeLevelPercentage : 0
            "
            :study-url="studyUrlFor(level)"
            :start-label="t('flashcardLevels.start')"
            :label="level.name || ''"
            :show-owner-menu="false"
          />
        </div>
      </div>
    </LingoFeedLayout>
  </LingoLayout>
</template>

<script setup lang="ts">
import { ArrowLeft, Dumbbell, GalleryVerticalIcon, Pencil } from '@lucide/vue'
import { ref, computed, nextTick, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import LingoFeedLayout from '@/components/LingoFeedLayout.vue'
import LingoUnitBanner from '@/components/LingoUnitBanner.vue'
import LingoLessonButton from '@/components/LingoLessonButton.vue'
import LingoUserProgress from '@/components/LingoUserProgress.vue'
import LingoGuestSaveAlert from '@/components/LingoGuestSaveAlert.vue'
import { useLingoProgress } from '@/composables/useLingoProgress'
import { getCollection, getLevels } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useAnonymousProgress, type LevelProgressData } from '@/composables/useAnonymousProgress'
import { ensureGuestSession, hasGuestSession } from '@/composables/useGuestSession'
import { useLocalePath } from '@/composables/useLocalePath'
import { useSeoHead } from '@/composables/useSeoHead'

const props = defineProps({
  collectionId: {
    type: [String, Number],
    required: true,
    validator: (value) => !isNaN(Number(value)),
  },
})

const auth = useAuth()
const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const localePath = useLocalePath()
const {
  getProgress,
  isLevelUnlockedForAnon,
  resolveAndRememberActiveLevelId,
  getActiveLevelId,
  anonProgressVersion,
} = useAnonymousProgress()
const { setActiveCourse } = useLingoProgress()

const collection = ref(null)
const levels = ref([])
const isLoading = ref(true)

/** Owner edit: level ladder when the course has levels, otherwise the card list. */
const editPath = computed(() => {
  const id = props.collectionId
  const path =
    levels.value.length > 0 ? `/collections/${id}/levels` : `/collections/${id}/flashcards`
  return localePath(path)
})

/** Collection reels (tiktoknu) — browse cards; `from=lingo` sends back here. */
const reelsPath = computed(() => ({
  path: `/collections/${props.collectionId}/tiktoknu`,
  query: { from: 'lingo' },
}))

/** After guest login/signup, return to this course tree. */
const levelsReturnTo = computed(() => `/collections/${props.collectionId}/lingo/levels`)

const isOwner = computed(
  () => auth.state.isLoggedIn && collection.value?.owner?.username === auth.state.username
)

/** When the collection has no levels, treat the whole course as one playable lesson. */
const IMPLICIT_COURSE_LEVEL_ID = 'course'

function startStudy() {
  const target = activeLevel.value
  router.push(studyUrlFor(target ?? displayLevels.value[0]))
}

function studyUrlFor(level) {
  if (!level || level.level_id === IMPLICIT_COURSE_LEVEL_ID) {
    return `/collections/${props.collectionId}/lingo/study`
  }
  return `/collections/${props.collectionId}/lingo/study?levelId=${level.level_id}`
}

const sortedEffectiveLevels = computed(() => {
  return [...effectiveLevels.value].sort((a, b) => a.position - b.position)
})

/** Ladder rows: real levels, or a single implicit course lesson. */
const displayLevels = computed(() => {
  if (sortedEffectiveLevels.value.length > 0) return sortedEffectiveLevels.value
  return [
    {
      level_id: IMPLICIT_COURSE_LEVEL_ID,
      name: collection.value?.name || '',
      position: 0,
      is_locked: false,
      card_count: 0,
      progress: {
        cards_completed: 0,
        is_unlocked: true,
        is_completed: false,
      },
    },
  ]
})

const activeLevel = computed(() => {
  const list = displayLevels.value
  if (!list.length) return null
  // Logged-in / backend guest: first unlocked incomplete from API progress.
  if (auth.state.isLoggedIn || hasGuestSession()) {
    return list.find((l) => isLevelUnlocked(l) && !l.progress?.is_completed) ?? list[0] ?? null
  }
  // Legacy localStorage guests: prefer remembered active level if still unlocked.
  void anonProgressVersion.value
  const real = list.filter((l) => l.level_id !== IMPLICIT_COURSE_LEVEL_ID)
  if (real.length) {
    const activeId = getActiveLevelId(props.collectionId)
    const remembered = real.find(
      (l) => Number(l.level_id) === activeId && isLevelUnlocked(l) && !l.progress?.is_completed
    )
    if (remembered) return remembered
  }
  return list.find((l) => isLevelUnlocked(l) && !l.progress?.is_completed) ?? list[0] ?? null
})

const activeLevelPercentage = computed(() => {
  const level = activeLevel.value
  if (!level?.card_count) return 0
  const done = level.progress?.cards_completed ?? 0
  return Math.round((done / level.card_count) * 100)
})

const effectiveLevels = computed(() => {
  const raw = levels.value
  // Logged-in or backend guest session: trust API lock/progress (single mechanism).
  if (auth.state.isLoggedIn || hasGuestSession() || !raw.length) return raw
  void anonProgressVersion.value
  const cid = props.collectionId
  return raw.map((level) => {
    const local = getProgress(cid, level.level_id) as LevelProgressData | null
    const hasLocal = local && (local.cards_completed > 0 || local.total_answers > 0)
    // Legacy localStorage unlock when no guest JWT.
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

const isLevelUnlocked = (level) => level && (!level.is_locked || !!level.progress?.is_unlocked)

// Persist the prereq-aware active level for guests (side effect kept out of computed).
watch(
  effectiveLevels,
  (list) => {
    if (auth.state.isLoggedIn || !list.length) return
    resolveAndRememberActiveLevelId(props.collectionId, list)
  },
  { immediate: true }
)

/** Center the next required level (from study finish `?focusLevelId=`) or the active node. */
function scrollFocusLevelIntoView() {
  const raw = route.query.focusLevelId
  const fromQuery = Array.isArray(raw) ? raw[0] : raw
  const focusId =
    fromQuery != null && String(fromQuery) !== ''
      ? String(fromQuery)
      : activeLevel.value?.level_id != null
        ? String(activeLevel.value.level_id)
        : ''
  if (!focusId) return
  // First two ladder rows are already near the top; leave the feed at the tree start.
  const focusIndex = displayLevels.value.findIndex((l) => String(l.level_id) === focusId)
  if (focusIndex >= 0 && focusIndex < 2) return
  nextTick(() => {
    const el = document.querySelector(`[data-level-id="${CSS.escape(focusId)}"]`)
    el?.scrollIntoView({ block: 'center', behavior: 'smooth' })
  })
}

const fetchCollection = async () => {
  try {
    const response = await getCollection(props.collectionId)
    collection.value = response.data
    setActiveCourse(Number(props.collectionId), response.data?.name || '')
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
  }
}

useSeoHead({
  title: computed(() =>
    collection.value?.name
      ? t('flashcardLevels.title', { collectionName: collection.value.name })
      : t('lingo.courses')
  ),
})

onMounted(async () => {
  try {
    if (!auth.state.isLoggedIn) {
      await ensureGuestSession()
    }
    await Promise.all([fetchCollection(), fetchLevels()])
  } finally {
    isLoading.value = false
    scrollFocusLevelIntoView()
  }
})
</script>
