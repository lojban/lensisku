<template>

  <div class="flashcard-study-root" :class="{ 'pt-for-anon-banner': anonBannerVisible }">
     <AnonymousProgressBanner
      v-if="cardsAnsweredInSession >= 4"
      position="top"
      @visible="anonBannerVisible = $event"
    /> <!-- Session Header -->
    <div class="bg-white border rounded-lg p-4 mb-6">

      <div class="flex flex-wrap justify-between items-center gap-4">

        <div class="flex flex-col sm:flex-row sm:items-center gap-3 min-w-0 flex-1">
          <CollectionCoverLightbox
            v-if="collectionCoverDisplayUrl"
            :image-url="collectionCoverDisplayUrl"
            :alt="
              collectionMeta?.name
                ? t('collectionDetail.coverImageAlt', { name: collectionMeta.name })
                : t('flashcardStudy.title')
            "
            :aria-label="
              collectionMeta?.name
                ? t('collectionDetail.coverLightboxDialog', { name: collectionMeta.name })
                : t('flashcardStudy.title')
            "
            :close-aria-label="t('collectionDetail.coverLightboxClose')"
            class="shrink-0 mx-auto sm:mx-0"
          >
            <div class="collection-card-logo overflow-hidden">
              <img
                :src="collectionCoverDisplayUrl"
                :alt="
                  collectionMeta?.name
                    ? t('collectionDetail.coverImageAlt', { name: collectionMeta.name })
                    : t('flashcardStudy.title')
                "
                class="h-full w-full object-cover"
                loading="lazy"
                decoding="async"
              />
            </div>
          </CollectionCoverLightbox>
          <div
            v-else
            class="collection-card-logo-placeholder shrink-0 mx-auto sm:mx-0"
            aria-hidden="true"
          >
            <BookOpen class="h-8 w-8" />
          </div>
          <div class="min-w-0 text-center sm:text-left">
          <h2 class="text-xl font-bold text-gray-800"> {{ t('flashcardStudy.title') }} </h2>

          <p v-if="showNewCardsMessage" class="text-sm text-orange-600 font-medium mt-1">
             {{ t('flashcardStudy.newCardsMessage') }}
          </p>

          <p v-else class="text-sm text-gray-600 mt-1">
             {{ t('flashcardStudy.remainingCards', { count: remainingCards.length }) }}
          </p>

          </div>
        </div>

        <div class="flex gap-4 space-x-4">
           <button class="ui-btn--cancel" @click="router.back()">
             {{ t('flashcardStudy.endSession') }} </button
          > <button v-if="currentCard" class="ui-btn--empty" @click="snoozeCard">
             {{ t('flashcardStudy.snooze') }} </button
          >
        </div>

      </div>

    </div>
     <!-- Loading State -->
    <div v-if="isLoading" class="flex justify-center py-8">

      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />

    </div>
     <!-- No Cards State -->
    <div
      v-else-if="!currentCard && remainingCards.length === 0"
      class="text-center py-12 bg-white rounded-lg border"
    >

      <h3 class="text-lg font-medium text-gray-800 mb-2">
         {{ t('flashcardStudy.allCaughtUp') }}
      </h3>

      <p class="text-gray-600 mb-4"> {{ t('flashcardStudy.allReviewed') }} </p>

      <div class="flex justify-center">
         <button
          ref="returnToDeckButtonRef"
          class="ui-btn--get w-auto h-10 text-base shadow-sm"
          @click="router.push(returnToUrl)"
        >
           {{
            levelIdParam != null
              ? t('flashcardStudy.returnToLevels')
              : t('flashcardStudy.returnToDeck')
          }} </button
        >
      </div>

    </div>
     <!-- Current Card -->
    <div v-else-if="currentCard" class="flex flex-col gap-4">
       <!-- Card Display -->
      <div class="bg-white border rounded-lg p-4 sm:p-6">
         <!-- Progress indicator -->
        <div
          v-for="progress in currentCard.progress"
          :key="progress.card_side"
          class="flex justify-between items-center mb-4 text-xs sm:text-sm text-gray-600"
        >
           <span>{{
            t('flashcardStudy.cardOfTotal', {
              current: totalCards - remainingCards.length,
              total: totalCards,
            })
          }}</span
          > <span :class="getStatusClass(progress.status)"
            > {{ progress.card_side }}: {{ progress.status }} </span
          >
        </div>
         <!-- Card content -->
        <div class="flex flex-col gap-4">

          <div
            v-if="currentCard.flashcard.definition_language_id"
            class="text-sm text-gray-600 text-center"
          >
             {{
              t('flashcardStudy.definitionLanguage', {
                language: getLanguageName(currentCard.flashcard.definition_language_id),
              })
            }}
          </div>
           <!-- Canonical Comparison Indicator for Fill-in Mode -->
          <div
            v-if="isFillInMode && currentCard.flashcard.use_canonical_comparison"
            class="badge-streak-success"
          >
             <CheckCircle2 class="h-3.5 w-3.5" /> <span>{{
              t('flashcardStudy.canonicalComparisonEnabled')
            }}</span
            >
          </div>
           <!-- Question --> <!-- For quiz mode prefer question_text when present --> <!-- Display Word/Front for 'direct' side -->

          <div v-if="currentCard.progress[0].card_side === 'direct'">

            <div class="flex items-center justify-center gap-2">

              <h3 class="text-2xl font-bold text-gray-800">
                 {{
                  currentCard.flashcard.question_text ??
                  currentCard.flashcard.word ??
                  currentCard.flashcard.free_content_front
                }}
              </h3>
               <AudioPlayer
                v-if="currentCard.flashcard.sound_url"
                :key="'q-' + currentCard.flashcard.id"
                ref="questionAudioPlayerRef"
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
                class="h-6 w-6"
              />
            </div>

            <div v-if="currentCard.flashcard.has_front_image" class="mt-4 flex justify-center">
               <img
                :src="`/api/collections/${currentCard.flashcard.collection_id}/items/${currentCard.flashcard.item_id}/image/front`"
                class="max-h-48 rounded-lg object-contain bg-gray-100"
                alt="Front image"
              />
            </div>

          </div>
           <!-- Display Definition/Back for 'reverse' side -->
          <div v-else>

            <div class="text-center text-gray-800 text-2xl">
               <LazyMathJax
                :content="
                  currentCard.flashcard.question_text ??
                  currentCard.flashcard.definition ??
                  currentCard.flashcard.free_content_back
                "
              />
              <div v-if="currentCard.flashcard.has_back_image" class="mt-4 flex justify-center">
                 <img
                  :src="`/api/collections/${currentCard.flashcard.collection_id}/items/${currentCard.flashcard.item_id}/image/back`"
                  class="max-h-48 rounded-lg object-contain bg-gray-100"
                  alt="Back image"
                />
              </div>

            </div>

          </div>
           <!-- Quiz: multiple-choice options (text or image) -->
          <div
            v-if="
              isQuizMode &&
              !quizResult &&
              (currentCard.flashcard.quiz_options?.length || quizImageOptions?.length)
            "
            class="mt-4"
          >

            <p class="text-sm text-gray-600 text-center mb-3">
               {{ t('flashcardStudy.selectOption') }}
            </p>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 sm:gap-3">
               <template v-if="quizImageOptions?.length"
                > <button
                  v-for="opt in quizImageOptions"
                  :key="opt.id"
                  type="button"
                  class="study-quiz-option-flashcard-image"
                  :disabled="isSubmitting"
                  @click="submitQuizOption(opt.id)"
                >
                   <img
                    :src="opt.imageUrl"
                    class="w-full h-24 sm:h-28 object-contain rounded-lg"
                    :alt="opt.id"
                  /> </button
                > </template
              > <button
                v-else
                v-for="(opt, idx) in currentCard.flashcard.quiz_options"
                :key="idx"
                type="button"
                class="study-quiz-option-flashcard-text"
                :disabled="isSubmitting"
                @click="submitQuizOption(opt)"
              >
                 <LazyMathJax :content="opt" /> </button
              >
            </div>

          </div>
           <!-- Answer Input (for fill-in modes) -->
          <div v-if="isFillInMode" class="mt-4">
             <textarea
              ref="fillInTextareaRef"
              v-model="userAnswer"
              type="text"
              rows="1"
              class="textarea-field w-full text-center text-xl"
              :placeholder="t('flashcardStudy.typeAnswer')"
              @keydown.enter.prevent="() => submitAnswer()"
              :disabled="!!fillinResult"
            />
          </div>
           <!-- Quiz result (correct/incorrect + correct answer) -->
          <div v-if="quizResult" class="flex flex-col gap-4 pt-4 border-t">

            <div
              :class="
                quizResult.correct ? 'text-green-600 font-semibold' : 'text-red-600 font-semibold'
              "
              class="text-center text-lg"
            >
               {{
                quizResult.correct
                  ? t('flashcardStudy.answerCorrect')
                  : t('flashcardStudy.answerIncorrect')
              }}
            </div>

            <p v-if="!quizResult.correct && quizResult.message" class="text-center text-gray-700">
               {{ quizResult.message }}
            </p>

          </div>
           <!-- Answer Display (shown after revealing, or after fill-in when incorrect) -->
          <div
            v-if="(showAnswer || (fillinResult && !isFillinCorrect)) && !quizResult"
            class="flex flex-col gap-4 pt-4 border-t"
          >

            <div class="prose max-w-none text-center text-lg">

              <h4 class="text-sm text-center text-gray-700 mb-2">
                 {{ t('flashcardStudy.correctAnswer') }}
              </h4>
               <!-- Show the OTHER side as the correct answer --> <template
                v-if="currentCard.progress[0].card_side === 'direct'"
                > <!-- If question was front/word, show back/definition --> <LazyMathJax
                  :content="
                    currentCard.flashcard.definition ?? currentCard.flashcard.free_content_back
                  "
                />
                <div v-if="currentCard.flashcard.has_back_image" class="mt-4 flex justify-center">
                   <img
                    :src="`/api/collections/${currentCard.flashcard.collection_id}/items/${currentCard.flashcard.item_id}/image/back`"
                    class="max-h-48 rounded-lg object-contain bg-gray-100"
                    alt="Back image"
                  />
                </div>
                 </template
              > <template v-else
                > <!-- If question was back/definition, show front/word -->
                <div class="flex items-center justify-center gap-2">
                   <span>{{
                    currentCard.flashcard.word ?? currentCard.flashcard.free_content_front
                  }}</span
                  > <AudioPlayer
                    ref="answerAudioPlayerRef"
                    v-if="currentCard.flashcard.sound_url"
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
                    class="h-6 w-6 inline-block"
                  />
                </div>

                <div
                  v-if="showCanonicalForm"
                  class="mt-3 pt-3 border-t flex flex-col gap-1.5 text-center"
                >

                  <div
                    class="flex items-center justify-center gap-2 text-xs font-semibold text-gray-400 uppercase tracking-wider"
                  >
                     <EqualApproximately class="h-3.5 w-3.5 text-blue-400" /> <span>{{
                      t('components.definitionCard.canonicalLabel')
                    }}</span
                    >
                  </div>

                  <div
                    class="code-snippet-surface inline-block mx-auto"
                  >
                     {{ currentCard.flashcard.canonical_form }}
                  </div>

                </div>

                <div v-if="currentCard.flashcard.has_front_image" class="mt-4 flex justify-center">
                   <img
                    :src="`/api/collections/${currentCard.flashcard.collection_id}/items/${currentCard.flashcard.item_id}/image/front`"
                    class="max-h-48 rounded-lg object-contain bg-gray-100"
                    alt="Front image"
                  />
                </div>
                 </template
              >
            </div>
             <!-- Display Notes only when showing the definition side -->
            <div
              v-if="currentCard.flashcard.notes && currentCard.progress[0].card_side === 'direct'"
            >

              <h4 class="text-sm font-medium text-gray-700 mb-2">
                 {{ t('flashcardStudy.notes') }}
              </h4>
               <LazyMathJax :content="currentCard.flashcard.notes" :enable-markdown="true" />
            </div>

          </div>

        </div>

      </div>
       <!-- Review Controls (quiz mode uses option buttons in card, no extra button) -->
      <div
        v-if="!showAnswer && !fillinResult && !quizResult && !isJustInformationMode && !isQuizMode"
        class="flex justify-center px-4"
      >
         <button
          v-if="isFillInMode"
          class="ui-btn--get w-auto h-10 text-base shadow-sm"
          @click="submitAnswer()"
        >
           {{ t('flashcardStudy.submitAnswer') }} </button
        > <button
          v-else
          ref="showAnswerButtonRef"
          class="ui-btn--get w-auto h-10 text-base shadow-sm"
          @click="revealAnswerAndPlayAudio"
        >
           {{ t('flashcardStudy.showAnswer') }} </button
        >
      </div>
       <!-- OK button for JustInformation mode -->
      <div v-else-if="isJustInformationMode && !showAnswer" class="flex justify-center px-4">
         <button class="ui-btn--get w-auto h-10 text-base shadow-sm" @click="submitAnswer(4)">
           <Check class="h-4 w-4" /> </button
        >
      </div>
       <!-- Quiz: Next card after answering -->
      <div v-else-if="quizResult" class="flex justify-center px-4 mt-4">
         <button
          v-if="remainingCards.length <= 0"
          class="ui-btn--get w-auto h-10 text-base shadow-sm"
          @click="router.back()"
        >
           {{ t('flashcardStudy.endSession') }} </button
        > <button
          v-else
          ref="nextCardButtonRef"
          class="ui-btn--get w-auto h-10 text-base shadow-sm"
          @click="handleQuizNextCard"
        >
           {{ t('flashcardStudy.nextCard') }} </button
        >
      </div>
       <!-- Rating buttons (for non-fill-in modes after showing answer) -->
      <div
        v-else-if="showAnswer && !isFillInMode && !isJustInformationMode"
        class="bg-white border rounded-lg p-4 sm:p-6"
      >

        <h4 class="text-lg font-medium text-center text-gray-700 mb-6">
           {{ t('flashcardStudy.howWellRemembered') }}
        </h4>

        <div class="flex justify-center px-4 sm:px-6">

          <div class="w-full max-w-xl">

            <div class="grid grid-cols-3 gap-2 sm:gap-4">
               <button
                class="ui-btn--error w-full sm:min-w-[120px] flex items-center justify-center gap-1.5"
                @click="submitAnswer(1)"
              >
                 <XCircle class="h-4 w-4 shrink-0" />
                <span class="inline-flex min-w-0 items-center gap-1">
                  <span>{{ t('flashcardStudy.forgot') }}</span>
                  <span class="hidden sm:inline">(1)</span>
                </span>
              </button
              > <button
                class="ui-btn--warning w-full sm:min-w-[120px] flex items-center justify-center gap-1.5"
                @click="submitAnswer(3)"
              >
                 <Smile class="h-4 w-4 shrink-0" />
                <span class="inline-flex min-w-0 items-center gap-1">
                  <span>{{ t('flashcardStudy.good') }}</span>
                  <span class="hidden sm:inline">(2)</span>
                </span>
              </button
              > <button
                class="ui-btn--success w-full sm:min-w-[120px] flex items-center justify-center gap-1.5"
                @click="submitAnswer(4)"
              >
                 <Check class="h-4 w-4 shrink-0" />
                <span class="inline-flex min-w-0 items-center gap-1">
                  <span>{{ t('flashcardStudy.easy') }}</span>
                  <span class="hidden sm:inline">(3)</span>
                </span>
              </button
              >
            </div>

          </div>

        </div>

      </div>
       <!-- Next Card button (for fill-in modes after submitting) -->
      <div v-else-if="fillinResult" class="flex justify-center px-4">

        <div class="flex flex-col gap-2 mt-4 items-center">
           <!-- When correct: show a clear success message only -->
          <div
            v-if="isFillinCorrect"
            class="flex items-center gap-2 text-green-600 font-medium text-lg"
          >
             <Check class="h-6 w-6 shrink-0" /> <span>{{ t('flashcardStudy.answerCorrect') }}</span
            >
          </div>
           <!-- When incorrect: correct answer is shown above; no error alert --> <button
            v-if="remainingCards.length <= 0"
            class="ui-btn--get w-auto h-10 text-base shadow-sm"
            @click="router.back()"
          >
             {{ t('flashcardStudy.endSession') }} </button
          > <button
            v-else
            ref="nextCardButtonRef"
            class="ui-btn--get w-auto h-10 text-base shadow-sm"
            @click="handleNextCard"
          >
             {{ t('flashcardStudy.nextCard') }} </button
          >
          <div v-if="remainingCards.length === 0" class="text-center text-gray-600 mt-2">
             {{ t('flashcardStudy.thanksSession') }}
          </div>

        </div>

      </div>

    </div>

  </div>

</template>

<script setup lang="ts">
import {
  XCircle,
  Check,
  Smile,
  CheckCircle2,
  EqualApproximately,
  BookOpen,
} from 'lucide-vue-next'
import { ref, onMounted, computed, watch, nextTick, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import AnonymousProgressBanner from '@/components/AnonymousProgressBanner.vue'
import { CollectionCoverLightbox } from '@packages/ui'

import {
  getDueCards,
  reviewFlashcard,
  getLanguages,
  submitFillinAnswer,
  submitQuizAnswer,
  getFlashcards,
  snoozeFlashcard,
  getLevelCards,
  getLevels,
  getCollectionFlashcardsPublic,
  getCollection,
  getCollectionImage,
} from '@/api'
import LazyMathJax from '@/components/LazyMathJax.vue'
import AudioPlayer from '@/components/AudioPlayer.vue'
import { useSeoHead } from '@/composables/useSeoHead'
import { useAuth } from '@/composables/useAuth'
import { useAnonymousProgress, type LevelProgressData } from '@/composables/useAnonymousProgress'
import { queryStr } from '@/utils/routeQuery'

const ANON_BANNER_ANSWERED_KEY = 'lensisku_study_cards_answered'

const anonBannerVisible = ref(false)
const cardsAnsweredInSession = ref(0)

const { t, locale } = useI18n()

const route = useRoute()
const router = useRouter()
const auth = useAuth()
const { getProgress, saveLevelProgress } = useAnonymousProgress()

const levelIdParam = computed((): number | null => {
  const s = queryStr(route.query.levelId)
  if (!s) return null
  const n = parseInt(s, 10)
  return Number.isNaN(n) ? null : n
})
const isAnonLevelMode = computed(() => !auth.state.isLoggedIn && levelIdParam.value != null)
const isAnonNoLevelsMode = ref(false) // anon studying collection with no levels (uses public flashcards)
const anonLevelMeta = ref(null) // { min_cards, min_success_rate } when in anon level mode

const collectionIdParam = computed(() => queryStr(route.params.collectionId))

/** Set when GET /collections/:id succeeds (for cover image + alt text). */
const collectionMeta = ref<{ name?: string; has_collection_image?: boolean } | null>(null)

const collectionCoverDisplayUrl = computed(() => {
  const cid = collectionIdParam.value
  if (!cid || !collectionMeta.value?.has_collection_image) return null
  return getCollectionImage(cid, { cached: true })
})

async function fetchCollectionMeta() {
  const cid = collectionIdParam.value
  if (!cid) return
  try {
    const res = await getCollection(cid)
    collectionMeta.value = {
      name: res.data.name,
      has_collection_image: res.data.has_collection_image,
    }
  } catch {
    collectionMeta.value = null
  }
}

const returnToUrl = computed(() => {
  const cid = collectionIdParam.value
  if (levelIdParam.value != null) return `/collections/${cid}/levels`
  if (isAnonNoLevelsMode.value) return `/collections/${cid}/flashcards`
  return `/collections/${cid}/flashcards`
})

const isLoading = ref(true)
const remainingCards = ref([])
const currentCard = ref(null)
const showAnswer = ref(false)
const totalCards = ref(0)
const showAnswerButtonRef = ref(null)
const nextCardButtonRef = ref(null)
const fillInTextareaRef = ref(null)
const returnToDeckButtonRef = ref(null)
const answerAudioPlayerRef = ref(null)
const questionAudioPlayerRef = ref(null)

const languages = ref([])

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

const getLanguageName = (langId: number) => {
  const lang = languages.value.find((l) => l.id === langId)
  return lang?.real_name || 'Unknown'
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

const loadDueCards = async (singleCardId: number | null = null) => {
  isLoading.value = true
  try {
    let response
    if (singleCardId) {
      // Fetch a single specific card for review
      response = await getFlashcards({
        collection_id: collectionIdParam.value,
        flashcard_id: singleCardId,
        per_page: 1, // Ensure we only get one
      })
      // Adapt the response structure if necessary, assuming it returns a list
      remainingCards.value = response.data.flashcards
      totalCards.value = 1 // Only one card in this session
    } else {
      // Fetch all due cards
      response = await getDueCards({
        collection_id: collectionIdParam.value,
        level_id: levelIdParam.value || undefined,
      })
      remainingCards.value = response.data.flashcards
      totalCards.value = response.data.total
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
  const cid = collectionIdParam.value
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
    totalCards.value = all.length
    loadNextCard()
  } catch (error) {
    console.error(t('flashcardStudy.loadError'), error)
  } finally {
    isLoading.value = false
  }
}

/** Anonymous no-levels mode: load all flashcards for the collection (public API), one study card per side. Returns true if cards were loaded. */
const loadCollectionCardsForAnon = async () => {
  const cid = collectionIdParam.value
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
    totalCards.value = all.length
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

function normalizeAnonLevelProgress(p: ReturnType<typeof getProgress>): LevelProgressData {
  if (p && typeof p === 'object' && 'total_answers' in p) {
    return p as LevelProgressData
  }
  return { cards_completed: 0, correct_answers: 0, total_answers: 0 }
}

const applyAnonLevelProgress = (correct: boolean) => {
  const cid = collectionIdParam.value
  const lid = isAnonNoLevelsMode.value ? 0 : levelIdParam.value
  if (!cid || lid == null) return
  const cur = normalizeAnonLevelProgress(getProgress(cid, lid))
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
        await nextTick()
        if (!isJustInformationMode.value) answerAudioPlayerRef.value?.play()
      } else {
        correct = typeof rating === 'number' ? rating >= 3 : true
        applyAnonLevelProgress(correct)
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
      await checkForDueChanges()
      await nextTick()
      if (!isJustInformationMode.value) answerAudioPlayerRef.value?.play()
    } else if (!isQuizMode.value) {
      await reviewFlashcard({
        flashcard_id: currentCard.value.flashcard.id,
        rating,
        card_side: currentCard.value.progress[0].card_side,
      })
      loadNextCard()
      await checkForDueChanges()
    }
    // Quiz mode is handled by submitQuizOption
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
    } else {
      const response = await submitQuizAnswer({
        flashcard_id: fc.id,
        card_side: currentCard.value.progress[0].card_side,
        selected_answer_text: selectedOption,
        presented_options: options,
      })
      quizResult.value = response.data
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

const snoozeCard = async () => {
  if (!currentCard.value || isSubmitting.value) return
  isSubmitting.value = true
  try {
    if (isAnonLevelMode.value || isAnonNoLevelsMode.value) {
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
  // Check if we need to refresh due cards if the current queue is empty
  if (remainingCards.value.length === 0) {
    await loadDueCards() // This might repopulate remainingCards
    if (remainingCards.value.length > 0) {
      // Check again after loading
      showNewCardsMessage.value = true
      newCardsMessage.value = t('flashcardStudy.newCardsMessage')
      // If new cards were loaded and the session seemed over, load the first new card
      if (!currentCard.value) {
        loadNextCard()
      }
    }
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

watch(fillinResult, (newValue) => {
  if (newValue) {
    nextTick(() => {
      nextCardButtonRef.value?.focus()
    })
  }
})

const handleKeydown = (event) => {
  // Ignore if typing in the textarea
  if (event.target.tagName === 'TEXTAREA') {
    return
  }

  if (event.key === ' ' || event.key === 'Enter') {
    event.preventDefault() // Prevent default space/enter behavior

    if (fillinResult.value && nextCardButtonRef.value) {
      nextCardButtonRef.value.click()
    } else if (!showAnswer.value && !isFillInMode.value && showAnswerButtonRef.value) {
      // For JustInformation mode, Enter/Space should act like clicking "OK"
      if (isJustInformationMode.value) {
        submitAnswer(4) // Rating 4 for "Easy" / "Learned"
      } else {
        showAnswerButtonRef.value.click()
      }
    } else if (isFillInMode.value && !fillinResult.value) {
      // Trigger submit for fill-in mode if answer is present
      if (userAnswer.value.trim()) {
        submitAnswer()
      }
    } else if (
      !currentCard.value &&
      remainingCards.value.length === 0 &&
      returnToDeckButtonRef.value &&
      !isJustInformationMode.value
    ) {
      // Trigger "Return to Deck" button at the end of the session
      returnToDeckButtonRef.value.click()
    }
  } else if (showAnswer.value && !isFillInMode.value) {
    // Handle rating button shortcuts
    if (event.key === '1') {
      submitAnswer(1) // Forgot
    } else if (event.key === '2') {
      submitAnswer(3) // Good
    } else if (event.key === '3') {
      submitAnswer(4) // Easy
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
})

// --- End Focus and Keyboard Handling ---

const getStatusClass = (status: string) => {
  const classes = {
    new: 'text-blue-600 bg-blue-50 px-2 py-1 rounded-full text-xs',
    learning: 'text-yellow-600 bg-yellow-50 px-2 py-1 rounded-full text-xs',
    review: 'text-green-600 bg-green-50 px-2 py-1 rounded-full text-xs',
    graduated: 'text-purple-600 bg-purple-50 px-2 py-1 rounded-full text-xs',
  }
  return classes[status.toLowerCase()] || ''
}

useSeoHead({ title: () => t('flashcardStudy.title') })

onMounted(async () => {
  try {
    const stored = sessionStorage.getItem(ANON_BANNER_ANSWERED_KEY)
    cardsAnsweredInSession.value = parseInt(stored || '0', 10)
  } catch (_) {}
  try {
    const langsResponse = await getLanguages()
    languages.value = langsResponse.data

    await fetchCollectionMeta()

    if (isAnonLevelMode.value) {
      await loadLevelCardsForAnon()
      return
    }
    if (!auth.state.isLoggedIn) {
      const cid = collectionIdParam.value
      const loaded = await loadCollectionCardsForAnon()
      if (!loaded) {
        router.replace(`/collections/${cid}/levels`)
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
.flashcard-study-root.pt-for-anon-banner {
  padding-top: 5.5rem;
}
</style>

