<template>
  <div class="lingo-study-root flex h-full min-h-0 flex-col overflow-hidden bg-slate-50">
    <!-- Full-bleed header stripe; inner chrome matches the lesson column width -->
    <LingoStudyHeader
      v-if="(currentCard || showEncouragement || outOfHearts) && !isLoading"
      :percentage="progressPercentage"
      @exit="handleExit"
    >
      <template #extra>
        <!-- Same compact chrome as the exit control (`icon-btn-header-back--compact`). -->
        <LingoUserProgress class="icon-btn-header-back--compact !min-w-9" />
      </template>
    </LingoStudyHeader>

    <!-- Lesson column (card body only) -->
    <div
      class="mx-auto flex h-full w-full max-w-[600px] flex-1 flex-col min-h-0 overflow-hidden px-2 sm:px-3"
    >
      <!-- Loading State -->
      <div v-if="isLoading" class="flex flex-1 items-center justify-center min-h-0 py-4">
        <LoadingSpinner variant="inline" size="sm" />
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
      <!-- Mid-lesson encouragement (Duo “I’m so proud of you!” beat). -->
      <div v-else-if="showEncouragement" class="lingo-study-encourage">
        <div class="lingo-study-encourage-stage">
          <div
            class="lingo-study-encourage-mascot"
            role="img"
            aria-hidden="true"
            :style="{ backgroundImage: `url(${encouragementMascotSrc})` }"
          />
          <p class="lingo-study-encourage-bubble">{{ encouragementMessage }}</p>
        </div>
      </div>
      <!-- Duo “Can't listen now” → drop listening for this session. -->
      <div v-else-if="showNoListening" class="lingo-study-encourage">
        <div class="lingo-study-encourage-stage">
          <div
            class="lingo-study-encourage-mascot"
            role="img"
            aria-hidden="true"
            :style="{ backgroundImage: `url(${studyQuestionCharacterSrc})` }"
          />
          <p class="lingo-study-encourage-bubble">{{ t('flashcardStudy.noListeningExercises') }}</p>
        </div>
      </div>
      <!-- Lesson finish (clone quiz.tsx: finish art + XP/hearts + continue to path) -->
      <div
        v-else-if="!currentCard && remainingCards.length === 0"
        class="flex flex-1 flex-col items-center justify-center gap-3 min-h-0 py-4 text-center sm:gap-4"
      >
        <div class="text-5xl sm:text-6xl" aria-hidden="true">🎉</div>
        <h1 class="text-lg font-bold text-neutral-700 sm:text-2xl">
          <template v-if="levelUnlockedBySchedule && cardsAnsweredInSession === 0">
            {{ t('flashcardStudy.nothingDueTitle') }} <br />
            {{
              nextLevelId != null
                ? t('flashcardStudy.nothingDueNextUnlocked')
                : t('flashcardStudy.nothingDueLastLevel')
            }}
          </template>
          <template v-else-if="levelIdParam != null && cardsAnsweredInSession > 0">
            {{ t('flashcardStudy.lessonCompleteTitle') }} <br />
            {{ t('flashcardStudy.lessonCompleteBody') }}
          </template>
          <template v-else-if="sessionCapReached">
            {{ t('flashcardStudy.sessionDone') }} <br />
            {{ t('flashcardStudy.sessionDoneHint', { count: cardsShown }) }}
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
          v-if="levelUnlockedBySchedule && nextLevelId != null"
          type="button"
          class="lingo-btn lingo-btn--primary lingo-btn--lg !normal-case !tracking-normal min-w-[140px]"
          @click="goToNextLevel"
        >
          {{ t('flashcardStudy.goToNextLevel') }}
        </button>
        <button
          v-if="levelUnlockedBySchedule && cardsAnsweredInSession === 0"
          type="button"
          class="lingo-btn lingo-btn--secondary lingo-btn--lg !normal-case !tracking-normal min-w-[140px]"
          @click="practiceLevelAgain"
        >
          {{ t('flashcardStudy.practiceAgain') }}
        </button>
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

            <!-- Listen-tap: big speaker + slow (no written prompt). -->
            <div
              v-if="isListenModeActive && currentCard.flashcard.sound_url"
              class="lingo-study-listen-prompt"
            >
              <AudioPlayer
                :key="'listen-' + currentCard.flashcard.id"
                ref="questionAudioPlayerRef"
                variant="lingo-speaker"
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
              <AudioPlayer
                :key="'listen-slow-' + currentCard.flashcard.id"
                variant="lingo-speaker-slow"
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

            <!-- Stimulus: question speaker + bubble (caret sits in the gap). -->
            <div v-else class="lingo-study-question-row">
              <div
                class="lingo-study-mascot"
                role="img"
                aria-hidden="true"
                :style="{ backgroundImage: `url(${studyQuestionCharacterSrc})` }"
              />
              <div class="lingo-study-question-bubble">
                <!-- Information card: title + Markdown body, no hints and no reply bubble. -->
                <div
                  v-if="isJustInformationMode"
                  class="lingo-study-info prose prose-sm max-w-none [overflow-wrap:anywhere]"
                >
                  <div class="lingo-study-info__title">
                    <LazyMathJax
                      :content="
                        currentCard.flashcard.word ?? currentCard.flashcard.free_content_front ?? ''
                      "
                      :enable-markdown="true"
                    />
                  </div>
                  <LazyMathJax
                    v-if="
                      currentCard.flashcard.definition ?? currentCard.flashcard.free_content_back
                    "
                    :content="
                      currentCard.flashcard.definition ?? currentCard.flashcard.free_content_back
                    "
                    :enable-markdown="true"
                  />
                </div>
                <div
                  v-else-if="currentCard.progress[0].card_side === 'direct'"
                  class="flex items-start justify-between gap-2"
                >
                  <p class="lingo-study-stimulus min-w-0 flex-1 [overflow-wrap:anywhere]">
                    <LingoHintPhrase
                      v-if="showStimulusHints"
                      :text="stimulusText"
                      :phrase-gloss="stimulusPhraseGloss"
                      :token-glosses="stimulusTokenGlosses"
                      :auto-open-first="stimulusAutoHint"
                      @auto-opened="markStimulusHintSeen"
                      @hint-opened="hintUsed = true"
                    />
                    <template v-else>{{ stimulusText }}</template>
                  </p>
                  <AudioPlayer
                    v-if="currentCard.flashcard.sound_url"
                    :key="'q-' + currentCard.flashcard.id"
                    ref="questionAudioPlayerRef"
                    class="shrink-0"
                    variant="lingo-speaker-inline"
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
                  <LingoHintPhrase
                    v-if="showStimulusHints && !stimulusLooksLikeMath"
                    :text="stimulusText"
                    :phrase-gloss="stimulusPhraseGloss"
                    :token-glosses="stimulusTokenGlosses"
                    :auto-open-first="stimulusAutoHint"
                    @auto-opened="markStimulusHintSeen"
                    @hint-opened="hintUsed = true"
                  />
                  <LazyMathJax v-else :content="stimulusText" />
                </div>
              </div>
            </div>

            <div
              v-if="
                !isListenModeActive &&
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

            <!-- Reply: empty/fill-in bubble + speaker on the right (mirrored). Hidden for quiz assist. -->
            <div
              v-if="!isQuizMode && !isJustInformationMode"
              class="lingo-study-question-row lingo-study-question-row--reply"
            >
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
                          variant="lingo-speaker-inline"
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
                  <!-- Tersmu / canonical form under the revealed answer (not a “feature on” badge). -->
                  <div v-if="showCanonicalForm" class="lingo-study-canonical">
                    <div class="lingo-study-canonical__label">
                      <EqualApproximately class="h-3 w-3 text-sky-500" aria-hidden="true" />
                      <span>{{ t('components.definitionCard.canonicalLabel') }}</span>
                    </div>
                    <div class="lingo-study-canonical__value">
                      {{ currentCard.flashcard.canonical_form }}
                    </div>
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
                <template v-else-if="isWordbankMode">
                  <LingoWordbankPanel
                    mode="line"
                    :bank="wordbankChips"
                    :selected-ids="wordbankSelectedIds"
                    :disabled="!!fillinResult"
                    :placeholder="t('flashcardStudy.assembleAnswerHint')"
                    :aria-label="t('flashcardStudy.assembleAnswer')"
                    @unpick="unpickWordbankChip"
                  />
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

            <!-- Word bank tray (typed fill-in lives in the reply bubble). -->
            <div v-if="isWordbankMode && !showAnswerBubble" class="lingo-study-card-shell">
              <LingoWordbankPanel
                mode="bank"
                :bank="wordbankChips"
                :selected-ids="wordbankSelectedIds"
                :disabled="!!fillinResult"
                :aria-label="t('flashcardStudy.wordBank')"
                @pick="pickWordbankChip"
              />
            </div>

            <!-- Quiz assist: Duo-style numbered choices (select, then Check). -->
            <div
              v-if="isQuizMode && quizChoiceItems.length"
              class="lingo-study-quiz-list"
              role="radiogroup"
              :aria-label="lessonPrompt"
            >
              <button
                v-for="(opt, idx) in quizChoiceItems"
                :key="opt.id"
                type="button"
                role="radio"
                class="lingo-study-quiz-choice"
                :class="quizChoiceClass(opt.id)"
                :aria-checked="selectedQuizOption === opt.id"
                :disabled="!!quizResult || isSubmitting"
                @click="selectQuizOption(opt.id)"
              >
                <span class="lingo-study-quiz-choice__num" aria-hidden="true">{{
                  Number(idx) + 1
                }}</span>
                <span class="lingo-study-quiz-choice__label">
                  <img
                    v-if="opt.imageUrl"
                    :src="opt.imageUrl"
                    class="lingo-study-quiz-choice__image"
                    alt=""
                  />
                  <LazyMathJax v-else :content="opt.label" />
                </span>
              </button>
            </div>

            <LingoGuestSaveAlert
              v-if="showGuestSaveAlert"
              class="mt-3 w-full"
              :return-to="levelsReturnTo"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Single footer instance — status swaps inside a fixed-height chrome box. -->
    <LingoStudyFooter
      v-if="(currentCard || showEncouragement || showNoListening) && !isLoading && !outOfHearts"
      :status="studyFooterStatus"
      :disabled="studyFooterDisabled"
      :compact="true"
      :show-card-actions="!showEncouragement && !showNoListening"
      :left-action="isListenModeActive ? 'cant-listen' : 'snooze'"
      :snooze-disabled="isSubmitting"
      :correct-label="t('flashcardStudy.nicelyDone')"
      :wrong-label="t('flashcardStudy.tryAgain')"
      :solution-text="footerSolutionText"
      :label-correct-solution="t('flashcardStudy.correctSolution')"
      :label-too-easy="t('flashcardStudy.tooEasy')"
      :label-too-difficult="t('flashcardStudy.tooDifficult')"
      :rate-prompt="t('flashcardStudy.howWellRemembered')"
      :label-check="
        showEncouragement || showNoListening || isJustInformationMode
          ? t('flashcardStudy.continue')
          : ''
      "
      @check="onStudyFooterCheck"
      @rate="onStudyFooterRate"
      @discuss="openWavesModal"
      @snooze="snoozeCard"
      @skip-listening="skipListeningExercises"
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
import { isTypingInEditable } from '@/utils/isEditableTarget'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import { Textarea } from '@packages/ui'
import { EqualApproximately, Heart } from '@lucide/vue'
import { ref, onMounted, computed, watch, nextTick, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import LingoGuestSaveAlert from '@/components/LingoGuestSaveAlert.vue'
import LingoModal from '@/components/LingoModal.vue'
import LingoStudyHeader from '@/components/LingoStudyHeader.vue'
import LingoStudyFooter from '@/components/LingoStudyFooter.vue'
import LingoHintPhrase from '@/components/LingoHintPhrase.vue'
import LingoResultCard from '@/components/LingoResultCard.vue'
import LingoUserProgress from '@/components/LingoUserProgress.vue'
import LingoWordbankPanel from '@/components/LingoWordbankPanel.vue'
import { expectedFillinLikeAnswer, type WordbankChip } from '@/utils/wordbank'
import CommentList from '@/pages/CommentList.vue'
import { useLingoProgress } from '@/composables/useLingoProgress'
import {
  hashLingoQuestionKey,
  pickLingoStudyCharacterPair,
  pickLingoProudCharacter,
  useLingoStudySfx,
  lingoStudyPose1Src,
  LINGO_STUDY_PROUD_CHARACTERS,
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
import { clearGuestSession, ensureGuestSession } from '@/composables/useGuestSession'
import { queryStr, paramStr } from '@/utils/routeQuery'
import { STUDY_SESSION_MAX_CARDS, studySessionRoom } from '@/config/studyConstants'

const ANON_BANNER_ANSWERED_KEY = 'lensisku_study_cards_answered'
/** Show login-to-save after this many answers in the current study visit (guests only). */
const GUEST_SAVE_ALERT_AFTER = 5

const cardsAnsweredInSession = ref(0)
/** Answers counted this study mount only — not restored from storage. */
const guestAnswersThisVisit = ref(0)
const sessionXpAwarded = ref(0)
const sessionXpClaimed = ref(false)

/** Mid-lesson Duo-style pep talk (once around halfway after a correct answer). */
const showEncouragement = ref(false)
const encouragementShownThisSession = ref(false)
const encouragementMessage = ref('')
const encouragementMascotSrc = ref<string>(LINGO_STUDY_PROUD_CHARACTERS[0])
const lastAnswerWasCorrect = ref(false)
const ENCOURAGEMENT_MESSAGE_KEYS = [
  'flashcardStudy.encouragement.proud',
  'flashcardStudy.encouragement.doingGreat',
  'flashcardStudy.encouragement.keepGoing',
  'flashcardStudy.encouragement.niceWork',
  'flashcardStudy.encouragement.onARoll',
] as const

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
  lastAnswerWasCorrect.value = false
  playIncorrect()
  loseHeart()
}

function registerCorrectAnswer() {
  lastAnswerWasCorrect.value = true
  playCorrect()
}

function pickEncouragementMessage(): string {
  const key =
    ENCOURAGEMENT_MESSAGE_KEYS[Math.floor(Math.random() * ENCOURAGEMENT_MESSAGE_KEYS.length)]!
  return t(key)
}

function pickEncouragementMascot(): string {
  return pickLingoProudCharacter()
}

/** Once mid-lesson after a correct card, if enough work remains. */
function shouldOfferEncouragement(): boolean {
  if (encouragementShownThisSession.value || showEncouragement.value) return false
  if (!lastAnswerWasCorrect.value) return false
  if (remainingCards.value.length < 2) return false
  if (sessionTotal.value < 6) return false
  const milestone = Math.max(4, Math.floor(sessionTotal.value / 2))
  return cardsAnsweredInSession.value >= milestone
}

function openEncouragement() {
  encouragementShownThisSession.value = true
  encouragementMessage.value = pickEncouragementMessage()
  encouragementMascotSrc.value = pickEncouragementMascot()
  fillinResult.value = null
  quizResult.value = null
  selectedQuizOption.value = null
  showAnswer.value = false
  hintUsed.value = false
  userAnswer.value = ''
  wordbankSelectedIds.value = []
  showEncouragement.value = true
}

function continueFromEncouragement() {
  showEncouragement.value = false
  loadNextCard()
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

const levelsReturnTo = computed(() => `/collections/${collectionId.value}/lingo/levels`)

const returnToUrl = computed(() => {
  const cid = collectionId.value
  if (levelIdParam.value != null) {
    const id = focusLevelId.value ?? levelIdParam.value
    return `/collections/${cid}/lingo/levels?focusLevelId=${id}`
  }
  // Always stay in Lingo (including anon / no-levels course study).
  return levelsReturnTo.value
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
/** Learner opened a gloss on this card: sent as `hint_used` (server lowers the rating). */
const hintUsed = ref(false)

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
/** Course-wide sessions (no levelId, no single card) stop after STUDY_SESSION_MAX_CARDS. */
const singleCardSession = ref(false)
const isCappedSession = computed(() => levelIdParam.value == null && !singleCardSession.value)
const cardsShown = ref(0)
const sessionCapReached = ref(false)
const showAnswerButtonRef = ref(null)
const nextCardButtonRef = ref(null)
const fillInTextareaRef = ref(null)
const returnToDeckButtonRef = ref(null)
const answerAudioPlayerRef = ref(null)
const questionAudioPlayerRef = ref(null)

const languages = ref([])

/** `auto` cards: the server picks the question type per review (`presented_type`). */
function cardDir(card) {
  return card?.flashcard?.presented_type || card?.flashcard?.direction
}

/** Typed fill-in: direction must include 'fillin' (e.g. fillin, fillin_both, fillin_reverse). */
const isFillInMode = computed(() => {
  const dir = cardDir(currentCard.value)
  return dir && dir.toLowerCase().includes('fillin')
})

/** Word-bank assemble: direction must include 'wordbank' (incl. listen_wordbank*). */
const isWordbankMode = computed(() => {
  const dir = cardDir(currentCard.value)
  return dir && dir.toLowerCase().includes('wordbank')
})

/** Duo listenTap: listen_wordbank* with audio, until session opts out. */
const isListenDirection = computed(() => {
  const dir = cardDir(currentCard.value)
  if (!dir) return false
  const n = String(dir).toLowerCase().replace(/_/g, '')
  return n.startsWith('listenwordbank')
})

const skipListeningSession = ref(false)
const showNoListening = ref(false)

const isListenModeActive = computed(
  () =>
    isListenDirection.value &&
    !skipListeningSession.value &&
    !!currentCard.value?.flashcard?.sound_url &&
    !fillinResult.value
)

function isListenCard(card) {
  const dir = cardDir(card)
  if (!dir) return false
  const n = String(dir).toLowerCase().replace(/_/g, '')
  return n.startsWith('listenwordbank')
}

/** Drop listen cards for the rest of this lesson (Duo “Can't listen now”). */
function skipListeningExercises() {
  skipListeningSession.value = true
  remainingCards.value = remainingCards.value.filter((c) => !isListenCard(c))
  showNoListening.value = true
  fillinResult.value = null
  quizResult.value = null
  showAnswer.value = false
  hintUsed.value = false
  userAnswer.value = ''
  wordbankSelectedIds.value = []
}

function continueFromNoListening() {
  showNoListening.value = false
  loadNextCard()
}

/** Fill-in or wordbank — both submit via the fill-in grading path. */
const isFillInLikeMode = computed(() => isFillInMode.value || isWordbankMode.value)

const isJustInformationMode = computed(() => {
  const dir = cardDir(currentCard.value)
  return dir && dir.toLowerCase() === 'justinformation'
})

/** Multiple-choice quiz: select one of the options (text or image). */
const isQuizMode = computed(() => {
  const dir = cardDir(currentCard.value)
  return dir && dir.toLowerCase().startsWith('quiz')
})

const wordbankChips = ref<WordbankChip[]>([])
const wordbankSelectedIds = ref<string[]>([])
/** Server target: selected chip texts joined in order must equal this. */
const wordbankExpectedNormalized = ref('')

function syncUserAnswerFromWordbank() {
  if (!isWordbankMode.value) return
  const byId = new Map(wordbankChips.value.map((c) => [c.id, c]))
  userAnswer.value = wordbankSelectedIds.value
    .map((id) => byId.get(id)?.text)
    .filter(Boolean)
    .join(' ')
}

function pickWordbankChip(id: string) {
  if (fillinResult.value || wordbankSelectedIds.value.includes(id)) return
  wordbankSelectedIds.value = [...wordbankSelectedIds.value, id]
  syncUserAnswerFromWordbank()
}

function unpickWordbankChip(id: string) {
  if (fillinResult.value) return
  wordbankSelectedIds.value = wordbankSelectedIds.value.filter((x) => x !== id)
  syncUserAnswerFromWordbank()
}

function rebuildWordbankForCard() {
  wordbankSelectedIds.value = []
  wordbankChips.value = []
  wordbankExpectedNormalized.value = ''
  userAnswer.value = ''
  const card = currentCard.value
  if (!card || !isWordbankMode.value) return
  const fc = card.flashcard || {}
  wordbankChips.value = (fc.wordbank_chips as WordbankChip[] | undefined) ?? []
  wordbankExpectedNormalized.value = fc.wordbank_expected_normalized ?? ''
}

const showCanonicalForm = computed(() => {
  const fc = currentCard.value?.flashcard
  if (!fc?.canonical_form) return false
  const main = (fc.word ?? fc.free_content_front ?? '').trim().toLowerCase()
  const canonical = fc.canonical_form.trim().toLowerCase()
  return main !== canonical
})

/**
 * True when fill-in / wordbank answer was correct.
 * Prefer `answer_correct` from the fill-in API. Do **not** treat `success` as correctness —
 * ReviewResponse.success means the request succeeded, and is always true on HTTP 200.
 */
const isFillinCorrect = computed(() => {
  const r = fillinResult.value
  if (!r) return false
  if (r.answer_correct !== undefined && r.answer_correct !== null) return r.answer_correct === true
  return r.correct === true
})

/** Hide Duo hint tokens after Check / when answer chrome is showing (avoid spoiling). */
const showStimulusHints = computed(
  () =>
    !!currentCard.value &&
    !isListenModeActive.value &&
    !fillinResult.value &&
    !quizResult.value &&
    !showAnswer.value &&
    !showEncouragement.value
)

/**
 * Listen card that cannot be heard (no audio, or "Can't listen now"): the server still grades
 * the Lojban phrase, so show the translation instead of the Lojban text (else the learner
 * would just copy the stimulus). Only the direct side of listen_wordbank* is affected.
 */
const isListenTextFallback = computed(
  () =>
    isListenDirection.value &&
    !isListenModeActive.value &&
    currentCard.value?.progress?.[0]?.card_side === 'direct'
)

/** True when the written stimulus is the front (Lojban) side. */
const stimulusShowsFront = computed(
  () => currentCard.value?.progress?.[0]?.card_side === 'direct' && !isListenTextFallback.value
)

const stimulusText = computed(() => {
  const fc = currentCard.value?.flashcard
  if (!fc) return ''
  if (isListenTextFallback.value) {
    return String(fc.definition ?? fc.free_content_back ?? '')
  }
  if (currentCard.value?.progress?.[0]?.card_side === 'direct') {
    return String(fc.question_text ?? fc.word ?? fc.free_content_front ?? '')
  }
  return String(fc.question_text ?? fc.definition ?? fc.free_content_back ?? '')
})

const stimulusLooksLikeMath = computed(() => /\$|\\[a-zA-Z]/.test(stimulusText.value))

/** Opposite-side phrase used as whole-phrase gloss when no per-token map. */
const stimulusPhraseGloss = computed(() => {
  const fc = currentCard.value?.flashcard
  if (!fc) return ''
  if (stimulusShowsFront.value) {
    return String(fc.definition ?? fc.free_content_back ?? '').trim()
  }
  return String(fc.word ?? fc.free_content_front ?? '').trim()
})

/** Optional `token|gloss` lines in flashcard notes. */
const stimulusTokenGlosses = computed(() => {
  const notes = currentCard.value?.flashcard?.notes
  if (!notes || typeof notes !== 'string') return {}
  const map: Record<string, string> = {}
  for (const line of notes.split(/\r?\n/)) {
    const m = line.match(/^\s*([^|]+)\|(.+)\s*$/)
    if (!m) continue
    const tok = m[1].trim()
    const gloss = m[2].trim()
    if (tok && gloss) map[tok] = gloss
  }
  return map
})

function hintSeenStorageKey(): string {
  const id = currentCard.value?.flashcard?.id
  const side = currentCard.value?.progress?.[0]?.card_side || 'direct'
  return `lingo-hint-seen:${id}:${side}`
}

const stimulusAutoHint = computed(() => {
  if (!showStimulusHints.value || !currentCard.value?.flashcard?.id) return false
  try {
    return sessionStorage.getItem(hintSeenStorageKey()) !== '1'
  } catch {
    return false
  }
})

function markStimulusHintSeen() {
  try {
    sessionStorage.setItem(hintSeenStorageKey(), '1')
  } catch (_) {}
}

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
  if (isCappedSession.value) {
    const target = Math.min(sessionTotal.value, STUDY_SESSION_MAX_CARDS)
    if (!currentCard.value && remainingCards.value.length === 0) return 100
    const doneCapped = Math.max(0, cardsShown.value - (currentCard.value ? 1 : 0))
    return Math.min(100, Math.round((doneCapped / target) * 100))
  }
  const stillInSession = remainingCards.value.length + (currentCard.value ? 1 : 0)
  if (stillInSession <= 0) return 100
  const done = Math.max(0, sessionTotal.value - stillInSession)
  return Math.min(100, Math.round((done / sessionTotal.value) * 100))
})

const showGuestSaveAlert = computed(
  () => !auth.state.isLoggedIn && guestAnswersThisVisit.value >= GUEST_SAVE_ALERT_AFTER
)

function bumpGuestAnswersThisVisit() {
  if (auth.state.isLoggedIn) return
  guestAnswersThisVisit.value += 1
}

/** After Check reveals the answer on a classical card, footer shows Forgot/Good/Easy. */
const footerShowsRate = computed(
  () =>
    !!currentCard.value &&
    showAnswer.value &&
    !isFillInLikeMode.value &&
    !isJustInformationMode.value &&
    !quizResult.value
)

/**
 * Pending graded commit after Duo Check (fill-in / quiz).
 * FSRS / anon progress apply on Continue / Too easy / Too difficult.
 */
const pendingGradeCommit = ref(null)

/** Single footer status — Check / correct / wrong / rate (fixed-height chrome). */
const studyFooterStatus = computed(() => {
  if (showEncouragement.value) return 'correct'
  // Skip listening → red Continue (Duo “can’t listen”).
  if (showNoListening.value) return 'wrong'
  if (footerShowsRate.value) return 'rate'
  if (quizResult.value) return quizResult.value.correct ? 'correct' : 'wrong'
  if (fillinResult.value) return isFillinCorrect.value ? 'correct' : 'wrong'
  if (showAnswer.value && isFillInLikeMode.value) return 'correct'
  return 'none'
})

/** Wrong-panel “Correct solution: …” body. */
const footerSolutionText = computed(() => {
  if (studyFooterStatus.value !== 'wrong' || showNoListening.value) return ''
  const card = currentCard.value
  if (!card) return ''
  if (quizResult.value) {
    const correctId = quizCorrectOptionId.value
    if (correctId == null) return ''
    const img = quizImageOptions.value?.find(
      (o) => String(o.id).trim().toLowerCase() === String(correctId).trim().toLowerCase()
    )
    if (img?.label) return String(img.label)
    return String(correctId)
  }
  if (fillinResult.value) {
    return expectedFillinLikeAnswer(card.flashcard, card.progress[0].card_side)
  }
  return ''
})

const studyFooterDisabled = computed(() => {
  if (showEncouragement.value || showNoListening.value) return false
  if (isJustInformationMode.value) return isSubmitting.value
  if (studyFooterStatus.value === 'correct' || studyFooterStatus.value === 'wrong') {
    return isSubmitting.value
  }
  if (studyFooterStatus.value !== 'none') return false
  if (isQuizMode.value) return !selectedQuizOption.value || isSubmitting.value
  if (isFillInLikeMode.value) return !userAnswer.value.trim() || isSubmitting.value
  return isSubmitting.value
})

function onStudyFooterCheck() {
  if (showEncouragement.value) {
    continueFromEncouragement()
    return
  }
  if (showNoListening.value) {
    continueFromNoListening()
    return
  }
  if (isJustInformationMode.value) {
    submitAnswer(4)
    return
  }
  onFooterCheck()
}

function onStudyFooterRate(rating: number) {
  if (fillinResult.value || quizResult.value) {
    void commitGradedAndAdvance(rating)
    return
  }
  void submitAnswer(rating)
}

function handleExit() {
  router.push(returnToUrl.value)
}

function onFooterCheck() {
  // Unlock Web Audio on this gesture so Forgot/Good/Easy SFX start without delay.
  void unlockStudySfx()
  const status = studyFooterStatus.value
  if (status === 'correct' || status === 'wrong') {
    // Duo: Continue commits FSRS with default rating (correct→3, wrong→1).
    if (fillinResult.value || quizResult.value) {
      void commitGradedAndAdvance(status === 'correct' ? 3 : 1)
      return
    }
    if (status === 'correct' && shouldOfferEncouragement()) {
      openEncouragement()
      return
    }
    handleNextCard()
    return
  }
  if (isQuizMode.value && selectedQuizOption.value) {
    void submitQuizOption(selectedQuizOption.value)
    return
  }
  if (isFillInLikeMode.value) submitAnswer()
  else revealAnswerAndPlayAudio()
}

/**
 * Commit deferred FSRS (or anon progress) after Duo footer Continue / soft rates, then advance.
 */
async function commitGradedAndAdvance(rating: number) {
  if (!currentCard.value || isSubmitting.value) return
  const pending = pendingGradeCommit.value
  const wasCorrect =
    quizResult.value?.correct === true || (fillinResult.value != null && isFillinCorrect.value)
  isSubmitting.value = true
  try {
    if (isAnonStudy.value) {
      applyAnonLevelProgress(rating >= 3)
    } else if (pending?.kind === 'fillin') {
      await submitFillinAnswer({
        flashcard_id: currentCard.value.flashcard.id,
        card_side: currentCard.value.progress[0].card_side,
        answer: pending.answer,
        rating,
        hint_used: hintUsed.value,
        presented_type: currentCard.value.flashcard.presented_type,
      })
      await checkForDueChanges()
    } else if (pending?.kind === 'quiz') {
      await submitQuizAnswer({
        flashcard_id: currentCard.value.flashcard.id,
        card_side: currentCard.value.progress[0].card_side,
        selected_answer_text: pending.selectedOption,
        presented_options: pending.options,
        rating,
        hint_used: hintUsed.value,
        presented_type: currentCard.value.flashcard.presented_type,
      })
      await checkForDueChanges()
    }
    pendingGradeCommit.value = null
    if (wasCorrect && rating >= 3 && shouldOfferEncouragement()) {
      // Clear chrome before pep-talk so footer shows encouragement Continue.
      fillinResult.value = null
      quizResult.value = null
      openEncouragement()
      return
    }
    if (quizResult.value) await handleQuizNextCard()
    else await handleNextCard()
  } catch (error) {
    console.error(t('flashcardStudy.submitError'), error)
  } finally {
    isSubmitting.value = false
  }
}

/** Anonymous fill-in / wordbank: check if user answer matches expected. */
function isAnonFillinAnswerCorrect() {
  const card = currentCard.value
  const side = card?.progress?.[0]?.card_side
  const fc = card?.flashcard
  if (!fc || !side) return false
  const expectedRaw = expectedFillinLikeAnswer(fc, side)
  if (!expectedRaw) return false
  const userRaw = userAnswer.value.trim()
  if (!userRaw) return false
  // Chips are already normalized; correct iff selection order matches expected_normalized.
  if (isWordbankMode.value) {
    const expected = wordbankExpectedNormalized.value
    return expected.length > 0 && userRaw === expected
  }
  const user = userRaw.toLowerCase()
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

/** Fetch all due pages for the current level (or open due batch). */
async function fetchDueQueue() {
  if (levelIdParam.value) {
    const all = []
    let page = 1
    const perPage = 100
    for (;;) {
      const response = await getDueCards({
        collection_id: collectionId.value,
        level_id: levelIdParam.value,
        page,
        per_page: perPage,
      })
      const batch = response.data.flashcards || []
      const total = Number(response.data.total || 0)
      if (page === 1 && response.data.level_completed_by_schedule) {
        levelUnlockedBySchedule.value = true
        nextLevelId.value = response.data.next_level_id ?? null
      }
      all.push(...batch)
      page += 1
      if (batch.length === 0 || all.length >= total) break
    }
    return all
  }
  const response = await getDueCards({
    collection_id: collectionId.value,
    per_page: STUDY_SESSION_MAX_CARDS,
  })
  return response.data.flashcards || []
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
      singleCardSession.value = true
      sessionTotal.value = remainingCards.value.length
    } else {
      let all = await fetchDueQueue()
      // First due call for a new guest/user seeds progress then used to return []; retry once.
      if (all.length === 0) {
        all = await fetchDueQueue()
      }
      // Nothing due in this level: the server marked it complete by schedule and unlocked the
      // next level — the finish screen offers "next level" (primary) and "practice again".
      // Otherwise (e.g. old server) fall back to practicing the level's cards early.
      if (all.length === 0 && levelIdParam.value && !levelUnlockedBySchedule.value) {
        all = await fetchLevelPracticeQueue()
        if (all.length > 0) {
          practiceEarly.value = true
          showNewCardsMessage.value = true
          newCardsMessage.value = t('flashcardStudy.practiceEarlyMessage')
        }
      }
      if (isCappedSession.value) all = all.slice(0, studySessionRoom(cardsShown.value))
      remainingCards.value = all
      sessionTotal.value = all.length
    }
    loadNextCard()
  } catch (error) {
    console.error(t('flashcardStudy.loadError'), error)
    // A guest session left over from an earlier deploy can be revoked: the token refresh
    // fails and every due fetch 401s, which used to look like "no due cards". Start a fresh
    // guest session once and reload instead of showing an empty lesson.
    const status = (error as { response?: { status?: number } })?.response?.status
    if (status === 401 && !auth.state.isLoggedIn && !guestRetryDone) {
      guestRetryDone = true
      clearGuestSession()
      guestSessionActive.value = !!(await ensureGuestSession())
      if (guestSessionActive.value) {
        isLoading.value = false
        await loadDueCards(singleCardId)
        return
      }
    }
  } finally {
    isLoading.value = false
  }
}
let guestRetryDone = false
/** Set when a level session found nothing due and fell back to practicing its cards. */
const practiceEarly = ref(false)
/** Server: nothing due in this level, so it was completed by schedule and the next unlocked. */
const levelUnlockedBySchedule = ref(false)
const nextLevelId = ref<number | null>(null)

function goToNextLevel() {
  if (nextLevelId.value == null) return
  // Full navigation: this view loads its queue once per levelId.
  window.location.assign(
    `/collections/${collectionId.value}/lingo/study?levelId=${nextLevelId.value}`
  )
}

/** Secondary option on the "nothing due" screen: practice this level's cards again. */
async function practiceLevelAgain() {
  isLoading.value = true
  try {
    const all = await fetchLevelPracticeQueue()
    levelUnlockedBySchedule.value = false
    practiceEarly.value = true
    remainingCards.value = all
    sessionTotal.value = all.length
    loadNextCard()
  } finally {
    isLoading.value = false
  }
}

/** All cards of the current level (due or not), for the "nothing due" fallback. */
async function fetchLevelPracticeQueue() {
  const all = []
  let page = 1
  for (;;) {
    const response = await getFlashcards({
      collection_id: collectionId.value,
      level_id: levelIdParam.value,
      page,
      per_page: 100,
    })
    const batch = response.data.flashcards || []
    const total = Number(response.data.total || 0)
    all.push(...batch)
    page += 1
    if (batch.length === 0 || all.length >= total) break
  }
  return all
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
        const dir = (c.presented_type || c.direction || 'direct').toString()
        const d = dir.toLowerCase().replace(/_/g, '')
        const cardSide = d.includes('reverse') && !d.includes('both') ? 'reverse' : 'direct'
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
            direction: dir,
            presented_type: c.presented_type,
            wordbank_chips: c.wordbank_chips,
            wordbank_expected_normalized: c.wordbank_expected_normalized,
          },
          progress: [{ card_side: cardSide, status: 'new' }],
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
  showEncouragement.value = false
  if (isCappedSession.value && cardsShown.value >= STUDY_SESSION_MAX_CARDS) {
    // Session cap reached: show the normal finish screen even if more cards are due.
    remainingCards.value = []
    currentCard.value = null
    sessionCapReached.value = true
    return
  }
  if (remainingCards.value.length > 0) {
    currentCard.value = remainingCards.value.shift()
    if (isCappedSession.value) cardsShown.value += 1
    showAnswer.value = false
    userAnswer.value = ''
    fillinResult.value = null
    quizResult.value = null
    pendingGradeCommit.value = null
    selectedQuizOption.value = null
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
/** Selected quiz choice id/text before Check (Duo assist select-then-confirm). */
const selectedQuizOption = ref(null)

/** True when stimulus is definition/back (user recalls the word). */
const isReverseSide = computed(() => currentCard.value?.progress?.[0]?.card_side !== 'direct')

/** Duolingo-style instruction above the card (clone lesson `title` h1). Kept on reveal to avoid CLS. */
const lessonPrompt = computed(() => {
  if (!currentCard.value) return ''
  const reverse = isReverseSide.value
  if (isQuizMode.value && !quizResult.value) {
    return reverse ? t('flashcardStudy.howDoYouSayThis') : t('flashcardStudy.selectOption')
  }
  if (isListenModeActive.value) {
    return t('flashcardStudy.tapWhatYouHear')
  }
  if (isListenTextFallback.value && !fillinResult.value) {
    return t('flashcardStudy.howDoYouSayThis')
  }
  if (isWordbankMode.value && !fillinResult.value) {
    return reverse ? t('flashcardStudy.howDoYouSayThis') : t('flashcardStudy.assembleAnswer')
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

/** Normalized choice rows for the assist-style list (text or image). */
const quizChoiceItems = computed(() => {
  if (!isQuizMode.value) return []
  const images = quizImageOptions.value
  if (images?.length) {
    return images.map((o) => ({ id: o.id, label: o.id, imageUrl: o.imageUrl }))
  }
  const opts = currentCard.value?.flashcard?.quiz_options
  if (!opts?.length) return []
  return opts.map((text) => ({ id: text, label: text, imageUrl: null }))
})

function selectQuizOption(optionId) {
  if (quizResult.value || isSubmitting.value) return
  selectedQuizOption.value = optionId
}

/** Correct option text for post-check highlighting (anon: first stored option). */
const quizCorrectOptionId = computed(() => {
  if (!quizResult.value) return null
  if (quizResult.value.correct) return selectedQuizOption.value
  const msg = String(quizResult.value.message || '')
  const marker = 'The correct answer was: '
  const idx = msg.indexOf(marker)
  if (idx >= 0) {
    const fromMsg = msg.slice(idx + marker.length).trim()
    const match = quizChoiceItems.value.find(
      (o) =>
        String(o.id).trim().toLowerCase() === fromMsg.toLowerCase() ||
        String(o.id).split(':')[0] === fromMsg ||
        fromMsg.startsWith(String(o.id))
    )
    if (match) return match.id
  }
  const first = currentCard.value?.flashcard?.quiz_options?.[0]
  return first ?? null
})

function quizChoiceClass(optionId) {
  const selected = selectedQuizOption.value === optionId
  if (!quizResult.value) {
    return selected ? 'lingo-study-quiz-choice--selected' : ''
  }
  const correctId = quizCorrectOptionId.value
  const isCorrectChoice =
    correctId != null &&
    String(optionId).trim().toLowerCase() === String(correctId).trim().toLowerCase()
  if (isCorrectChoice) return 'lingo-study-quiz-choice--correct'
  if (selected) return 'lingo-study-quiz-choice--wrong'
  return ''
}

const submitAnswer = async (rating?: number) => {
  if (!currentCard.value || isSubmitting.value) return
  isSubmitting.value = true

  try {
    const next = cardsAnsweredInSession.value + 1
    cardsAnsweredInSession.value = next
    bumpGuestAnswersThisVisit()
    try {
      sessionStorage.setItem(ANON_BANNER_ANSWERED_KEY, String(next))
    } catch (_) {}
    if (isAnonStudy.value) {
      let correct
      if (isFillInLikeMode.value) {
        if (isWordbankMode.value) syncUserAnswerFromWordbank()
        correct = isAnonFillinAnswerCorrect()
        fillinResult.value = { correct, message: correct ? t('flashcardStudy.correctAnswer') : '' }
        pendingGradeCommit.value = {
          kind: 'fillin',
          answer: userAnswer.value.trim(),
          correct,
        }
        // SFX now; anon XP/hearts defer to Continue / Too easy / Too difficult.
        if (correct) registerCorrectAnswer()
        else registerWrongAnswer()
        await nextTick()
        if (!isJustInformationMode.value) answerAudioPlayerRef.value?.play()
      } else {
        correct = typeof rating === 'number' ? rating >= 3 : true
        applyAnonLevelProgress(correct)
        if (correct) registerCorrectAnswer()
        else registerWrongAnswer()
        // SFX already played on Check; advance after rate (maybe pep-talk first).
        showAnswer.value = false
        if (correct && shouldOfferEncouragement()) openEncouragement()
        else loadNextCard()
      }
      isSubmitting.value = false
      return
    }
    if (isFillInLikeMode.value) {
      if (isWordbankMode.value) syncUserAnswerFromWordbank()
      const answer = userAnswer.value.trim()
      // Check: grade only (no `rating` → no FSRS commit).
      const response = await submitFillinAnswer({
        flashcard_id: currentCard.value.flashcard.id,
        card_side: currentCard.value.progress[0].card_side,
        answer,
      })
      fillinResult.value = response.data
      // `success` = request OK (always true on 200); grading lives in `answer_correct`.
      const ok = response.data?.answer_correct === true || response.data?.correct === true
      pendingGradeCommit.value = {
        kind: 'fillin',
        answer,
        correct: ok,
        suggestedRating: response.data?.suggested_rating,
      }
      if (ok) registerCorrectAnswer()
      else registerWrongAnswer()
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
        hint_used: hintUsed.value,
        presented_type: currentCard.value.flashcard.presented_type,
      })
      if (rating >= 3 && shouldOfferEncouragement()) openEncouragement()
      else loadNextCard()
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
    bumpGuestAnswersThisVisit()
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
      pendingGradeCommit.value = {
        kind: 'quiz',
        selectedOption,
        options,
        correct,
      }
      if (correct) registerCorrectAnswer()
      else registerWrongAnswer()
    } else {
      // Check: grade only (no `rating` → no FSRS commit).
      const response = await submitQuizAnswer({
        flashcard_id: fc.id,
        card_side: currentCard.value.progress[0].card_side,
        selected_answer_text: selectedOption,
        presented_options: options,
      })
      quizResult.value = response.data
      pendingGradeCommit.value = {
        kind: 'quiz',
        selectedOption,
        options,
        correct: !!response.data?.correct,
        suggestedRating: response.data?.suggested_rating,
      }
      if (response.data?.correct) registerCorrectAnswer()
      else registerWrongAnswer()
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
  selectedQuizOption.value = null
  pendingGradeCommit.value = null
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
  pendingGradeCommit.value = null
  userAnswer.value = ''
  wordbankSelectedIds.value = []
  wordbankChips.value = []
  wordbankExpectedNormalized.value = ''
  loadNextCard()
  if (!isAnonStudy.value) await checkForDueChanges()
}

const newCardsMessage = ref('')
const showNewCardsMessage = ref(false)

async function checkForDueChanges() {
  // Level sessions load the full queue up front — empty means the lesson is done.
  if (levelIdParam.value) return
  if (remainingCards.value.length > 0 || currentCard.value) return
  if (sessionCapReached.value) return
  const room = studySessionRoom(cardsShown.value)
  if (room <= 0) return

  try {
    const response = await getDueCards({
      collection_id: collectionId.value,
      per_page: Math.min(room, STUDY_SESSION_MAX_CARDS),
    })
    const batch = (response.data.flashcards || []).slice(0, room)
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
    rebuildWordbankForCard()
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
  if (newValue && !isFillInLikeMode.value) {
    // Focus rating buttons container or first button? For now, just log.
    // Consider focusing the "Forgot" button as a default.
  } else if (!newValue && !isFillInLikeMode.value) {
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
  if (isTypingInEditable(event)) return

  if (event.key === ' ' || event.key === 'Enter') {
    event.preventDefault()
    if (showEncouragement.value) {
      continueFromEncouragement()
      return
    }
    if (showNoListening.value) {
      continueFromNoListening()
      return
    }
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
    if (showAnswer.value && !isFillInLikeMode.value) return // rating buttons use 1/2/3
    if (!studyFooterDisabled.value) onFooterCheck()
  } else if (
    isQuizMode.value &&
    !quizResult.value &&
    !showEncouragement.value &&
    /^[1-9]$/.test(event.key)
  ) {
    const idx = Number(event.key) - 1
    const item = quizChoiceItems.value[idx]
    if (item) selectQuizOption(item.id)
  } else if (showAnswer.value && !isFillInLikeMode.value && !showEncouragement.value) {
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
.lingo-study-info__title {
  font-weight: 700;
  font-size: 1.05em;
  margin-bottom: 0.4rem;
}
.lingo-study-info :deep(ul) {
  margin: 0.25rem 0;
  padding-left: 1.1rem;
}
</style>
