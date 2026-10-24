<template>
  <footer
    class="lingo-study-footer sticky bottom-0 z-30 w-full shrink-0 border-t-2 transition-colors"
    :class="[footerClass, compact ? 'lingo-study-footer--compact' : 'lingo-study-footer--roomy']"
  >
    <!--
      Same max-w-[600px] as header `.lingo-study-chrome-inner` / lesson column.
      Bottom-pinned rows + 3 equal cells keep Check and Easy aligned (no CLS).
    -->
    <div class="lingo-study-footer-chrome">
      <!-- Row 1: Check / rate buttons. Row 2: discuss + prompt (justify-end packs this pair to the bottom). -->
      <div class="lingo-study-rate-row" role="group">
        <div class="flex h-9 min-w-0 items-center justify-start">
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

      <div class="lingo-study-footer-meta">
        <button
          v-if="showCardActions"
          type="button"
          class="lingo-btn lingo-btn--ghost lingo-btn--sm lingo-btn--icon !min-w-9 !px-0 !normal-case !tracking-normal"
          :aria-label="labelDiscuss || t('flashcardStudy.wavesButton')"
          :title="labelDiscuss || t('flashcardStudy.wavesButton')"
          @click="$emit('discuss')"
        >
          <MessagesSquare class="h-3.5 w-3.5 shrink-0" />
        </button>
        <span v-else class="inline-block h-9 w-9 shrink-0" aria-hidden="true" />

        <p
          class="lingo-study-footer-meta__prompt"
          :class="{ invisible: status !== 'rate' || !ratePrompt }"
          :aria-hidden="status !== 'rate' || !ratePrompt"
        >
          {{ ratePrompt || '\u00a0' }}
        </p>

        <!-- Width mirror so the prompt stays centered in every status. -->
        <span class="inline-flex h-9 w-9 shrink-0" aria-hidden="true" />
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
  /** Discussions + snooze (classical study parity). */
  showCardActions: {
    type: Boolean,
    default: false,
  },
  snoozeDisabled: {
    type: Boolean,
    default: false,
  },
  /**
   * Left chrome action while status is `none`:
   * `snooze` | `cant-listen` (Duo listenTap) | `none`.
   */
  leftAction: {
    type: String,
    default: 'snooze',
    validator: (v: unknown) =>
      typeof v === 'string' && ['snooze', 'cant-listen', 'none'].includes(v),
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
</style>
