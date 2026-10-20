<template>
  <div
    class="lingo-study-root flex h-full min-h-0 flex-col overflow-hidden bg-slate-50"
    :class="{ 'pt-for-anon-banner': anonBannerVisible }"
  >
    <AnonymousProgressBanner
      v-if="cardsAnsweredInSession >= 4"
      position="top"
      @visible="anonBannerVisible = $event"
    />
    <!-- Full-bleed header stripe; inner chrome matches the lesson column width -->
    <LingoStudyHeader
      v-if="(currentCard || outOfHearts) && !isLoading"
      :percentage="progressPercentage"
      @exit="handleExit"
    >
      <template #extra>
        <div class="flex items-center gap-1 text-rose-500">
          <Heart class="h-5 w-5 fill-rose-500" />
          <InfinityIcon v-if="hasUnlimitedHearts" class="h-4 w-4 stroke-[3]" />
          <span v-else class="text-sm font-bold">{{ hearts }}</span>
        </div>
      </template>
    </LingoStudyHeader>

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
      <!-- No Cards State (Duolingo finish: centered, compact) -->
      <div
        v-else-if="!currentCard && remainingCards.length === 0"
        class="flex flex-1 flex-col items-center justify-center gap-2 min-h-0 py-4 text-center"
      >
        <h1 class="text-base font-bold text-neutral-700 sm:text-lg">
          {{ t('flashcardStudy.allCaughtUp') }} <br />
          {{ t('flashcardStudy.allReviewed') }}
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
              ? t('flashcardStudy.returnToLevels')
              : t('flashcardStudy.returnToDeck')
          }}
        </button>
      </div>
      <!-- Lesson card (footer is full-bleed below this column) -->
      <div v-else-if="currentCard" class="flex flex-1 flex-col min-h-0 overflow-hidden">
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

            <!-- Clone QuestionBubble: random LTD dialogue icon + speech bubble -->
            <div class="lingo-study-question-row">
              <img :src="studyCharacterSrc" alt="" class="lingo-study-mascot" decoding="async" />
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

            <div
              v-if="isFillInMode && currentCard.flashcard.use_canonical_comparison"
              class="badge-streak-success--compact"
            >
              <CheckCircle2 class="h-3 w-3" />
              <span>{{ t('flashcardStudy.canonicalComparisonEnabled') }}</span>
            </div>

            <!-- Options / fill-in / revealed answer (clone: challenge cards, not the title) -->
            <div v-if="showStudyCardShell" class="lingo-study-card-shell">
              <div class="flex flex-col gap-1 min-h-0">
                <!-- Quiz: multiple-choice options (text or image) -->
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
                          class="w-full h-16 sm:h-20 object-contain rounded-lg"
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
                <!-- Answer Input (fill-in) -->
                <div v-if="isFillInMode" class="mt-1">
                  <Textarea
                    ref="fillInTextareaRef"
                    v-model="userAnswer"
                    type="text"
                    rows="1"
                    class="textarea-field w-full text-center text-sm sm:text-base"
                    :placeholder="t('flashcardStudy.typeAnswer')"
                    :disabled="!!fillinResult"
                    @keydown.enter.prevent="() => submitAnswer()"
                  />
                </div>
                <!-- Quiz result (options hidden once answered — no divider needed) -->
                <div v-if="quizResult" class="flex flex-col gap-1">
                  <div
                    :class="
                      quizResult.correct
                        ? 'text-green-600 font-semibold'
                        : 'text-red-600 font-semibold'
                    "
                    class="text-center text-sm"
                  >
                    {{
                      quizResult.correct
                        ? t('flashcardStudy.answerCorrect')
                        : t('flashcardStudy.answerIncorrect')
                    }}
                  </div>

                  <p
                    v-if="!quizResult.correct && quizResult.message"
                    class="text-center text-gray-600 text-xs"
                  >
                    {{ quizResult.message }}
                  </p>
                </div>
                <!-- Answer after Check / wrong fill-in. Divider only when fill-in input still sits above. -->
                <div
                  v-if="(showAnswer || (fillinResult && !isFillinCorrect)) && !quizResult"
                  class="flex flex-col gap-1"
                  :class="isFillInMode ? 'mt-1.5 border-t border-slate-200 pt-1.5' : ''"
                >
                  <div class="prose prose-sm max-w-none text-center text-sm sm:text-base">
                    <h4 class="text-[10px] text-center text-gray-700 mb-0.5 sm:text-xs">
                      {{ t('flashcardStudy.correctAnswer') }}
                    </h4>
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
                          alt="Back image"
                        />
                      </div>
                    </template>
                    <template v-else>
                      <div class="flex items-center justify-center gap-1.5">
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
                        class="mt-1 pt-1 border-t border-slate-100 flex flex-col gap-0.5 text-center"
                      >
                        <div
                          class="flex items-center justify-center gap-1 text-[10px] font-semibold text-gray-400 uppercase tracking-wider"
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
                          alt="Front image"
                        />
                      </div>
                    </template>
                  </div>

                  <div
                    v-if="
                      currentCard.flashcard.notes && currentCard.progress[0].card_side === 'direct'
                    "
                    class="text-[10px] sm:text-xs"
                  >
                    <h4 class="font-medium text-gray-700 mb-0.5">
                      {{ t('flashcardStudy.notes') }}
                    </h4>
                    <LazyMathJax :content="currentCard.flashcard.notes" :enable-markdown="true" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Full-bleed footer stripe; inner chrome matches the lesson column width -->
    <template v-if="currentCard && !isLoading && !outOfHearts">
      <LingoStudyFooter
        v-if="footerShowsRate"
        status="rate"
        :disabled="isSubmitting"
        :compact="true"
        :rate-prompt="t('flashcardStudy.howWellRemembered')"
        @rate="submitAnswer"
      />
      <LingoStudyFooter
        v-else-if="!isJustInformationMode"
        :status="footerStatus"
        :disabled="footerDisabled"
        :compact="true"
        :correct-label="t('flashcardStudy.nicelyDone')"
        :wrong-label="t('flashcardStudy.tryAgain')"
        @check="onFooterCheck"
      />
      <LingoStudyFooter
        v-else
        status="none"
        :disabled="isSubmitting"
        :compact="true"
        :label-check="t('flashcardStudy.continue')"
        @check="submitAnswer(4)"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { Button, Textarea } from '@packages/ui'
import { CheckCircle2, EqualApproximately, Heart, Infinity as InfinityIcon } from '@lucide/vue'
import { ref, onMounted, computed, watch, nextTick, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import AnonymousProgressBanner from '@/components/AnonymousProgressBanner.vue'
import LingoStudyHeader from '@/components/LingoStudyHeader.vue'
import LingoStudyFooter from '@/components/LingoStudyFooter.vue'
import LingoResultCard from '@/components/LingoResultCard.vue'
import { useLingoProgress } from '@/composables/useLingoProgress'
import { pickRandomLingoCharacter, useLingoStudySfx } from '@/composables/useLingoStudySfx'

import {
  getDueCards,
  reviewFlashcard,
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
import { useAnonymousProgress, type LevelProgressData } from '@/composables/useAnonymousProgress'
import { queryStr, paramStr } from '@/utils/routeQuery'

const ANON_BANNER_ANSWERED_KEY = 'lensisku_study_cards_answered'

const anonBannerVisible = ref(false)
const cardsAnsweredInSession = ref(0)
const sessionXpAwarded = ref(0)
const sessionXpClaimed = ref(false)

const { t } = useI18n()

const route = useRoute()
const router = useRouter()
const auth = useAuth()
const { getProgress, saveLevelProgress } = useAnonymousProgress()
const { hearts, hasUnlimitedHearts, loseHeart, awardSessionXp } = useLingoProgress()
const { playCorrect, playIncorrect, playFinish } = useLingoStudySfx()
const studyCharacterSrc = ref(pickRandomLingoCharacter())

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
const isAnonNoLevelsMode = ref(false) // anon studying collection with no levels (uses public flashcards)
const anonLevelMeta = ref(null) // { min_cards, min_success_rate } when in anon level mode

const returnToUrl = computed(() => {
  const cid = collectionId.value
  if (levelIdParam.value != null) return `/collections/${cid}/lingo/levels`
  if (isAnonNoLevelsMode.value) return `/collections/${cid}/flashcards`
  return `/collections/${cid}/lingo/levels`
})

const isLoading = ref(true)
const remainingCards = ref([])
const currentCard = ref(null)
const showAnswer = ref(false)
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

/** Footer status for Duolingo-style footer */
const footerStatus = computed(() => {
  if (quizResult.value) return quizResult.value.correct ? 'correct' : 'wrong'
  if (fillinResult.value) return isFillinCorrect.value ? 'correct' : 'wrong'
  if (showAnswer.value && isFillInMode.value) return 'correct'
  return 'none'
})

const footerDisabled = computed(() => {
  if (footerStatus.value !== 'none') return false
  if (isQuizMode.value) return true // User must tap an option
  if (isFillInMode.value) return !userAnswer.value.trim() || isSubmitting.value
  return isSubmitting.value
})

function handleExit() {
  router.push(returnToUrl.value)
}

function onFooterCheck() {
  if (footerStatus.value === 'correct' || footerStatus.value === 'wrong') {
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
    const level = (levelsRes.data.levels || []).find((l) => l.level_id === lid)
    if (level) {
      anonLevelMeta.value = { min_cards: level.min_cards, min_success_rate: level.min_success_rate }
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
    remainingCards.value = all
    sessionTotal.value = all.length
    loadNextCard()
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
    remainingCards.value = all
    sessionTotal.value = all.length
    isAnonNoLevelsMode.value = true
    anonLevelMeta.value = null
    loadNextCard()
    return true
  } catch (error) {
    console.error(t('flashcardStudy.loadError'), error)
    return false
  } finally {
    isLoading.value = false
  }
}

const applyAnonLevelProgress = (correct: boolean) => {
  const cid = collectionId.value
  const lid = isAnonNoLevelsMode.value ? 0 : levelIdParam.value
  if (!cid || lid == null) return
  const merged: LevelProgressData = {
    cards_completed: 0,
    correct_answers: 0,
    total_answers: 0,
    ...(getProgress(cid, lid) as LevelProgressData | null),
  }
  const cur = merged
  const total_answers = (cur.total_answers || 0) + 1
  const correct_answers = (cur.correct_answers || 0) + (correct ? 1 : 0)
  const cards_completed = correct_answers
  let completed_at = cur.completed_at ?? null
  const meta = anonLevelMeta.value
  if (meta && total_answers > 0) {
    const rate = correct_answers / total_answers
    if (cards_completed >= meta.min_cards && rate >= meta.min_success_rate) {
      completed_at = new Date().toISOString()
    }
  }
  saveLevelProgress(cid, lid, { cards_completed, correct_answers, total_answers, completed_at })
}

const loadNextCard = () => {
  if (remainingCards.value.length > 0) {
    currentCard.value = remainingCards.value.shift()
    studyCharacterSrc.value = pickRandomLingoCharacter()
    showAnswer.value = false
    userAnswer.value = ''
    fillinResult.value = null
    quizResult.value = null
  } else {
    currentCard.value = null
  }
}

const userAnswer = ref('')
const isSubmitting = ref(false)
const fillinResult = ref(null)
const quizResult = ref(null)

/** True when stimulus is definition/back (user recalls the word). */
const isReverseSide = computed(() => currentCard.value?.progress?.[0]?.card_side !== 'direct')

/** Duolingo-style instruction above the card (clone lesson `title` h1). */
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
  if (!showAnswer.value) {
    return reverse ? t('flashcardStudy.howDoYouSayThis') : t('flashcardStudy.whatDoesThisMean')
  }
  return ''
})

/** Hide empty white shell until there is quiz / fill-in / revealed answer content. */
const showStudyCardShell = computed(() => {
  if (!currentCard.value) return false
  if (isQuizMode.value) return true
  if (isFillInMode.value) return true
  if (quizResult.value) return true
  if (showAnswer.value) return true
  if (fillinResult.value) return true
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
    if (isAnonLevelMode.value || isAnonNoLevelsMode.value) {
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
      await reviewFlashcard({
        flashcard_id: currentCard.value.flashcard.id,
        rating,
        card_side: currentCard.value.progress[0].card_side,
      })
      if (rating < 3) registerWrongAnswer()
      else registerCorrectAnswer()
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
    if (isAnonLevelMode.value || isAnonNoLevelsMode.value) {
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
  if (!isAnonLevelMode.value && !isAnonNoLevelsMode.value) await checkForDueChanges()
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
  if (!isAnonLevelMode.value && !isAnonNoLevelsMode.value) await checkForDueChanges()
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

// Watch for changes in the current card: focus textarea for fill-in, auto-play question audio for direct
watch(
  currentCard,
  (newCard) => {
    if (newCard && isFillInMode.value && !fillinResult.value) {
      nextTick(() => {
        fillInTextareaRef.value?.focus()
      })
    }
    // Auto-play question audio when showing a direct card with sound (play runs after nextTick; AudioPlayer loads then plays)
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
  (finished) => {
    if (finished && cardsAnsweredInSession.value > 0) claimSessionXpOnce()
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
    if (footerStatus.value === 'correct' || footerStatus.value === 'wrong') {
      onFooterCheck()
      return
    }
    if (isJustInformationMode.value) {
      submitAnswer(4)
      return
    }
    if (showAnswer.value && !isFillInMode.value) return // rating buttons use 1/2/3
    if (!footerDisabled.value) onFooterCheck()
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
  try {
    const stored = sessionStorage.getItem(ANON_BANNER_ANSWERED_KEY)
    cardsAnsweredInSession.value = parseInt(stored || '0', 10)
  } catch (_) {}
  try {
    const langsResponse = await getLanguages()
    languages.value = langsResponse.data

    if (isAnonLevelMode.value) {
      await loadLevelCardsForAnon()
      return
    }
    if (!auth.state.isLoggedIn) {
      const cid = collectionId.value
      const loaded = await loadCollectionCardsForAnon()
      if (!loaded) {
        router.replace(`/collections/${cid}/lingo/levels`)
      }
      return
    }
    const cardIdStr = queryStr(route.query.card_id)
    const cardIdToLoad = cardIdStr ? parseInt(cardIdStr, 10) : null
    await loadDueCards(cardIdToLoad)
  } catch (error) {
    console.error(t('flashcardStudy.loadInitialError'), error)
  }
})
</script>

<style scoped>
.lingo-study-root.pt-for-anon-banner {
  padding-top: 4.5rem;
}

/* Scrollbar-less card content when overflow; keep layout compact */
.lingo-study-root :deep(.overflow-auto) {
  scrollbar-width: none;
  -ms-overflow-style: none;
}
.lingo-study-root :deep(.overflow-auto)::-webkit-scrollbar {
  display: none;
}
</style>
