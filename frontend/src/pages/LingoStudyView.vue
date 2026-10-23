<template>
  <div class="lingo-study-root flex h-full min-h-0 flex-col overflow-hidden bg-slate-50">
    <!-- Full-bleed header stripe; inner chrome matches the lesson column width -->
    <LingoStudyHeader
      v-if="(currentCard || outOfHearts) && !isLoading"
      :percentage="progressPercentage"
      @exit="handleExit"
    >
      <template #extra>
        <!-- Same compact chrome as the exit control (`icon-btn-header-back--compact`). -->
        <LingoUserProgress class="icon-btn-header-back--compact !min-w-9" />
      </template>
    </LingoStudyHeader>

    <AnonymousProgressBanner
      v-if="cardsAnsweredInSession >= ANON_BANNER_AFTER_ANSWERS"
      variant="lingo"
      position="inline"
    />

    <!-- Lesson column (card body only) -->
    <div
      class="mx-auto flex h-full w-full max-w-[600px] flex-1 flex-col min-h-0 overflow-hidden px-2 sm:px-3"
    >
      <!-- Loading State -->
      <div v-if="isLoading" class="flex flex-1 items-center justify-center min-h-0 py-4">
        <div
          class="animate-spin rounded-full h-6 w-6 border-2 border-green-500 border-t-transparent"
        />
      </div>
      <!-- Out of hearts -->
      <div
        v-else-if="outOfHearts"
        class="flex flex-1 flex-col items-center justify-center gap-3 min-h-0 py-4 text-center px-4"
      >
        <Heart class="h-14 w-14 fill-rose-500 text-rose-500" />
        <h1 class="text-lg font-bold text-neutral-700">{{ t('lingo.outOfHearts') }}</h1>
        <p class="max-w-sm text-sm text-slate-500">{{ t('lingo.outOfHeartsHint') }}</p>
        <RouterLink
          to="/lingo/shop"
          class="lingo-btn lingo-btn--secondary lingo-btn--lg !normal-case !tracking-normal"
        >
          {{ t('lingo.goToShop') }}
        </RouterLink>
      </div>
      <!-- Lesson finish (clone quiz.tsx: finish art + XP/hearts + continue to path) -->
      <div
        v-else-if="!currentCard && remainingCards.length === 0"
        class="flex flex-1 flex-col items-center justify-center gap-3 min-h-0 py-4 text-center sm:gap-4"
      >
        <div class="text-5xl sm:text-6xl" aria-hidden="true">🎉</div>
        <h1 class="text-lg font-bold text-neutral-700 sm:text-2xl">
          <template v-if="levelIdParam != null && cardsAnsweredInSession > 0">
            {{ t('flashcardStudy.lessonCompleteTitle') }} <br />
            {{ t('flashcardStudy.lessonCompleteBody') }}
          </template>
          <template v-else>
            {{ t('flashcardStudy.allCaughtUp') }} <br />
            {{ t('flashcardStudy.allReviewed') }}
          </template>
        </h1>

        <div class="flex w-full max-w-xs flex-col gap-2 sm:flex-row sm:max-w-sm justify-center">
          <LingoResultCard variant="points" :value="sessionXpAwarded" />
          <LingoResultCard variant="hearts" :value="hasUnlimitedHearts ? Infinity : hearts" />
        </div>
        <button
          ref="returnToDeckButtonRef"
          type="button"
          class="lingo-btn lingo-btn--secondary lingo-btn--lg !normal-case !tracking-normal min-w-[140px]"
          @click="router.push(returnToUrl)"
        >
          {{
            levelIdParam != null
              ? t('flashcardStudy.continueToLevels')
              : t('flashcardStudy.returnToDeck')
          }}
        </button>
      </div>
      <!-- Lesson card (footer is full-bleed below this column) -->
      <div v-else-if="currentCard" class="flex flex-1 flex-col min-h-0 overflow-hidden">
        <!-- Top-anchored (not centered) so answer reveal does not shove the question. -->
        <div class="flex flex-1 min-h-0 items-center justify-center overflow-hidden">
          <div class="lingo-study-lesson-stack overflow-auto">
            <!-- Clone lesson title: large; stimulus below is smaller in a bubble -->
            <h1 v-if="lessonPrompt" class="lingo-study-lesson-title">
              {{ lessonPrompt }}
            </h1>

            <div
              v-if="currentCard.flashcard.definition_language_id"
              class="text-[10px] text-gray-600 text-center sm:text-xs lg:text-start"
            >
              {{
                t('flashcardStudy.definitionLanguage', {
                  language: getLanguageName(currentCard.flashcard.definition_language_id),
                })
              }}
            </div>

            <!-- Stimulus: question speaker + bubble (caret sits in the gap). -->
            <div class="lingo-study-question-row">
              <div
                class="lingo-study-mascot"
                role="img"
                aria-hidden="true"
                :style="{ backgroundImage: `url(${studyQuestionCharacterSrc})` }"
              />
              <div class="lingo-study-question-bubble">
                <div
                  v-if="currentCard.progress[0].card_side === 'direct'"
                  class="flex items-start justify-between gap-2"
                >
                  <p class="lingo-study-stimulus min-w-0 flex-1 [overflow-wrap:anywhere]">
                    {{
                      currentCard.flashcard.question_text ??
                      currentCard.flashcard.word ??
                      currentCard.flashcard.free_content_front
                    }}
                  </p>
                  <AudioPlayer
                    v-if="currentCard.flashcard.sound_url"
                    :key="'q-' + currentCard.flashcard.id"
                    ref="questionAudioPlayerRef"
                    class="shrink-0"
                    :url="currentCard.flashcard.sound_url"
                    :collection-id="
                      currentCard.flashcard.sound_url?.startsWith?.('/api/')
                        ? currentCard.flashcard.collection_id
                        : undefined
                    "
                    :item-id="
                      currentCard.flashcard.sound_url?.startsWith?.('/api/')
                        ? currentCard.flashcard.item_id
                        : undefined
                    "
                    :suppress-play-errors="true"
                  />
                </div>
                <div v-else class="lingo-study-stimulus prose prose-sm max-w-none">
                  <LazyMathJax
                    :content="
                      currentCard.flashcard.question_text ??
                      currentCard.flashcard.definition ??
                      currentCard.flashcard.free_content_back
                    "
                  />
                </div>
              </div>
            </div>

            <div
              v-if="
                currentCard.progress[0].card_side === 'direct' &&
                currentCard.flashcard.has_front_image
              "
              class="flex justify-center"
            >
              <img
                :src="`/api/collections/${currentCard.flashcard.collection_id}/items/${currentCard.flashcard.item_id}/image/front`"
                class="max-h-20 rounded object-contain bg-gray-100 sm:max-h-24"
                alt=""
              />
            </div>
            <div
              v-else-if="
                currentCard.progress[0].card_side !== 'direct' &&
                currentCard.flashcard.has_back_image
              "
              class="flex justify-center"
            >
              <img
                :src="`/api/collections/${currentCard.flashcard.collection_id}/items/${currentCard.flashcard.item_id}/image/back`"
                class="max-h-20 rounded object-contain bg-gray-100 sm:max-h-24"
                alt=""
              />
            </div>

            <!-- Reply: empty/fill-in bubble + speaker on the right (mirrored). -->
            <div class="lingo-study-question-row lingo-study-question-row--reply">
              <div
                class="lingo-study-question-bubble"
                :class="showAnswerBubble ? 'lingo-study-answer-bubble' : 'lingo-study-reply-bubble'"
                :aria-label="showAnswerBubble ? undefined : t('flashcardStudy.typeAnswer')"
              >
                <template v-if="showAnswerBubble">
                  <p class="lingo-study-answer-label">{{ t('flashcardStudy.correctAnswer') }}</p>
                  <div class="lingo-study-stimulus prose prose-sm max-w-none">
                    <template v-if="currentCard.progress[0].card_side === 'direct'">
                      <LazyMathJax
                        :content="
                          currentCard.flashcard.definition ??
                          currentCard.flashcard.free_content_back
                        "
                      />
                      <div
                        v-if="currentCard.flashcard.has_back_image"
                        class="mt-1 flex justify-center"
                      >
                        <img
                          :src="`/api/collections/${currentCard.flashcard.collection_id}/items/${currentCard.flashcard.item_id}/image/back`"
                          class="max-h-20 rounded object-contain bg-gray-100 sm:max-h-24"
                          alt=""
                        />
                      </div>
                    </template>
                    <template v-else>
                      <div class="flex items-center gap-1.5">
                        <span>{{
                          currentCard.flashcard.word ?? currentCard.flashcard.free_content_front
                        }}</span>
                        <AudioPlayer
                          v-if="currentCard.flashcard.sound_url"
                          ref="answerAudioPlayerRef"
                          :url="currentCard.flashcard.sound_url"
                          :collection-id="
                            currentCard.flashcard.sound_url?.startsWith?.('/api/')
                              ? currentCard.flashcard.collection_id
                              : undefined
                          "
                          :item-id="
                            currentCard.flashcard.sound_url?.startsWith?.('/api/')
                              ? currentCard.flashcard.item_id
                              : undefined
                          "
                          :suppress-play-errors="true"
                        />
                      </div>
                      <div
                        v-if="showCanonicalForm"
                        class="mt-1 flex flex-col gap-0.5 border-t border-green-100 pt-1"
                      >
                        <div
                          class="flex items-center gap-1 text-[10px] font-semibold uppercase tracking-wider text-gray-400"
                        >
                          <EqualApproximately class="h-3 text-blue-400" />
                          <span>{{ t('components.definitionCard.canonicalLabel') }}</span>
                        </div>
                        <div class="code-snippet-inline">
                          {{ currentCard.flashcard.canonical_form }}
                        </div>
                      </div>
                      <div
                        v-if="currentCard.flashcard.has_front_image"
                        class="mt-1 flex justify-center"
                      >
                        <img
                          :src="`/api/collections/${currentCard.flashcard.collection_id}/items/${currentCard.flashcard.item_id}/image/front`"
                          class="max-h-20 rounded object-contain bg-gray-100 sm:max-h-24"
                          alt=""
                        />
                      </div>
                    </template>
                  </div>
                  <div
                    v-if="
                      currentCard.flashcard.notes && currentCard.progress[0].card_side === 'direct'
                    "
                    class="mt-1.5 border-t border-green-100 pt-1.5 text-[10px] sm:text-xs"
                  >
                    <h4 class="mb-0.5 font-medium text-gray-700">
                      {{ t('flashcardStudy.notes') }}
                    </h4>
                    <LazyMathJax :content="currentCard.flashcard.notes" :enable-markdown="true" />
                  </div>
                </template>
                <template v-else-if="isFillInMode">
                  <Textarea
                    ref="fillInTextareaRef"
                    v-model="userAnswer"
                    type="text"
                    rows="2"
                    class="lingo-study-reply-input"
                    :placeholder="t('flashcardStudy.typeAnswer')"
                    :disabled="!!fillinResult"
                    @keydown.enter.prevent="() => submitAnswer()"
                  />
                </template>
                <!-- Empty two-line invite for recall / just-info (Check fills this slot). -->
                <div v-else class="lingo-study-reply-invite" aria-hidden="true"><br /><br /></div>
              </div>
              <div
                class="lingo-study-mascot"
                role="img"
                aria-hidden="true"
                :style="{ backgroundImage: `url(${studyAnswerCharacterSrc})` }"
              />
            </div>

            <div
              v-if="isFillInMode && currentCard.flashcard.use_canonical_comparison"
              class="badge-streak-success--compact"
            >
              <CheckCircle2 class="h-3 w-3" />
              <span>{{ t('flashcardStudy.canonicalComparisonEnabled') }}</span>
            </div>

            <!-- Quiz options / result (fill-in lives in the reply bubble above). -->
            <div v-if="showStudyCardShell" class="lingo-study-card-shell">
              <div class="flex min-h-0 flex-col gap-1">
                <div
                  v-if="
                    isQuizMode &&
                    !quizResult &&
                    (currentCard.flashcard.quiz_options?.length || quizImageOptions?.length)
                  "
                >
                  <div class="grid grid-cols-2 gap-1.5 sm:gap-2">
                    <template v-if="quizImageOptions?.length">
                      <Button
                        v-for="opt in quizImageOptions"
                        :key="opt.id"
                        variant="neutral"
                        type="button"
                        class="study-quiz-option-lingo-image"
                        :disabled="isSubmitting"
                        @click="submitQuizOption(opt.id)"
                      >
                        <img
                          :src="opt.imageUrl"
                          class="h-16 w-full rounded-lg object-contain sm:h-20"
                          :alt="opt.id"
                        />
                      </Button>
                    </template>
                    <Button
                      v-for="(opt, idx) in currentCard.flashcard.quiz_options"
                      v-else
                      :key="idx"
                      variant="neutral"
                      type="button"
                      class="study-quiz-option-lingo-text"
                      :disabled="isSubmitting"
                      @click="submitQuizOption(opt)"
                    >
                      <LazyMathJax :content="opt" />
                    </Button>
                  </div>
                </div>
                <div v-if="quizResult" class="flex flex-col gap-1">
                  <div
                    class="text-center text-sm font-semibold"
                    :class="quizResult.correct ? 'text-green-600' : 'text-red-600'"
                  >
                    {{
                      quizResult.correct
                        ? t('flashcardStudy.answerCorrect')
                        : t('flashcardStudy.answerIncorrect')
                    }}
                  </div>
                  <p
                    v-if="!quizResult.correct && quizResult.message"
                    class="text-center text-xs text-gray-600"
                  >
                    {{ quizResult.message }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Single footer instance — status swaps inside a fixed-height chrome box. -->
    <LingoStudyFooter
      v-if="currentCard && !isLoading && !outOfHearts"
      :status="studyFooterStatus"
      :disabled="studyFooterDisabled"
      :compact="true"
      :show-card-actions="true"
      :snooze-disabled="isSubmitting"
      :correct-label="t('flashcardStudy.nicelyDone')"
      :wrong-label="t('flashcardStudy.tryAgain')"
      :rate-prompt="t('flashcardStudy.howWellRemembered')"
      :label-check="isJustInformationMode ? t('flashcardStudy.continue') : ''"
      @check="onStudyFooterCheck"
      @rate="submitAnswer"
      @discuss="openWavesModal"
      @snooze="snoozeCard"
    />

    <LingoModal
      :show="showWavesModal"
      :title="t('flashcardStudy.wavesModalTitle', { id: currentFlashcardId })"
      @close="showWavesModal = false"
    >
      <div v-if="currentCard?.flashcard?.id" class="pt-1">
        <CommentList
          :collection-id="currentCard.flashcard.collection_id"
          :definition-link-id="currentCard.flashcard.id"
          :initial-subject="currentFlashcardTag"
          :embedded="true"
        />
      </div>
    </LingoModal>
  </div>
</template>

<script setup lang="ts">
import { Button, Textarea } from '@packages/ui'
import { CheckCircle2, EqualApproximately, Heart } from '@lucide/vue'
import { ref, onMounted, computed, watch, nextTick, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import AnonymousProgressBanner from '@/components/AnonymousProgressBanner.vue'
import LingoModal from '@/components/LingoModal.vue'
import LingoStudyHeader from '@/components/LingoStudyHeader.vue'
import LingoStudyFooter from '@/components/LingoStudyFooter.vue'
import LingoResultCard from '@/components/LingoResultCard.vue'
import LingoUserProgress from '@/components/LingoUserProgress.vue'
import CommentList from '@/pages/CommentList.vue'
import { useLingoProgress } from '@/composables/useLingoProgress'
import {
  hashLingoQuestionKey,
  pickLingoStudyCharacterPair,
  useLingoStudySfx,
  lingoStudyPose1Src,
} from '@/composables/useLingoStudySfx'

import {
  getDueCards,
  reviewFlashcard,
  snoozeFlashcard,
  getLanguages,
  submitFillinAnswer,
  submitQuizAnswer,
  getFlashcards,
  getLevelCards,
  getLevels,
  getCollectionFlashcardsPublic,
} from '@/api'
import LazyMathJax from '@/components/LazyMathJax.vue'
import AudioPlayer from '@/components/AudioPlayer.vue'
import { useSeoHead } from '@/composables/useSeoHead'
import { useAuth } from '@/composables/useAuth'
import {
  useAnonymousProgress,
  type LevelProgressData,
  answeredAttemptKeys,
  attemptKey,
} from '@/composables/useAnonymousProgress'
import { ensureGuestSession } from '@/composables/useGuestSession'
import { queryStr, paramStr } from '@/utils/routeQuery'

const ANON_BANNER_ANSWERED_KEY = 'lensisku_study_cards_answered'
/** Show sign-up nudge only after a real stretch of practice (not a few taps). */
const ANON_BANNER_AFTER_ANSWERS = 12

const cardsAnsweredInSession = ref(0)
const sessionXpAwarded = ref(0)
const sessionXpClaimed = ref(false)

const { t } = useI18n()

const route = useRoute()
const router = useRouter()
const auth = useAuth()
const {
  getProgress,
  recordCardAttempt,
  isLevelUnlockedForAnon,
  resolveAndRememberActiveLevelId,
  setActiveLevelId,
} = useAnonymousProgress()
const { hearts, hasUnlimitedHearts, loseHeart, awardSessionXp } = useLingoProgress()
const { playCorrect, playIncorrect, playFinish, unlock: unlockStudySfx } = useLingoStudySfx()
const initialStudyCharacters = pickLingoStudyCharacterPair()
const studyQuestionCharacterSrc = ref(initialStudyCharacters.question)
/** Idle reply pose (`*5` / `*7`); display switches to same-family `*1` when the answer is shown. */
const studyAnswerIdleSrc = ref(initialStudyCharacters.answer)

const outOfHearts = computed(() => !hasUnlimitedHearts.value && hearts.value <= 0)

function registerWrongAnswer() {
  playIncorrect()
  loseHeart()
}

function registerCorrectAnswer() {
  playCorrect()
}

function claimSessionXpOnce() {
  if (sessionXpClaimed.value) return
  sessionXpClaimed.value = true
  sessionXpAwarded.value = awardSessionXp(cardsAnsweredInSession.value)
  playFinish()
}

const collectionId = computed(() => paramStr(route.params.collectionId))

const levelIdParam = computed(() => {
  const s = queryStr(route.query.levelId)
  return s ? parseInt(s, 10) : null
})
const isAnonLevelMode = computed(() => !auth.state.isLoggedIn && levelIdParam.value != null)
const isAnonNoLevelsMode = ref(false) // legacy local-only fallback when guest session fails
/** Backend guest session active — same review/due APIs as logged-in. */
const guestSessionActive = ref(false)
/** True only for the legacy localStorage study path (no guest JWT). */
const isAnonStudy = computed(() => !auth.state.isLoggedIn && !guestSessionActive.value)
const anonLevelMeta = ref(null) // { min_cards, min_success_rate } when in anon level mode
/** Next required level to center on the path after a levelled lesson finish. */
const focusLevelId = ref<number | null>(null)

const returnToUrl = computed(() => {
  const cid = collectionId.value
  if (levelIdParam.value != null) {
    const id = focusLevelId.value ?? levelIdParam.value
    return `/collections/${cid}/lingo/levels?focusLevelId=${id}`
  }
  // Always stay in Lingo (including anon / no-levels course study).
  return `/collections/${cid}/lingo/levels`
})

/** Pick the first unlocked incomplete level (next required), else the lesson just finished. */
async function resolveReturnFocusLevel() {
  const cid = collectionId.value
  const fallback = levelIdParam.value
  if (!cid || fallback == null) {
    focusLevelId.value = fallback
    return
  }
  try {
    const res = await getLevels(cid)
    const list = [...(res.data.levels || [])].sort((a, b) => (a.position ?? 0) - (b.position ?? 0))
    if (auth.state.isLoggedIn) {
      const next = list.find(
        (l) => (!l.is_locked || !!l.progress?.is_unlocked) && !l.progress?.is_completed
      )
      focusLevelId.value = next?.level_id ?? fallback
    } else {
      focusLevelId.value = resolveAndRememberActiveLevelId(cid, list) ?? fallback
    }
  } catch {
    focusLevelId.value = fallback
  }
}

const isLoading = ref(true)
const remainingCards = ref([])
const currentCard = ref(null)
const showWavesModal = ref(false)
const showAnswer = ref(false)

const currentFlashcardTag = computed(() => {
  const id = currentCard.value?.flashcard?.id
  if (!id) return ''
  return `#[${id}]`
})

const currentFlashcardId = computed(() => {
  const id = currentCard.value?.flashcard?.id
  return id ? String(id) : ''
})
/** Cards in this study session (level queue / loaded batch) — not collection-wide due total. */
const sessionTotal = ref(0)
const showAnswerButtonRef = ref(null)
const nextCardButtonRef = ref(null)
const fillInTextareaRef = ref(null)
const returnToDeckButtonRef = ref(null)
const answerAudioPlayerRef = ref(null)
const questionAudioPlayerRef = ref(null)

const languages = ref([])

/** Fill-in is supported: direction must include 'fillin' (e.g. fillin, fillin_both, fillin_reverse). API "both" = reveal then rate. */
const isFillInMode = computed(() => {
  const dir = currentCard.value?.flashcard?.direction
  return dir && dir.toLowerCase().includes('fillin')
})

const isJustInformationMode = computed(() => {
  const dir = currentCard.value?.flashcard?.direction
  return dir && dir.toLowerCase() === 'justinformation'
})

/** Multiple-choice quiz: select one of the options (text or image). */
const isQuizMode = computed(() => {
  const dir = currentCard.value?.flashcard?.direction
  return dir && dir.toLowerCase().startsWith('quiz')
})

const showCanonicalForm = computed(() => {
  const fc = currentCard.value?.flashcard
  if (!fc?.canonical_form) return false
  const main = (fc.word ?? fc.free_content_front ?? '').trim().toLowerCase()
  const canonical = fc.canonical_form.trim().toLowerCase()
  return main !== canonical
})

/** True when fill-in answer was correct (API `answer_correct`, or anon/legacy `correct`/`success`). */
const isFillinCorrect = computed(() => {
  const r = fillinResult.value
  if (!r) return false
  if (r.answer_correct !== undefined && r.answer_correct !== null) return r.answer_correct === true
  return r.correct === true || r.success === true
})

const getLanguageName = (langId) => {
  const lang = languages.value.find((l) => l.id === langId)
  return lang?.real_name || 'Unknown'
}

/**
 * Progress 0–100 for header bar: share of this session’s cards finished.
 * Uses the loaded queue size (per level / batch), not API `total` due in the collection.
 */
const progressPercentage = computed(() => {
  if (sessionTotal.value <= 0) return 0
  const stillInSession = remainingCards.value.length + (currentCard.value ? 1 : 0)
  if (stillInSession <= 0) return 100
  const done = Math.max(0, sessionTotal.value - stillInSession)
  return Math.min(100, Math.round((done / sessionTotal.value) * 100))
})

/** After Check reveals the answer on a graded card, footer shows Forgot/Good/Easy. */
const footerShowsRate = computed(
  () =>
    !!currentCard.value &&
    showAnswer.value &&
    !isFillInMode.value &&
    !isJustInformationMode.value &&
    !quizResult.value
)

/** Single footer status — Check / correct / wrong / rate (fixed-height chrome). */
const studyFooterStatus = computed(() => {
  if (footerShowsRate.value) return 'rate'
  if (quizResult.value) return quizResult.value.correct ? 'correct' : 'wrong'
  if (fillinResult.value) return isFillinCorrect.value ? 'correct' : 'wrong'
  if (showAnswer.value && isFillInMode.value) return 'correct'
  return 'none'
})

const studyFooterDisabled = computed(() => {
  if (isJustInformationMode.value) return isSubmitting.value
  if (studyFooterStatus.value !== 'none') return false
  if (isQuizMode.value) return true // User must tap an option
  if (isFillInMode.value) return !userAnswer.value.trim() || isSubmitting.value
  return isSubmitting.value
})

function onStudyFooterCheck() {
  if (isJustInformationMode.value) {
    submitAnswer(4)
    return
  }
  onFooterCheck()
}

function handleExit() {
  router.push(returnToUrl.value)
}

function onFooterCheck() {
  // Unlock Web Audio on this gesture so Forgot/Good/Easy SFX start without delay.
  void unlockStudySfx()
  const status = studyFooterStatus.value
  if (status === 'correct' || status === 'wrong') {
    if (quizResult.value) handleQuizNextCard()
    else handleNextCard()
    return
  }
  if (isFillInMode.value) submitAnswer()
  else revealAnswerAndPlayAudio()
}

/** Anonymous fill-in: check if user answer matches expected (direct → definition/back, reverse → word/front). */
function isAnonFillinAnswerCorrect() {
  const card = currentCard.value
  const side = card?.progress?.[0]?.card_side
  const dir = (card?.flashcard?.direction ?? '').toLowerCase()
  const fc = card?.flashcard
  if (!fc || !side) return false
  let expectedRaw
  if (side === 'direct' && (dir === 'fillin' || dir === 'fillin_both')) {
    expectedRaw = (fc.definition ?? fc.free_content_back ?? '').trim()
  } else if (side === 'reverse' && (dir === 'fillin_reverse' || dir === 'fillin_both')) {
    expectedRaw = (fc.word ?? fc.free_content_front ?? '').trim()
  } else {
    return false
  }
  const user = userAnswer.value.trim().toLowerCase()
  if (!user) return false
  const expectedLower = expectedRaw.toLowerCase()
  if (expectedLower.includes(';')) {
    const options = expectedLower
      .split(';')
      .map((s) => s.trim())
      .filter(Boolean)
    return options.some((opt) => opt === user)
  }
  return expectedLower === user
}

/** Load due cards for this session. With a levelId, fetch every page so progress is per-level. */
const loadDueCards = async (singleCardId = null) => {
  isLoading.value = true
  try {
    if (singleCardId) {
      const response = await getFlashcards({
        collection_id: collectionId.value,
        flashcard_id: singleCardId,
        per_page: 1,
      })
      remainingCards.value = response.data.flashcards || []
      sessionTotal.value = remainingCards.value.length
    } else if (levelIdParam.value) {
      const all = []
      let page = 1
      const perPage = 100
      let total = 0
      do {
        const response = await getDueCards({
          collection_id: collectionId.value,
          level_id: levelIdParam.value,
          page,
          per_page: perPage,
        })
        const batch = response.data.flashcards || []
        total = Number(response.data.total || 0)
        all.push(...batch)
        page += 1
        if (batch.length === 0) break
      } while (all.length < total)
      remainingCards.value = all
      sessionTotal.value = all.length
    } else {
      // Open due session: progress within the loaded batch (not collection-wide due count)
      const response = await getDueCards({
        collection_id: collectionId.value,
        per_page: 50,
      })
      remainingCards.value = response.data.flashcards || []
      sessionTotal.value = remainingCards.value.length
    }
    loadNextCard()
  } catch (error) {
    console.error(t('flashcardStudy.loadError'), error)
  } finally {
    isLoading.value = false
  }
}

/** Anonymous level mode: load all cards for the level (no auth), shape as study cards. */
const loadLevelCardsForAnon = async () => {
  const lid = levelIdParam.value
  const cid = collectionId.value
  if (!lid || !cid) return
  isLoading.value = true
  try {
    const levelsRes = await getLevels(cid)
    const list = levelsRes.data.levels || []
    const level = list.find((l) => l.level_id === lid)
    // Enforce prerequisites for guests (deep links cannot skip the ladder).
    if (level && !isLevelUnlockedForAnon(cid, level)) {
      const activeId = resolveAndRememberActiveLevelId(cid, list)
      router.replace(
        activeId != null
          ? `/collections/${cid}/lingo/levels?focusLevelId=${activeId}`
          : `/collections/${cid}/lingo/levels`
      )
      return
    }
    if (level) {
      anonLevelMeta.value = { min_cards: level.min_cards, min_success_rate: level.min_success_rate }
      setActiveLevelId(cid, Number(lid))
    }
    const all = []
    let page = 1
    const perPage = 100
    let total = 0
    do {
      const res = await getLevelCards(lid, page, perPage)
      const list = res.data.cards || []
      total = res.data.total || 0
      for (const c of list) {
        all.push({
          flashcard: {
            id: c.flashcard_id,
            word: c.word,
            definition: c.definition,
            free_content_front: c.free_content_front,
            free_content_back: c.free_content_back,
            has_front_image: c.has_front_image,
            has_back_image: c.has_back_image,
            sound_url: c.sound_url,
            collection_id: c.collection_id,
            item_id: c.item_id,
            direction: 'direct',
          },
          progress: [{ card_side: 'direct', status: 'new' }],
        })
      }
      page++
    } while (all.length < total)
    restoreAnonQueue(cid, lid, all)
  } catch (error) {
    console.error(t('flashcardStudy.loadError'), error)
  } finally {
    isLoading.value = false
  }
}

/** Anonymous no-levels mode: load all flashcards for the collection (public API), one study card per side. Returns true if cards were loaded. */
const loadCollectionCardsForAnon = async () => {
  const cid = collectionId.value
  if (!cid) return false
  isLoading.value = true
  try {
    const res = await getCollectionFlashcardsPublic(cid)
    const list = res.data.flashcards || []
    const all = []
    for (const item of list) {
      const progressList = item.progress || []
      for (const p of progressList) {
        all.push({
          flashcard: item.flashcard,
          progress: [{ ...p, status: p.status || 'new' }],
        })
      }
    }
    if (all.length === 0) return false
    isAnonNoLevelsMode.value = true
    anonLevelMeta.value = null
    restoreAnonQueue(cid, 0, all)
    return true
  } catch (error) {
    console.error(t('flashcardStudy.loadError'), error)
    return false
  } finally {
    isLoading.value = false
  }
}

/** Resume guest queue from localStorage attempts so reload does not restart the lesson. */
function restoreAnonQueue(cid: string | number, lid: string | number, all: unknown[]) {
  const local = getProgress(cid, lid) as LevelProgressData | null
  const done = answeredAttemptKeys(local)
  const remaining = (
    all as Array<{ flashcard?: { id?: number }; progress?: Array<{ card_side?: string }> }>
  ).filter((c) => {
    const id = c.flashcard?.id
    if (id == null) return true
    return !done.has(attemptKey(id, c.progress?.[0]?.card_side))
  })
  sessionTotal.value = all.length
  const answered = Math.max(done.size, local?.total_answers ?? 0)
  cardsAnsweredInSession.value = answered
  try {
    sessionStorage.setItem(ANON_BANNER_ANSWERED_KEY, String(answered))
  } catch (_) {}
  remainingCards.value = remaining
  currentCard.value = null
  if (remaining.length === 0) {
    // All cards already answered — show finish screen.
    return
  }
  // Prefer URL hash card if it is still unanswered; else next remaining.
  const hash = route.hash || ''
  if (hash.startsWith('#q-')) {
    const want = hash.slice(1)
    const idx = remaining.findIndex((c) => {
      const side = c.progress?.[0]?.card_side ?? 'direct'
      const key = `${side}\0${stimulusTextForCard(c)}`
      return `q-${hashLingoQuestionKey(key)}` === want
    })
    if (idx > 0) {
      const [picked] = remaining.splice(idx, 1)
      remainingCards.value = remaining
      currentCard.value = picked
      showAnswer.value = false
      return
    }
  }
  loadNextCard()
}

const applyAnonLevelProgress = (correct: boolean) => {
  const cid = collectionId.value
  // Prefer real level id; fall back to 0 for collection-wide guest study.
  const lid = levelIdParam.value ?? (isAnonNoLevelsMode.value || isAnonStudy.value ? 0 : null)
  if (!cid || lid == null) return
  const card = currentCard.value
  const flashcardId = card?.flashcard?.id
  if (flashcardId == null) return
  recordCardAttempt(
    cid,
    lid,
    {
      flashcard_id: Number(flashcardId),
      card_side: card?.progress?.[0]?.card_side || 'direct',
      correct,
    },
    anonLevelMeta.value
  )
}

const loadNextCard = () => {
  if (remainingCards.value.length > 0) {
    currentCard.value = remainingCards.value.shift()
    showAnswer.value = false
    userAnswer.value = ''
    fillinResult.value = null
    quizResult.value = null
  } else {
    currentCard.value = null
  }
}

/** Stimulus text used for the per-question URL hash (content-addressed, not card id). */
function stimulusTextForCard(card) {
  const fc = card?.flashcard
  if (!fc) return ''
  const side = card.progress?.[0]?.card_side
  if (side === 'direct') {
    return String(fc.question_text ?? fc.word ?? fc.free_content_front ?? '')
  }
  return String(fc.question_text ?? fc.definition ?? fc.free_content_back ?? '')
}

function syncQuestionHash(card) {
  // Always keep path + query (e.g. levelId) — `{ hash }` alone drops them in vue-router.
  const keep = { path: route.path, query: { ...route.query } }
  if (!card) {
    if (route.hash) router.replace({ ...keep, hash: '' })
    return
  }
  const side = card.progress?.[0]?.card_side ?? 'direct'
  const key = `${side}\0${stimulusTextForCard(card)}`
  const next = `#q-${hashLingoQuestionKey(key)}`
  if (route.hash !== next) {
    router.replace({ ...keep, hash: next })
  }
}

const userAnswer = ref('')
const isSubmitting = ref(false)
const fillinResult = ref(null)
const quizResult = ref(null)

/** True when stimulus is definition/back (user recalls the word). */
const isReverseSide = computed(() => currentCard.value?.progress?.[0]?.card_side !== 'direct')

/** Duolingo-style instruction above the card (clone lesson `title` h1). Kept on reveal to avoid CLS. */
const lessonPrompt = computed(() => {
  if (!currentCard.value) return ''
  const reverse = isReverseSide.value
  if (isQuizMode.value && !quizResult.value) {
    return reverse ? t('flashcardStudy.howDoYouSayThis') : t('flashcardStudy.selectOption')
  }
  if (isFillInMode.value && !fillinResult.value) {
    return reverse ? t('flashcardStudy.howDoYouSayThis') : t('flashcardStudy.typeYourAnswer')
  }
  if (isJustInformationMode.value) return t('flashcardStudy.readThis')
  return reverse ? t('flashcardStudy.howDoYouSayThis') : t('flashcardStudy.whatDoesThisMean')
})

/** Graded / wrong fill-in reveal — second speech bubble under the question. */
const showAnswerBubble = computed(
  () =>
    !!currentCard.value &&
    !quizResult.value &&
    (showAnswer.value || (!!fillinResult.value && !isFillinCorrect.value))
)

/** Same reply character; pose `*1` while the answer text is visible. */
const studyAnswerCharacterSrc = computed(() =>
  showAnswerBubble.value ? lingoStudyPose1Src(studyAnswerIdleSrc.value) : studyAnswerIdleSrc.value
)

/** Quiz options / result surface (fill-in types inside the reply bubble). */
const showStudyCardShell = computed(() => {
  if (!currentCard.value) return false
  if (isQuizMode.value) return true
  if (quizResult.value) return true
  return false
})

/** Image quiz: options are "item_id:side"; build list with image URLs for current collection. */
const quizImageOptions = computed(() => {
  const fc = currentCard.value?.flashcard
  const opts = fc?.quiz_options
  if (!opts?.length || !fc?.collection_id) return null
  const first = opts[0]
  if (typeof first !== 'string' || !first.includes(':')) return null
  return opts.map((id) => ({
    id,
    imageUrl: `/api/collections/${fc.collection_id}/items/${id.split(':')[0]}/image/${id.split(':')[1] || 'front'}`,
  }))
})

const submitAnswer = async (rating?: number) => {
  if (!currentCard.value || isSubmitting.value) return
  isSubmitting.value = true

  try {
    const next = cardsAnsweredInSession.value + 1
    cardsAnsweredInSession.value = next
    try {
      sessionStorage.setItem(ANON_BANNER_ANSWERED_KEY, String(next))
    } catch (_) {}
    if (isAnonStudy.value) {
      let correct
      if (isFillInMode.value) {
        correct = isAnonFillinAnswerCorrect()
        fillinResult.value = { correct, message: correct ? t('flashcardStudy.correctAnswer') : '' }
        applyAnonLevelProgress(correct)
        if (correct) registerCorrectAnswer()
        else registerWrongAnswer()
        await nextTick()
        if (!isJustInformationMode.value) answerAudioPlayerRef.value?.play()
      } else {
        correct = typeof rating === 'number' ? rating >= 3 : true
        applyAnonLevelProgress(correct)
        if (correct) registerCorrectAnswer()
        else registerWrongAnswer()
        // SFX already played on Check; advance after rate.
        showAnswer.value = false
        loadNextCard()
      }
      isSubmitting.value = false
      return
    }
    if (isFillInMode.value) {
      const response = await submitFillinAnswer({
        flashcard_id: currentCard.value.flashcard.id,
        card_side: currentCard.value.progress[0].card_side,
        answer: userAnswer.value.trim(),
      })
      fillinResult.value = response.data
      const ok =
        response.data &&
        (response.data.answer_correct === true ||
          response.data.correct === true ||
          response.data.success === true)
      if (ok) registerCorrectAnswer()
      else registerWrongAnswer()
      await checkForDueChanges()
      await nextTick()
      if (!isJustInformationMode.value) answerAudioPlayerRef.value?.play()
    } else if (!isQuizMode.value) {
      if (rating === undefined) return
      // Play SFX on tap — do not wait for the review network round-trip.
      if (rating < 3) registerWrongAnswer()
      else registerCorrectAnswer()
      await reviewFlashcard({
        flashcard_id: currentCard.value.flashcard.id,
        rating,
        card_side: currentCard.value.progress[0].card_side,
      })
      loadNextCard()
      await checkForDueChanges()
    }
  } catch (error) {
    console.error(t('flashcardStudy.submitError'), error)
  } finally {
    isSubmitting.value = false
  }
}

const submitQuizOption = async (selectedOption) => {
  if (!currentCard.value || isSubmitting.value) return
  const fc = currentCard.value.flashcard
  const options = quizImageOptions.value?.length
    ? quizImageOptions.value.map((o) => o.id)
    : fc.quiz_options || []
  isSubmitting.value = true
  try {
    const next = cardsAnsweredInSession.value + 1
    cardsAnsweredInSession.value = next
    try {
      sessionStorage.setItem(ANON_BANNER_ANSWERED_KEY, String(next))
    } catch (_) {}
    if (isAnonStudy.value) {
      const correct =
        String(selectedOption).trim().toLowerCase() ===
        String((fc.quiz_options || [])[0] || '')
          .trim()
          .toLowerCase()
      quizResult.value = { correct, message: correct ? '' : t('flashcardStudy.correctAnswer') }
      applyAnonLevelProgress(correct)
      if (correct) registerCorrectAnswer()
      else registerWrongAnswer()
    } else {
      const response = await submitQuizAnswer({
        flashcard_id: fc.id,
        card_side: currentCard.value.progress[0].card_side,
        selected_answer_text: selectedOption,
        presented_options: options,
      })
      quizResult.value = response.data
      if (response.data?.correct) registerCorrectAnswer()
      else registerWrongAnswer()
      await checkForDueChanges()
    }
    await nextTick()
    if (quizResult.value?.correct && fc.sound_url) answerAudioPlayerRef.value?.play()
  } catch (error) {
    console.error(t('flashcardStudy.submitError'), error)
  } finally {
    isSubmitting.value = false
  }
}

const handleQuizNextCard = async () => {
  quizResult.value = null
  loadNextCard()
  if (!isAnonStudy.value) await checkForDueChanges()
}

const snoozeCard = async () => {
  if (!currentCard.value || isSubmitting.value) return
  isSubmitting.value = true
  try {
    if (isAnonStudy.value) {
      loadNextCard()
      return
    }
    await snoozeFlashcard(currentCard.value.flashcard.id)
    loadNextCard()
    await checkForDueChanges()
  } catch (error) {
    console.error(t('flashcardStudy.snoozeError'), error)
  } finally {
    isSubmitting.value = false
  }
}

const openWavesModal = () => {
  if (!currentCard.value) return
  showWavesModal.value = true
}

const revealAnswerAndPlayAudio = async () => {
  showAnswer.value = true
  showNewCardsMessage.value = false
  if (!isJustInformationMode.value) {
    // Don't play audio for justinformation cards on reveal
    await nextTick() // Wait for the answer section to render
    answerAudioPlayerRef.value?.play() // Play audio when answer is revealed
  }
}

const handleNextCard = async () => {
  fillinResult.value = null
  userAnswer.value = ''
  loadNextCard()
  if (!isAnonStudy.value) await checkForDueChanges()
}

const newCardsMessage = ref('')
const showNewCardsMessage = ref(false)

async function checkForDueChanges() {
  // Level sessions load the full queue up front — empty means the lesson is done.
  if (levelIdParam.value) return
  if (remainingCards.value.length > 0 || currentCard.value) return

  try {
    const response = await getDueCards({
      collection_id: collectionId.value,
      per_page: 50,
    })
    const batch = response.data.flashcards || []
    if (batch.length === 0) return
    const completedSoFar = sessionTotal.value
    remainingCards.value = batch
    sessionTotal.value = completedSoFar + batch.length
    showNewCardsMessage.value = true
    newCardsMessage.value = t('flashcardStudy.newCardsMessage')
    loadNextCard()
  } catch (e) {
    console.error(e)
  }
}

// --- Focus and Keyboard Handling ---

// Card change: random Q/A sprites (different prefixes), hash URL, focus / autoplay.
watch(
  currentCard,
  (newCard) => {
    if (showWavesModal.value) showWavesModal.value = false
    if (newCard) {
      const pair = pickLingoStudyCharacterPair()
      studyQuestionCharacterSrc.value = pair.question
      studyAnswerIdleSrc.value = pair.answer
    }
    syncQuestionHash(newCard)
    if (newCard && isFillInMode.value && !fillinResult.value) {
      nextTick(() => {
        fillInTextareaRef.value?.focus()
      })
    }
    if (newCard?.progress?.[0]?.card_side === 'direct' && newCard?.flashcard?.sound_url) {
      nextTick(() => {
        questionAudioPlayerRef.value?.play()
      })
    }
  },
  { immediate: true }
)

// Watch for end of session to focus the "Return to Deck" button
watch(
  [currentCard, remainingCards],
  ([newCard, newRemaining]) => {
    if (!newCard && newRemaining.length === 0) {
      nextTick(() => {
        returnToDeckButtonRef.value?.focus()
      })
    }
  },
  { deep: true }
)

watch(showAnswer, (newValue) => {
  if (newValue && !isFillInMode.value) {
    // Focus rating buttons container or first button? For now, just log.
    // Consider focusing the "Forgot" button as a default.
  } else if (!newValue && !isFillInMode.value) {
    nextTick(() => {
      showAnswerButtonRef.value?.focus()
    })
  }
})

watch(
  () => !isLoading.value && !currentCard.value && remainingCards.value.length === 0,
  async (finished) => {
    if (!finished || cardsAnsweredInSession.value <= 0) return
    if (levelIdParam.value != null) await resolveReturnFocusLevel()
    claimSessionXpOnce()
  }
)

watch(fillinResult, (newValue) => {
  if (newValue) {
    nextTick(() => {
      nextCardButtonRef.value?.focus()
    })
  }
})

const handleKeydown = (event) => {
  if (event.target.tagName === 'TEXTAREA') return

  if (event.key === ' ' || event.key === 'Enter') {
    event.preventDefault()
    if (!currentCard.value && remainingCards.value.length === 0) {
      returnToDeckButtonRef.value?.click()
      return
    }
    if (studyFooterStatus.value === 'correct' || studyFooterStatus.value === 'wrong') {
      onFooterCheck()
      return
    }
    if (isJustInformationMode.value) {
      submitAnswer(4)
      return
    }
    if (showAnswer.value && !isFillInMode.value) return // rating buttons use 1/2/3
    if (!studyFooterDisabled.value) onFooterCheck()
  } else if (showAnswer.value && !isFillInMode.value) {
    if (event.key === '1') submitAnswer(1)
    else if (event.key === '2') submitAnswer(3)
    else if (event.key === '3') submitAnswer(4)
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
})

// --- End Focus and Keyboard Handling ---

const flashcardStudySeoTitle = computed(() => t('flashcardStudy.title'))
useSeoHead({ title: flashcardStudySeoTitle })

onMounted(async () => {
  void unlockStudySfx()
  try {
    const stored = sessionStorage.getItem(ANON_BANNER_ANSWERED_KEY)
    cardsAnsweredInSession.value = parseInt(stored || '0', 10)
  } catch (_) {}
  try {
    const langsResponse = await getLanguages()
    languages.value = langsResponse.data

    if (!auth.state.isLoggedIn) {
      const session = await ensureGuestSession()
      guestSessionActive.value = !!session
    }

    // Real user or backend guest: one study mechanism (due + review APIs).
    if (auth.state.isLoggedIn || guestSessionActive.value) {
      const cardIdStr = queryStr(route.query.card_id)
      const cardIdToLoad = cardIdStr ? parseInt(cardIdStr, 10) : null
      await loadDueCards(cardIdToLoad)
      return
    }

    // Fallback if guest session could not be created (offline / API down).
    if (isAnonLevelMode.value) {
      await loadLevelCardsForAnon()
      return
    }
    const cid = collectionId.value
    const loaded = await loadCollectionCardsForAnon()
    if (!loaded) {
      router.replace(`/collections/${cid}/lingo/levels`)
    }
  } catch (error) {
    console.error(t('flashcardStudy.loadInitialError'), error)
  }
})
</script>

<style scoped>
/* Scrollbar-less card content when overflow; keep layout compact */
.lingo-study-root :deep(.overflow-auto) {
  scrollbar-width: none;
  -ms-overflow-style: none;
}
.lingo-study-root :deep(.overflow-auto)::-webkit-scrollbar {
  display: none;
}

/* Reply/answer: caret points right toward the sprite (overrides left caret). */
.lingo-study-root :deep(.lingo-study-reply-bubble::before),
.lingo-study-root :deep(.lingo-study-answer-bubble::before) {
  left: auto;
  right: -0.75rem;
  transform: translateY(-50%) rotate(-90deg);
}
.lingo-study-root :deep(.lingo-study-answer-bubble::before) {
  border-top-color: #bbf7d0;
}

/* Dialogue stagger: question inset from the right, reply from the left. */
.lingo-study-root :deep(.lingo-study-question-row:not(.lingo-study-question-row--reply)) {
  padding-right: 3rem;
}
.lingo-study-root :deep(.lingo-study-question-row--reply) {
  padding-left: 3rem;
}
@media (min-width: 640px) {
  .lingo-study-root :deep(.lingo-study-question-row:not(.lingo-study-question-row--reply)) {
    padding-right: 4rem;
  }
  .lingo-study-root :deep(.lingo-study-question-row--reply) {
    padding-left: 4rem;
  }
}
@media (min-width: 1024px) {
  .lingo-study-root :deep(.lingo-study-question-row:not(.lingo-study-question-row--reply)) {
    padding-right: 5rem;
  }
  .lingo-study-root :deep(.lingo-study-question-row--reply) {
    padding-left: 5rem;
  }
}
</style>
