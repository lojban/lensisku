<template>
  <footer
    class="lingo-study-footer sticky bottom-0 z-30 w-full shrink-0 border-t-2 transition-colors"
    :class="[
      footerToneClass,
      compact ? 'lingo-study-footer--compact' : 'lingo-study-footer--roomy',
    ]"
  >
    <div class="lingo-study-footer-chrome">
      <!--
        Graded result: the same h-9 three-cell grid as Forgot/Good/Easy, so the footer keeps
        the idle Check height (no CLS). The feedback copy moves to the second (meta) row.
      -->
      <div
        v-if="status === 'correct' || status === 'wrong'"
        class="lingo-study-rate-row"
        role="group"
      >
        <!-- Wrong answers are server-graded (stored as 1): no Too difficult / Too easy. -->
        <button
          v-if="status === 'correct'"
          type="button"
          class="lingo-btn lingo-btn--danger lingo-btn--sm w-full !normal-case !tracking-normal"
          :disabled="disabled"
          @click="$emit('rate', 1)"
        >
          <XCircle class="hidden h-3.5 w-3.5 shrink-0 min-[400px]:block" />
          <span class="truncate">{{
            labelTooDifficult || labelForgot || t('flashcardStudy.forgot')
          }}</span>
        </button>
        <button
          v-if="status === 'correct'"
          type="button"
          class="lingo-btn lingo-btn--primary lingo-btn--sm w-full !normal-case !tracking-normal"
          :disabled="disabled"
          @click="$emit('rate', 4)"
        >
          <Check class="hidden h-3.5 w-3.5 shrink-0 min-[400px]:block" />
          <span class="truncate">{{ labelTooEasy || labelEasy || t('flashcardStudy.easy') }}</span>
        </button>
        <button
          type="button"
          class="lingo-btn lingo-btn--sm w-full !normal-case !tracking-normal"
          :class="buttonVariantClass"
          :disabled="disabled"
          :aria-disabled="disabled"
          @click="$emit('check')"
        >
          {{ labelContinue || t('flashcardStudy.continue') }}
        </button>
      </div>

      <!-- Classical rate triad / idle Check -->
      <div v-else class="lingo-study-rate-row" role="group">
        <div class="lingo-study-footer-feedback" :class="feedbackToneClass">
          <template v-if="status === 'rate'">
            <button
              type="button"
              class="lingo-btn lingo-btn--danger lingo-btn--sm w-full !normal-case !tracking-normal"
              :disabled="disabled"
              @click="$emit('rate', 1)"
            >
              <XCircle class="h-3.5 w-3.5 shrink-0" />
              <span>{{ labelForgot || t('flashcardStudy.forgot') }}</span>
              <span class="hidden opacity-70 sm:inline">(1)</span>
            </button>
          </template>
          <template v-else-if="showCardActions && status === 'none' && leftAction !== 'none'">
            <button
              type="button"
              class="lingo-btn lingo-btn--default lingo-btn--sm !normal-case !tracking-normal"
              :disabled="snoozeDisabled"
              :aria-label="leftActionLabel"
              :title="leftActionLabel"
              @click="onLeftAction"
            >
              <VolumeX v-if="leftAction === 'cant-listen'" class="h-3.5 w-3.5 shrink-0" />
              <AlarmClock v-else class="h-3.5 w-3.5 shrink-0" />
              <span>{{ leftActionLabel }}</span>
            </button>
          </template>
        </div>

        <div class="flex h-9 min-w-0 items-center">
          <button
            v-if="status === 'rate'"
            type="button"
            class="lingo-btn lingo-btn--primary lingo-btn--sm w-full !normal-case !tracking-normal"
            :disabled="disabled"
            @click="$emit('rate', 3)"
          >
            <Smile class="h-3.5 w-3.5 shrink-0" />
            <span>{{ labelGood || t('flashcardStudy.good') }}</span>
            <span class="hidden opacity-70 sm:inline">(2)</span>
          </button>
        </div>

        <div class="flex h-9 min-w-0 items-center">
          <button
            v-if="status === 'rate'"
            type="button"
            class="lingo-btn lingo-btn--secondary lingo-btn--sm w-full !normal-case !tracking-normal"
            :disabled="disabled"
            @click="$emit('rate', 4)"
          >
            <Check class="h-3.5 w-3.5 shrink-0" />
            <span>{{ labelEasy || t('flashcardStudy.easy') }}</span>
            <span class="hidden opacity-70 sm:inline">(3)</span>
          </button>
          <button
            v-else
            type="button"
            class="lingo-btn lingo-btn--sm w-full !normal-case !tracking-normal"
            :class="buttonVariantClass"
            :disabled="disabled"
            :aria-disabled="disabled"
            @click="$emit('check')"
          >
            {{ buttonLabel }}
          </button>
        </div>
      </div>

      <div
        class="lingo-study-footer-meta"
        :class="{ 'lingo-study-footer-meta--graded': status === 'correct' || status === 'wrong' }"
      >
        <button
          v-if="showCardActions"
          type="button"
          class="lingo-btn lingo-btn--ghost lingo-btn--sm lingo-btn--icon !min-w-9 !px-0 !normal-case !tracking-normal"
          :class="status === 'wrong' ? 'lingo-study-footer-meta__discuss--wrong' : undefined"
          :aria-label="labelDiscuss || t('flashcardStudy.wavesButton')"
          :title="labelDiscuss || t('flashcardStudy.wavesButton')"
          @click="$emit('discuss')"
        >
          <MessagesSquare class="h-3.5 w-3.5 shrink-0" />
        </button>
        <span v-else class="inline-block h-9 w-9 shrink-0" aria-hidden="true" />

        <div
          v-if="status === 'correct' || status === 'wrong'"
          class="lingo-study-footer-result"
          :class="feedbackToneClass"
          role="status"
          :title="status === 'wrong' && solutionText ? solutionText : undefined"
        >
          <CheckCircle
            v-if="status === 'correct'"
            class="lingo-study-footer-feedback__icon"
            aria-hidden="true"
          />
          <XCircle v-else class="lingo-study-footer-feedback__icon" aria-hidden="true" />
          <p class="lingo-study-footer-result__text">
            <span class="lingo-study-footer-feedback__label">{{
              status === 'correct' ? correctLabel : wrongLabel
            }}</span>
            <template v-if="status === 'wrong' && solutionText">
              <span class="lingo-study-footer-solution__label">
                {{ labelCorrectSolution || t('flashcardStudy.correctAnswer') }}</span
              >
              <span class="lingo-study-footer-solution__text"> {{ solutionText }}</span>
            </template>
          </p>
        </div>
        <p
          v-else
          class="lingo-study-footer-meta__prompt"
          :class="{ invisible: status !== 'rate' || !ratePrompt }"
          :aria-hidden="status !== 'rate' || !ratePrompt"
        >
          {{ ratePrompt || '\u00a0' }}
        </p>

        <span
          v-if="status !== 'correct' && status !== 'wrong'"
          class="inline-flex h-9 w-9 shrink-0"
          aria-hidden="true"
        />
      </div>
    </div>
  </footer>
</template>

<script setup lang="ts">
import {
  AlarmClock,
  Check,
  CheckCircle,
  MessagesSquare,
  Smile,
  VolumeX,
  XCircle,
} from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps({
  status: {
    type: String,
    default: 'none',
    validator: (v: unknown) =>
      typeof v === 'string' && ['none', 'correct', 'wrong', 'completed', 'rate'].includes(v),
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  showCardActions: {
    type: Boolean,
    default: false,
  },
  snoozeDisabled: {
    type: Boolean,
    default: false,
  },
  leftAction: {
    type: String,
    default: 'snooze',
    validator: (v: unknown) =>
      typeof v === 'string' && ['snooze', 'cant-listen', 'none'].includes(v),
  },
  correctLabel: { type: String, default: '' },
  wrongLabel: { type: String, default: '' },
  /** Wrong-answer “Correct solution: …” body text. */
  solutionText: { type: String, default: '' },
  labelCorrectSolution: { type: String, default: '' },
  labelCheck: { type: String, default: '' },
  labelNext: { type: String, default: '' },
  labelRetry: { type: String, default: '' },
  labelContinue: { type: String, default: '' },
  labelForgot: { type: String, default: '' },
  labelGood: { type: String, default: '' },
  labelEasy: { type: String, default: '' },
  labelTooEasy: { type: String, default: '' },
  labelTooDifficult: { type: String, default: '' },
  labelDiscuss: { type: String, default: '' },
  labelSnooze: { type: String, default: '' },
  labelCantListen: { type: String, default: '' },
  ratePrompt: { type: String, default: '' },
  compact: { type: Boolean, default: false },
})

const emit = defineEmits(['check', 'rate', 'discuss', 'snooze', 'skipListening'])

const { t } = useI18n()

const leftActionLabel = computed(() => {
  if (props.leftAction === 'cant-listen') {
    return props.labelCantListen || t('flashcardStudy.cantListenNow')
  }
  return props.labelSnooze || t('flashcardStudy.snooze')
})

function onLeftAction() {
  if (props.leftAction === 'cant-listen') emit('skipListening')
  else emit('snooze')
}

const footerToneClass = computed(() => {
  if (props.status === 'correct') return 'lingo-study-footer--correct'
  if (props.status === 'wrong') return 'lingo-study-footer--wrong'
  return 'lingo-study-footer--idle'
})

const feedbackToneClass = computed(() => {
  if (props.status === 'correct') return 'lingo-study-footer-feedback--correct'
  if (props.status === 'wrong') return 'lingo-study-footer-feedback--wrong'
  return undefined
})

const buttonVariantClass = computed(() => {
  if (props.status === 'wrong') return 'lingo-btn--danger'
  if (props.status === 'correct' || props.status === 'completed') return 'lingo-btn--secondary'
  return 'lingo-btn--secondary'
})

const buttonLabel = computed(() => {
  if (props.status === 'none') return props.labelCheck || t('flashcardStudy.check')
  if (props.status === 'correct') return props.labelContinue || t('flashcardStudy.continue')
  if (props.status === 'wrong') return props.labelContinue || t('flashcardStudy.continue')
  return props.labelContinue || t('flashcardStudy.continue')
})
</script>

<style scoped>
.lingo-study-footer {
  -webkit-tap-highlight-color: transparent;
}
.lingo-study-footer button:disabled {
  cursor: not-allowed;
}
</style>
