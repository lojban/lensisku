<template>
  <!-- Session Header -->
  <div class="bg-white border rounded-lg p-4 mb-6">
    <div class="flex flex-wrap justify-between items-center gap-4">
      <div>
        <h2 class="text-xl font-bold text-gray-800">
          {{ t('flashcardStudy.title') }}
        </h2>
        <p v-if="showNewCardsMessage" class="text-sm text-orange-600 font-medium mt-1">
          {{ t('flashcardStudy.newCardsMessage') }}
        </p>
        <p v-else class="text-sm text-gray-600 mt-1">
          {{ t('flashcardStudy.remainingCards', { count: remainingCards.length }) }}
        </p>
      </div>
      <div class="flex gap-4 space-x-4">
        <button class="btn-cancel" @click="router.back()">
          {{ t('flashcardStudy.endSession') }}
        </button>
        <button v-if="currentCard" class="btn-empty" @click="snoozeCard">
          {{ t('flashcardStudy.snooze') }}
        </button>
      </div>
    </div>
  </div>

  <!-- Loading State -->
  <div v-if="isLoading" class="flex justify-center py-8">
    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
  </div>

  <!-- No Cards State -->
  <div v-else-if="!currentCard && remainingCards.length === 0" class="text-center py-12 bg-white rounded-lg border">
    <h3 class="text-lg font-medium text-gray-800 mb-2">
      {{ t('flashcardStudy.allCaughtUp') }}
    </h3>
    <p class="text-gray-600 mb-4">
      {{ t('flashcardStudy.allReviewed') }}
    </p>
    <div class="flex justify-center">
      <button ref="returnToDeckButtonRef" class="btn-get w-auto h-10 text-base shadow-sm"
        @click="router.push(returnToUrl)">
        {{ isAnonLevelMode ? t('flashcardStudy.returnToLevels', 'Back to levels') : t('flashcardStudy.returnToDeck') }}
      </button>
    </div>
  </div>

  <!-- Current Card -->
  <div v-else-if="currentCard" class="flex flex-col gap-4">
    <!-- Card Display -->
    <div class="bg-white border rounded-lg p-4 sm:p-6">
      <!-- Progress indicator -->
      <div v-for="progress in currentCard.progress" :key="progress.card_side"
        class="flex justify-between items-center mb-4 text-xs sm:text-sm text-gray-600">
        <span>{{ t('flashcardStudy.cardOfTotal', { current: totalCards - remainingCards.length, total: totalCards })
          }}</span>
        <span :class="getStatusClass(progress.status)">
          {{ progress.card_side }}: {{ progress.status }}
        </span>
      </div>

      <!-- Card content -->
      <div class="flex flex-col gap-4">
        <div v-if="currentCard.flashcard.definition_language_id" class="text-sm text-gray-600 text-center">
          {{ t('flashcardStudy.definitionLanguage', {
            language:
              getLanguageName(currentCard.flashcard.definition_language_id) }) }}
        </div>

        <!-- Canonical Comparison Indicator for Fill-in Mode -->
        <div v-if="isFillInMode && currentCard.flashcard.use_canonical_comparison" 
          class="flex items-center justify-center gap-2 text-xs text-green-600 bg-green-50 border border-green-200 rounded px-3 py-1.5">
          <CheckCircle2 class="h-3.5 w-3.5" />
          <span>{{ t('flashcardStudy.canonicalComparisonEnabled', 'Canonical comparison enabled') }}</span>
        </div>

        <!-- Question -->
        <!-- Display Word/Front for 'direct' side -->
        <div v-if="currentCard.progress[0].card_side === 'direct'">
          <div class="flex items-center justify-center gap-2">
            <h3 class="text-2xl font-bold text-gray-800">
              {{ currentCard.flashcard.word ?? currentCard.flashcard.free_content_front }}
            </h3>
            <AudioPlayer v-if="currentCard.flashcard.sound_url" :url="currentCard.flashcard.sound_url"
              :collection-id="currentCard.flashcard.sound_url?.startsWith?.('/api/') ? currentCard.flashcard.collection_id : undefined"
              :item-id="currentCard.flashcard.sound_url?.startsWith?.('/api/') ? currentCard.flashcard.item_id : undefined"
              class="h-6 w-6" />
          </div>
          <div v-if="currentCard.flashcard.has_front_image" class="mt-4 flex justify-center">
            <img
              :src="`/api/collections/${currentCard.flashcard.collection_id}/items/${currentCard.flashcard.item_id}/image/front`"
              class="max-h-48 rounded-lg object-contain bg-gray-100" alt="Front image">
          </div>
        </div>
        <!-- Display Definition/Back for 'reverse' side -->
        <div v-else>
          <div class="text-center text-gray-800 text-2xl">
            <LazyMathJax :content="currentCard.flashcard.definition ?? currentCard.flashcard.free_content_back" />
            <div v-if="currentCard.flashcard.has_back_image" class="mt-4 flex justify-center">
              <img
                :src="`/api/collections/${currentCard.flashcard.collection_id}/items/${currentCard.flashcard.item_id}/image/back`"
                class="max-h-48 rounded-lg object-contain bg-gray-100" alt="Back image">
            </div>
          </div>
        </div>

        <!-- Answer Input (for fill-in modes) -->
        <div v-if="isFillInMode" class="mt-4">
          <textarea ref="fillInTextareaRef" v-model="userAnswer" type="text" rows="1"
            class="textarea-field w-full text-center text-xl" :placeholder="t('flashcardStudy.typeAnswer')"
            @keydown.enter.prevent="submitAnswer" :disabled="!!fillinResult" />
        </div>

        <!-- Answer Display (shown after revealing or submitting fill-in) -->
        <div v-if="showAnswer || fillinResult" class="flex flex-col gap-4 pt-4 border-t">
          <div class="prose max-w-none text-center text-lg">
            <h4 class="text-sm text-center text-gray-700 mb-2">
              {{ t('flashcardStudy.correctAnswer') }}
            </h4>
            <!-- Show the OTHER side as the correct answer -->
            <template v-if="currentCard.progress[0].card_side === 'direct'">
              <!-- If question was front/word, show back/definition -->
              <LazyMathJax :content="currentCard.flashcard.definition ?? currentCard.flashcard.free_content_back" />
              <div v-if="currentCard.flashcard.has_back_image" class="mt-4 flex justify-center">
                <img
                  :src="`/api/collections/${currentCard.flashcard.collection_id}/items/${currentCard.flashcard.item_id}/image/back`"
                  class="max-h-48 rounded-lg object-contain bg-gray-100" alt="Back image">
              </div>
            </template>
            <template v-else>
              <!-- If question was back/definition, show front/word -->
              <div class="flex items-center justify-center gap-2">
                <span>{{ currentCard.flashcard.word ?? currentCard.flashcard.free_content_front }}</span>
                <AudioPlayer ref="answerAudioPlayerRef" v-if="currentCard.flashcard.sound_url"
                  :url="currentCard.flashcard.sound_url"
                  :collection-id="currentCard.flashcard.sound_url?.startsWith?.('/api/') ? currentCard.flashcard.collection_id : undefined"
                  :item-id="currentCard.flashcard.sound_url?.startsWith?.('/api/') ? currentCard.flashcard.item_id : undefined"
                  class="h-6 w-6 inline-block" />
              </div>
              <div v-if="showCanonicalForm" class="mt-3 pt-3 border-t flex flex-col gap-1.5 text-center">
                <div class="flex items-center justify-center gap-2 text-xs font-semibold text-gray-400 uppercase tracking-wider">
                  <EqualApproximately class="h-3.5 w-3.5 text-blue-400" />
                  <span>{{ t('components.definitionCard.canonicalLabel') }}</span>
                </div>
                <div class="text-sm text-gray-700 font-mono bg-blue-50/30 p-2 rounded border border-blue-100/30 inline-block mx-auto">
                  {{ currentCard.flashcard.canonical_form }}
                </div>
              </div>
              <div v-if="currentCard.flashcard.has_front_image" class="mt-4 flex justify-center">
                <img
                  :src="`/api/collections/${currentCard.flashcard.collection_id}/items/${currentCard.flashcard.item_id}/image/front`"
                  class="max-h-48 rounded-lg object-contain bg-gray-100" alt="Front image">
              </div>
            </template>
          </div>

          <!-- Display Notes only when showing the definition side -->
          <div v-if="currentCard.flashcard.notes && currentCard.progress[0].card_side === 'direct'">
            <h4 class="text-sm font-medium text-gray-700 mb-2">
              {{ t('flashcardStudy.notes') }}
            </h4>
            <LazyMathJax :content="currentCard.flashcard.notes" :enable-markdown="true" />
          </div>
        </div>
      </div>
    </div>

    <!-- Review Controls -->
    <div v-if="!showAnswer && !fillinResult && !isJustInformationMode" class="flex justify-center px-4">
      <!-- Show Submit button for fill-in modes -->
      <button v-if="isFillInMode" class="btn-get w-auto h-10 text-base shadow-sm" @click="submitAnswer()">
        {{ t('flashcardStudy.submitAnswer') }}
      </button>
      <!-- Show "Show Answer" button for non-fill-in modes -->
      <button v-else ref="showAnswerButtonRef" class="btn-get w-auto h-10 text-base shadow-sm"
        @click="revealAnswerAndPlayAudio">
        {{ t('flashcardStudy.showAnswer') }}
      </button>
    </div>
    <!-- OK button for JustInformation mode -->
    <div v-else-if="isJustInformationMode && !showAnswer" class="flex justify-center px-4">
      <button class="btn-get w-auto h-10 text-base shadow-sm" @click="submitAnswer(4)">
        <Check class="h-4 w-4" />
      </button>
    </div>

    <!-- Rating buttons (for non-fill-in modes after showing answer) -->
    <div v-else-if="showAnswer && !isFillInMode && !isJustInformationMode"
      class="bg-white border rounded-lg p-4 sm:p-6">
      <h4 class="text-lg font-medium text-center text-gray-700 mb-6">
        {{ t('flashcardStudy.howWellRemembered') }}
      </h4>

      <div class="flex justify-center px-4 sm:px-6">
        <div class="w-full max-w-xl">
          <div class="grid grid-cols-3 gap-2 sm:gap-4">
            <button class="btn-error w-full sm:min-w-[120px] flex items-center justify-center gap-1.5"
              @click="submitAnswer(1)">
              <XCircle class="h-4 w-4" />
              {{ t('flashcardStudy.forgot') }}<span class="hidden sm:inline ml-1">(1)</span>
            </button>
            <button class="btn-warning w-full sm:min-w-[120px] flex items-center justify-center gap-1.5"
              @click="submitAnswer(3)">
              <Smile class="h-4 w-4" />
              {{ t('flashcardStudy.good') }}<span class="hidden sm:inline ml-1">(2)</span>
            </button>
            <button class="btn-success w-full sm:min-w-[120px] flex items-center justify-center gap-1.5"
              @click="submitAnswer(4)">
              <Check class="h-4 w-4" />
              {{ t('flashcardStudy.easy') }}<span class="hidden sm:inline ml-1">(3)</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Next Card button (for fill-in modes after submitting) -->
    <div v-else-if="fillinResult" class="flex justify-center px-4">
      <div class="flex flex-col gap-2 mt-4">
        <AlertComponent :type="fillinResult.correct ? 'success' : 'error'">
          {{ fillinResult.message }}
        </AlertComponent>
        <button v-if="remainingCards.length <= 0" class="btn-get w-auto h-10 text-base shadow-sm"
          @click="router.back()">
          {{ t('flashcardStudy.endSession') }}
        </button>
        <button v-else ref="nextCardButtonRef" class="btn-get w-auto h-10 text-base shadow-sm" @click="handleNextCard">
          {{ t('flashcardStudy.nextCard') }}
        </button>
        <div v-if="remainingCards.length === 0" class="text-center text-gray-600 mt-2">
          {{ t('flashcardStudy.thanksSession') }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { XCircle, Check, Smile, CheckCircle2, EqualApproximately } from 'lucide-vue-next'
import { ref, onMounted, computed, watch, nextTick, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import AlertComponent from '@/components/AlertComponent.vue'

import { getDueCards, reviewFlashcard, getLanguages, submitFillinAnswer, getFlashcards, snoozeFlashcard, getLevelCards, getLevels } from '@/api'
import LazyMathJax from '@/components/LazyMathJax.vue'
import AudioPlayer from '@/components/AudioPlayer.vue'
import { useSeoHead } from '@/composables/useSeoHead'
import { useAuth } from '@/composables/useAuth'
import { useAnonymousProgress } from '@/composables/useAnonymousProgress'

const { t, locale } = useI18n()

const route = useRoute()
const router = useRouter()
const auth = useAuth()
const { getProgress, saveLevelProgress } = useAnonymousProgress()

const levelIdParam = computed(() => route.query.levelId ? parseInt(route.query.levelId, 10) : null)
const isAnonLevelMode = computed(() => !auth.state.isLoggedIn && levelIdParam.value != null)
const anonLevelMeta = ref(null) // { min_cards, min_success_rate } when in anon level mode

const returnToUrl = computed(() =>
  isAnonLevelMode.value
    ? `/collections/${route.params.collectionId}/levels`
    : `/collections/${route.params.collectionId}/flashcards`
)

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

const languages = ref([])

const isFillInMode = computed(() => {
  const dir = currentCard.value?.flashcard?.direction
  return dir && dir.toLowerCase().includes('fillin')
})

const isJustInformationMode = computed(() => {
  const dir = currentCard.value?.flashcard?.direction
  return dir && dir.toLowerCase() === 'justinformation'
})

const showCanonicalForm = computed(() => {
  const fc = currentCard.value?.flashcard
  if (!fc?.canonical_form) return false
  const main = (fc.word ?? fc.free_content_front ?? '').trim().toLowerCase()
  const canonical = fc.canonical_form.trim().toLowerCase()
  return main !== canonical
})

const getLanguageName = (langId) => {
  const lang = languages.value.find((l) => l.id === langId)
  return lang?.real_name || 'Unknown'
}

const loadDueCards = async (singleCardId = null) => {
  isLoading.value = true
  try {
    let response
    if (singleCardId) {
      // Fetch a single specific card for review
      response = await getFlashcards({
        collection_id: route.params.collectionId,
        flashcard_id: singleCardId,
        per_page: 1, // Ensure we only get one
      })
      // Adapt the response structure if necessary, assuming it returns a list
      remainingCards.value = response.data.flashcards
      totalCards.value = 1 // Only one card in this session
    } else {
      // Fetch all due cards
      response = await getDueCards({
        collection_id: route.params.collectionId,
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
  const cid = route.params.collectionId
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
            direction: 'direct'
          },
          progress: [{ card_side: 'direct', status: 'new' }]
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

const applyAnonLevelProgress = (correct) => {
  const cid = route.params.collectionId
  const lid = levelIdParam.value
  if (!cid || !lid) return
  const cur = getProgress(cid, lid) || { cards_completed: 0, correct_answers: 0, total_answers: 0 }
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
    fillinResult.value = null // Reset fill-in result
  } else {
    currentCard.value = null
  }
}

const userAnswer = ref('')
const isSubmitting = ref(false)
const fillinResult = ref(null)

const submitAnswer = async (rating) => {
  if (!currentCard.value || isSubmitting.value) return
  isSubmitting.value = true

  try {
    if (isAnonLevelMode.value) {
      const correct = typeof rating === 'number' ? rating >= 3 : true
      applyAnonLevelProgress(correct)
      if (isFillInMode.value) {
        fillinResult.value = { correct, message: correct ? t('flashcardStudy.correctAnswer') : '' }
        await nextTick()
        if (!isJustInformationMode.value) answerAudioPlayerRef.value?.play()
      } else {
        loadNextCard()
      }
      isSubmitting.value = false
      return
    }
    if (isFillInMode.value) {
      const response = await submitFillinAnswer({
        flashcard_id: currentCard.value.flashcard.id,
        card_side: currentCard.value.progress[0].card_side,
        answer: userAnswer.value.trim()
      })
      fillinResult.value = response.data
      await checkForDueChanges()
      await nextTick()
      if (!isJustInformationMode.value) answerAudioPlayerRef.value?.play()
    } else {
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

const snoozeCard = async () => {
  if (!currentCard.value || isSubmitting.value) return
  isSubmitting.value = true
  try {
    if (isAnonLevelMode.value) {
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
  if (!isJustInformationMode.value) { // Don't play audio for justinformation cards on reveal
    await nextTick() // Wait for the answer section to render
    answerAudioPlayerRef.value?.play() // Play audio when answer is revealed
  }
}

const handleNextCard = async () => {
  fillinResult.value = null
  userAnswer.value = ''
  loadNextCard()
  if (!isAnonLevelMode.value) await checkForDueChanges()
}

const newCardsMessage = ref('')
const showNewCardsMessage = ref(false)

async function checkForDueChanges() {
  // Check if we need to refresh due cards if the current queue is empty
  if (remainingCards.value.length === 0) {
    await loadDueCards() // This might repopulate remainingCards
    if (remainingCards.value.length > 0) { // Check again after loading
      showNewCardsMessage.value = true
      newCardsMessage.value = t('flashcardStudy.newCardsMessage')
      // If new cards were loaded and the session seemed over, load the first new card
      if (!currentCard.value) {
        loadNextCard();
      }
    }
  }
}

// --- Focus and Keyboard Handling ---

// Watch for changes in the current card to focus the textarea if it's a fill-in type
watch(currentCard, (newCard) => {
  if (newCard && isFillInMode.value && !fillinResult.value) {
    nextTick(() => {
      fillInTextareaRef.value?.focus()
    })
  }
}, { immediate: true }) // immediate: true to run on initial load if applicable
watch(currentCard, (newCard) => {
  if (newCard && isFillInMode.value && !fillinResult.value) {
    nextTick(() => {
      fillInTextareaRef.value?.focus()
    })
  }
}, { immediate: true })

// Watch for end of session to focus the "Return to Deck" button
watch([currentCard, remainingCards], ([newCard, newRemaining]) => {
  if (!newCard && newRemaining.length === 0) {
    nextTick(() => {
      returnToDeckButtonRef.value?.focus()
    })
  }
}, { deep: true })

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
    } else if (!currentCard.value && remainingCards.value.length === 0 && returnToDeckButtonRef.value && !isJustInformationMode.value) {
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

const getStatusClass = (status) => {
  const classes = {
    new: 'text-blue-600 bg-blue-50 px-2 py-1 rounded-full text-xs',
    learning: 'text-yellow-600 bg-yellow-50 px-2 py-1 rounded-full text-xs',
    review: 'text-green-600 bg-green-50 px-2 py-1 rounded-full text-xs',
    graduated: 'text-purple-600 bg-purple-50 px-2 py-1 rounded-full text-xs',
  }
  return classes[status.toLowerCase()] || ''
}

useSeoHead({ title: t('flashcardStudy.title') }, locale.value)

onMounted(async () => {
  try {
    const langsResponse = await getLanguages()
    languages.value = langsResponse.data

    if (isAnonLevelMode.value) {
      await loadLevelCardsForAnon()
      return
    }
    if (!auth.state.isLoggedIn) {
      router.replace(`/collections/${route.params.collectionId}/levels`)
      return
    }
    const cardIdToLoad = route.query.card_id ? parseInt(route.query.card_id) : null
    await loadDueCards(cardIdToLoad)
  } catch (error) {
    console.error(t('flashcardStudy.loadInitialError'), error)
  }
})
</script>
