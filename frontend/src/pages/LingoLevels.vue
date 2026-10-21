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
              <span class="hidden sm:inline">{{ t('flashcardLevels.studyButton') }}</span>
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
              :to="flashcardsPath"
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
        <div class="relative mt-4 flex w-full flex-col items-center">
          <LingoLessonButton
            v-for="(level, i) in displayLevels"
            :key="level.level_id"
            :index="i"
            :total-count="displayLevels.length - 1"
            :locked="!isLevelUnlocked(level) && !level.progress?.is_completed"
            :current="level === activeLevel"
            :completed="!!level.progress?.is_completed"
            :percentage="
              activeLevel && level.level_id === activeLevel.level_id ? activeLevelPercentage : 0
            "
            :study-url="studyUrlFor(level)"
            :start-label="t('flashcardLevels.start')"
            :show-owner-menu="false"
          />
        </div>
      </div>
    </LingoFeedLayout>
  </LingoLayout>
</template>

<script setup lang="ts">
import { ArrowLeft, Dumbbell, GalleryVerticalIcon, Pencil } from '@lucide/vue'
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import LingoFeedLayout from '@/components/LingoFeedLayout.vue'
import LingoUnitBanner from '@/components/LingoUnitBanner.vue'
import LingoLessonButton from '@/components/LingoLessonButton.vue'
import LingoUserProgress from '@/components/LingoUserProgress.vue'
import { useLingoProgress } from '@/composables/useLingoProgress'
import { getCollection, getLevels } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useAnonymousProgress, type LevelProgressData } from '@/composables/useAnonymousProgress'
import { useSeoHead } from '@/composables/useSeoHead'

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
const { getProgress } = useAnonymousProgress()
const { setActiveCourse } = useLingoProgress()

const collection = ref(null)
const levels = ref([])
const isLoading = ref(true)

const flashcardsPath = computed(() => `/collections/${props.collectionId}/flashcards`)

/** Collection reels (tiktoknu) — browse cards; `from=lingo` sends back here. */
const reelsPath = computed(() => ({
  path: `/collections/${props.collectionId}/tiktoknu`,
  query: { from: 'lingo' },
}))

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
  return (
    displayLevels.value.find((l) => isLevelUnlocked(l) && !l.progress?.is_completed) ??
    displayLevels.value[0] ??
    null
  )
})

const activeLevelPercentage = computed(() => {
  const level = activeLevel.value
  if (!level?.card_count) return 0
  const done = level.progress?.cards_completed ?? 0
  return Math.round((done / level.card_count) * 100)
})

function isLevelUnlockedForAnon(level) {
  if (!level?.prerequisites?.length) return true
  const cid = props.collectionId
  return level.prerequisites.every((p) => {
    const local = getProgress(cid, p.level_id) as LevelProgressData | null
    return local?.completed_at != null
  })
}

const effectiveLevels = computed(() => {
  const raw = levels.value
  if (auth.state.isLoggedIn || !raw.length) return raw
  const cid = props.collectionId
  return raw.map((level) => {
    const local = getProgress(cid, level.level_id) as LevelProgressData | null
    const hasLocal = local && (local.cards_completed > 0 || local.total_answers > 0)
    const isLocked = hasLocal ? !isLevelUnlockedForAnon(level) : level.is_locked
    const progress = hasLocal
      ? {
          cards_completed: local.cards_completed ?? 0,
          correct_answers: local.correct_answers ?? 0,
          total_answers: local.total_answers ?? 0,
          is_unlocked: !isLocked,
          is_completed: !!local.completed_at,
        }
      : level.progress
    return { ...level, is_locked: isLocked, progress }
  })
})

const isLevelUnlocked = (level) => level && (!level.is_locked || !!level.progress?.is_unlocked)

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
    await Promise.all([fetchCollection(), fetchLevels()])
  } finally {
    isLoading.value = false
  }
})
</script>
