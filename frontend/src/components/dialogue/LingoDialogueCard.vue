<template>
  <div class="dialogue-card">
    <LoadingSpinner v-if="loading" size="md" />
    <div v-else-if="loadError" class="dialogue-card__error" role="alert">{{ loadError }}</div>
    <template v-else-if="view">
      <div class="dialogue-card__toolbar">
        <h2 v-if="view.dialogue.title" class="dialogue-card__title">{{ view.dialogue.title }}</h2>
        <div class="dialogue-card__tools">
          <button
            v-if="anyAudio"
            type="button"
            class="lingo-btn lingo-btn--default lingo-btn--sm !normal-case !tracking-normal"
            @click="replayAll"
          >
            <Volume2 class="h-4 w-4" aria-hidden="true" />
            {{ t('dialogue.replay') }}
          </button>
          <button
            type="button"
            class="lingo-btn lingo-btn--default lingo-btn--sm !normal-case !tracking-normal"
            :aria-pressed="showTranscript"
            @click="showTranscript = !showTranscript"
          >
            <component :is="showTranscript ? EyeOff : Eye" class="h-4 w-4" aria-hidden="true" />
            {{ showTranscript ? t('dialogue.hideTranscript') : t('dialogue.showTranscript') }}
          </button>
        </div>
      </div>

      <div ref="scrollRef" class="dialogue-card__lines" aria-live="polite">
        <DialogueLineBubble
          v-for="line in shownLines"
          :key="line.index"
          :line="line"
          :side="sideOf(line.speaker)"
          :show-text="showTranscript"
          :has-audio="hasAudio(line.index)"
          :playing="audio.playingLine.value === line.index"
          @play="audio.play(line.index, audioVersion)"
          @gloss="onGloss"
        />
      </div>

      <DialogueQuestionPanel
        v-if="phase === 'questions' && currentQuestion"
        :key="currentQuestion.index"
        :question="currentQuestion"
        :total="view.dialogue.questions.length"
        :chips="chipsFor(currentQuestion.index)"
        :result="results[currentQuestion.index] ?? null"
        :busy="busy"
        @submit="onSubmit"
        @next="nextQuestion"
      />

      <div v-if="phase === 'lines'" class="dialogue-card__footer">
        <button
          type="button"
          class="lingo-btn lingo-btn--primary lingo-btn--sm w-full !normal-case !tracking-normal"
          :disabled="busy"
          @click="advanceLine"
        >
          {{ revealed === 0 ? t('dialogue.start') : t('flashcardStudy.continue') }}
        </button>
      </div>
      <div v-else-if="phase === 'finished'" class="dialogue-card__footer">
        <p class="dialogue-card__summary">
          {{
            t('dialogue.summary', { correct: correctCount, total: view.dialogue.questions.length })
          }}
        </p>
        <button
          v-if="mode === 'preview'"
          type="button"
          class="lingo-btn lingo-btn--default lingo-btn--sm w-full !normal-case !tracking-normal"
          @click="restart"
        >
          {{ t('dialogue.restart') }}
        </button>
      </div>
      <p v-if="submitError" class="dialogue-card__error" role="alert">{{ submitError }}</p>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Eye, EyeOff, Volume2 } from '@lucide/vue'
import LoadingSpinner from '@/components/LoadingSpinner.vue'
import { answerDialogue, checkDialogueAnswer, getDialogue } from '@/api'
import { useDialogueAudio } from '@/composables/useDialogueAudio'
import DialogueLineBubble from './DialogueLineBubble.vue'
import DialogueQuestionPanel from './DialogueQuestionPanel.vue'
import { NARRATOR, gradeDialogueLocally, type DialogueGrade, type DialogueView } from './types'

const props = withDefaults(
  defineProps<{
    /** study = logged-in/guest-session (scores recorded); guest = grade only; preview = editor. */
    mode: 'study' | 'guest' | 'preview'
    itemId?: number | null
    flashcardId?: number | null
    /** Editor preview passes the parsed dialogue directly. */
    previewView?: DialogueView | null
    /** Bump to refetch clips after regeneration. */
    audioVersion?: number
  }>(),
  { itemId: null, flashcardId: null, previewView: null, audioVersion: 0 }
)
const emit = defineEmits<{ answered: [correct: boolean]; done: [rating: number] }>()
const { t } = useI18n()

const view = ref<DialogueView | null>(props.previewView)
const loading = ref(false)
const loadError = ref('')
const submitError = ref('')
const busy = ref(false)
const revealed = ref(0)
const questionPos = ref(0)
const showTranscript = ref(true)
const results = ref<Record<number, DialogueGrade>>({})
const glossSinceAnswer = ref(false)
const attemptId = ref(crypto.randomUUID())
const scrollRef = ref<HTMLElement | null>(null)

const audio = useDialogueAudio(() => view.value?.item_id ?? props.itemId)

const lines = computed(() => view.value?.dialogue.lines ?? [])
const questions = computed(() => view.value?.dialogue.questions ?? [])
const shownLines = computed(() => lines.value.slice(0, revealed.value))
const phase = computed<'lines' | 'questions' | 'finished'>(() => {
  if (revealed.value < lines.value.length) return 'lines'
  if (questionPos.value < questions.value.length) return 'questions'
  return 'finished'
})
const currentQuestion = computed(() => questions.value[questionPos.value] ?? null)
const correctCount = computed(() => Object.values(results.value).filter((r) => r.correct).length)
const anyAudio = computed(() => (view.value?.lines ?? []).some((l) => l.has_audio))

const sides = computed(() => {
  const order: string[] = []
  for (const l of lines.value) {
    if (l.speaker !== NARRATOR && !order.includes(l.speaker)) order.push(l.speaker)
  }
  return order
})
function sideOf(speaker: string): 'left' | 'right' {
  return sides.value.indexOf(speaker) % 2 === 1 ? 'right' : 'left'
}
function hasAudio(index: number) {
  return !!view.value?.lines.find((l) => l.index === index)?.has_audio
}
function chipsFor(index: number) {
  return view.value?.bank_chips.find((b) => b.question_index === index)?.chips ?? []
}

async function load() {
  if (props.previewView || props.itemId == null) return
  loading.value = true
  loadError.value = ''
  try {
    const res = await getDialogue(props.itemId)
    view.value = res.data as DialogueView
  } catch {
    loadError.value = t('dialogue.loadFailed')
  } finally {
    loading.value = false
  }
}

async function scrollDown() {
  await nextTick()
  const el = scrollRef.value
  if (el) el.scrollTop = el.scrollHeight
}

function onGloss() {
  glossSinceAnswer.value = true
  void scrollDown()
}

function advanceLine() {
  const line = lines.value[revealed.value]
  revealed.value += 1
  void scrollDown()
  if (line && hasAudio(line.index)) void audio.play(line.index, props.audioVersion)
  if (phase.value === 'finished') void finish()
}

watch(shownLines, () => void scrollDown())

function replayAll() {
  void audio.playAll(
    shownLines.value.filter((l) => hasAudio(l.index)).map((l) => l.index),
    props.audioVersion
  )
}

async function onSubmit(answer: string) {
  const q = currentQuestion.value
  if (!q || !view.value) return
  busy.value = true
  submitError.value = ''
  const hint = glossSinceAnswer.value
  try {
    let grade: DialogueGrade
    if (props.mode === 'preview') {
      grade = gradeDialogueLocally(q, answer)
    } else if (props.mode === 'guest' || props.flashcardId == null) {
      const res = await checkDialogueAnswer(view.value.item_id ?? props.itemId!, {
        question_index: q.index,
        answer,
      })
      grade = res.data as DialogueGrade
    } else {
      const res = await answerDialogue(props.flashcardId, {
        attempt_id: attemptId.value,
        question_index: q.index,
        answer,
        hint_used: hint,
        finish: false,
      })
      grade = res.data as DialogueGrade
    }
    if (!grade.rating) grade.rating = grade.correct ? (hint ? 2 : 3) : 1
    results.value = { ...results.value, [q.index]: grade }
    glossSinceAnswer.value = false
    emit('answered', grade.correct)
  } catch {
    submitError.value = t('dialogue.submitFailed')
  } finally {
    busy.value = false
  }
}

function nextQuestion() {
  questionPos.value += 1
  if (phase.value === 'finished') void finish()
}

/** One FSRS review per dialogue: the weakest question's rating (3 when there are none). */
async function finish() {
  const ratings = Object.values(results.value).map((r) => r.rating)
  let rating = ratings.length ? Math.min(...ratings) : 3
  if (props.mode === 'study' && props.flashcardId != null) {
    busy.value = true
    try {
      const res = await answerDialogue(props.flashcardId, {
        attempt_id: attemptId.value,
        question_index: null,
        answer: '',
        hint_used: false,
        finish: true,
      })
      rating = res.data?.card_rating ?? rating
    } catch {
      submitError.value = t('dialogue.submitFailed')
      busy.value = false
      return
    }
    busy.value = false
  }
  emit('done', rating)
}

function restart() {
  audio.stop()
  revealed.value = 0
  questionPos.value = 0
  results.value = {}
  glossSinceAnswer.value = false
  attemptId.value = crypto.randomUUID()
}

watch(
  () => props.previewView,
  (v) => {
    if (!v) return
    view.value = v
    if (revealed.value > v.dialogue.lines.length) revealed.value = v.dialogue.lines.length
  }
)
watch(
  () => props.itemId,
  () => {
    restart()
    void load()
  }
)
watch(
  () => props.audioVersion,
  () => audio.clear()
)

onMounted(load)

defineExpose({ restart, revealAll: () => (revealed.value = lines.value.length) })
</script>

<style scoped>
.dialogue-card {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  width: 100%;
  max-width: 40rem;
  margin: 0 auto;
  min-height: 0;
}
.dialogue-card__toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}
.dialogue-card__title {
  font-size: 1.15rem;
  font-weight: 800;
}
.dialogue-card__tools {
  display: flex;
  gap: 0.4rem;
  margin-inline-start: auto;
}
.dialogue-card__lines {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  max-height: 50vh;
  overflow-y: auto;
  padding: 0.25rem;
}
.dialogue-card__footer {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.dialogue-card__summary {
  text-align: center;
  font-weight: 700;
}
.dialogue-card__error {
  color: #b42323;
  font-size: 0.9rem;
}
</style>
