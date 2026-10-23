<template>
  <footer
    class="lingo-study-footer sticky bottom-0 z-30 w-full border-t-2 transition-colors shrink-0"
    :class="[footerClass, compact ? 'lingo-study-footer--compact' : 'lingo-study-footer--roomy']"
  >
    <!-- One chrome box for Check and rate — fixed height avoids CLS (clone footer.tsx). -->
    <!-- Same chrome for Check and rate so the Easy/Check cell never moves. -->
    <div
      class="lingo-study-chrome-inner lingo-study-footer__chrome !items-stretch flex-col justify-center gap-1.5 sm:gap-2"
    >
      <div class="flex w-full min-w-0 items-center gap-2">
        <div
          v-if="showCardActions"
          class="flex shrink-0 items-center justify-start gap-1.5 sm:gap-2"
        >
          <button
            type="button"
            class="lingo-btn lingo-btn--ghost lingo-btn--sm !normal-case !tracking-normal"
            :class="{ 'lingo-btn--icon !min-w-9 !px-0': discussIconOnly }"
            :aria-label="labelDiscuss || t('flashcardStudy.wavesButton')"
            :title="labelDiscuss || t('flashcardStudy.wavesButton')"
            @click="$emit('discuss')"
          >
            <MessagesSquare class="h-3.5 w-3.5 shrink-0" />
            <span v-if="!discussIconOnly">{{
              labelDiscuss || t('flashcardStudy.wavesButton')
            }}</span>
          </button>
        </div>
        <p
          class="min-w-0 flex-1 truncate text-center text-[10px] font-bold uppercase tracking-wide text-slate-500 sm:text-xs"
          :class="status === 'rate' && ratePrompt ? '' : 'invisible'"
          :aria-hidden="status !== 'rate' || !ratePrompt"
        >
          {{ ratePrompt || '\u00a0' }}
        </p>
        <div
          v-if="showCardActions && discussIconOnly"
          class="invisible flex shrink-0 items-center gap-1.5 sm:gap-2"
          aria-hidden="true"
        >
          <span class="lingo-btn lingo-btn--ghost lingo-btn--sm lingo-btn--icon !min-w-9 !px-0">
            <MessagesSquare class="h-3.5 w-3.5 shrink-0" />
          </span>
        </div>
      </div>
      <div class="lingo-study-rate-row">
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
          <button
            type="button"
            class="lingo-btn lingo-btn--primary lingo-btn--sm w-full !normal-case !tracking-normal"
            :disabled="disabled"
            @click="$emit('rate', 3)"
          >
            <Smile class="h-3.5 w-3.5 shrink-0" />
            <span>{{ labelGood || t('flashcardStudy.good') }}</span>
            <span class="hidden opacity-70 sm:inline">(2)</span>
          </button>
          <button
            type="button"
            class="lingo-btn lingo-btn--secondary lingo-btn--sm w-full !normal-case !tracking-normal"
            :disabled="disabled"
            @click="$emit('rate', 4)"
          >
            <Check class="h-3.5 w-3.5 shrink-0" />
            <span>{{ labelEasy || t('flashcardStudy.easy') }}</span>
            <span class="hidden opacity-70 sm:inline">(3)</span>
          </button>
        </template>
        <template v-else>
          <div class="flex min-w-0 items-center justify-start gap-1.5 sm:gap-2">
            <button
              v-if="showCardActions && status === 'none'"
              type="button"
              class="lingo-btn lingo-btn--default lingo-btn--sm !normal-case !tracking-normal"
              :disabled="snoozeDisabled"
              :aria-label="labelSnooze || t('flashcardStudy.snooze')"
              :title="labelSnooze || t('flashcardStudy.snooze')"
              @click="$emit('snooze')"
            >
              <AlarmClock class="h-3.5 w-3.5 shrink-0" />
              <span>{{ labelSnooze || t('flashcardStudy.snooze') }}</span>
            </button>
            <template v-else-if="status === 'correct'">
              <CheckCircle
                class="h-4 w-4 shrink-0 text-green-600 sm:h-5 sm:w-5"
                :class="{ 'lg:h-6 lg:w-6': !compact }"
              />
              <span
                class="truncate text-xs font-bold text-green-600 sm:text-sm"
                :class="{ 'lg:text-base': !compact, 'lg:text-sm': compact }"
              >
                {{ correctLabel }}
              </span>
            </template>
            <template v-else-if="status === 'wrong'">
              <XCircle
                class="h-4 w-4 shrink-0 text-rose-500 sm:h-5 sm:w-5"
                :class="{ 'lg:h-6 lg:w-6': !compact }"
              />
              <span
                class="truncate text-xs font-bold text-rose-500 sm:text-sm"
                :class="{ 'lg:text-base': !compact, 'lg:text-sm': compact }"
              >
                {{ wrongLabel }}
              </span>
            </template>
          </div>
          <div aria-hidden="true" />
          <button
            type="button"
            class="lingo-btn lingo-btn--sm w-full !normal-case !tracking-normal"
            :class="buttonVariantClass"
            :disabled="disabled"
            :aria-disabled="disabled"
            @click="$emit('check')"
          >
            {{ buttonLabel }}
          </button>
        </template>
      </div>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { AlarmClock, Check, CheckCircle, MessagesSquare, Smile, XCircle } from '@lucide/vue'
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
  /** Discussions + snooze (classical study parity). */
  showCardActions: {
    type: Boolean,
    default: false,
  },
  snoozeDisabled: {
    type: Boolean,
    default: false,
  },
  correctLabel: {
    type: String,
    default: '',
  },
  wrongLabel: {
    type: String,
    default: '',
  },
  labelCheck: { type: String, default: '' },
  labelNext: { type: String, default: '' },
  labelRetry: { type: String, default: '' },
  labelContinue: { type: String, default: '' },
  labelForgot: { type: String, default: '' },
  labelGood: { type: String, default: '' },
  labelEasy: { type: String, default: '' },
  labelDiscuss: { type: String, default: '' },
  labelSnooze: { type: String, default: '' },
  ratePrompt: { type: String, default: '' },
  compact: { type: Boolean, default: false },
})

defineEmits(['check', 'rate', 'discuss', 'snooze'])

const { t } = useI18n()

/** Discussions: icon-only while the rate prompt shares the top row. */
const discussIconOnly = computed(() => props.status === 'rate')

const footerClass = computed(() => {
  if (props.status === 'correct') return 'border-transparent bg-green-100'
  if (props.status === 'wrong') return 'border-transparent bg-rose-100'
  return 'border-slate-200 bg-white'
})

const buttonVariantClass = computed(() => {
  if (props.status === 'wrong') return 'lingo-btn--danger'
  return 'lingo-btn--secondary'
})

const buttonLabel = computed(() => {
  if (props.status === 'none') return props.labelCheck || t('flashcardStudy.check')
  if (props.status === 'correct') return props.labelNext || t('flashcardStudy.next')
  if (props.status === 'wrong') return props.labelRetry || t('flashcardStudy.retry')
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
/* Equal Check/Easy cells even if Tailwind layer lags on HMR. */
.lingo-study-footer :deep(.lingo-study-rate-row) {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
</style>
