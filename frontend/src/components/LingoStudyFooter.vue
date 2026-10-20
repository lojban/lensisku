<template>
  <footer
    class="lingo-study-footer sticky bottom-0 z-30 w-full border-t-2 transition-colors shrink-0"
    :class="[footerClass, compact ? 'lingo-study-footer--compact' : '']"
  >
    <!-- SRS rating: Forgot / Good / Easy replace the single Check CTA -->
    <div
      v-if="status === 'rate'"
      class="lingo-study-chrome-inner !items-stretch flex-col justify-center gap-1.5 sm:gap-2"
      :class="compact ? 'min-h-[56px] sm:min-h-[64px]' : 'min-h-[72px] lg:min-h-[88px]'"
    >
      <p
        v-if="ratePrompt"
        class="text-center text-[10px] font-bold uppercase tracking-wide text-slate-500 sm:text-xs"
      >
        {{ ratePrompt }}
      </p>
      <div class="lingo-study-rate-row">
        <button
          type="button"
          class="lingo-btn lingo-btn--danger lingo-btn--sm flex-1 !normal-case !tracking-normal"
          :disabled="disabled"
          @click="$emit('rate', 1)"
        >
          <XCircle class="h-3.5 w-3.5 shrink-0" />
          <span>{{ labelForgot || t('flashcardStudy.forgot') }}</span>
          <span class="hidden opacity-70 sm:inline">(1)</span>
        </button>
        <button
          type="button"
          class="lingo-btn lingo-btn--primary lingo-btn--sm flex-1 !normal-case !tracking-normal"
          :disabled="disabled"
          @click="$emit('rate', 3)"
        >
          <Smile class="h-3.5 w-3.5 shrink-0" />
          <span>{{ labelGood || t('flashcardStudy.good') }}</span>
          <span class="hidden opacity-70 sm:inline">(2)</span>
        </button>
        <button
          type="button"
          class="lingo-btn lingo-btn--secondary lingo-btn--sm flex-1 !normal-case !tracking-normal"
          :disabled="disabled"
          @click="$emit('rate', 4)"
        >
          <Check class="h-3.5 w-3.5 shrink-0" />
          <span>{{ labelEasy || t('flashcardStudy.easy') }}</span>
          <span class="hidden opacity-70 sm:inline">(3)</span>
        </button>
      </div>
    </div>

    <div
      v-else
      class="lingo-study-chrome-inner justify-between gap-3"
      :class="compact ? 'min-h-[56px] sm:min-h-[60px]' : 'min-h-[72px] lg:min-h-[88px]'"
    >
      <div
        v-if="status === 'correct'"
        class="flex items-center gap-1.5 text-xs font-bold text-green-600 sm:gap-2 sm:text-sm"
        :class="{ 'lg:text-base': !compact, 'lg:text-sm': compact }"
      >
        <CheckCircle
          class="h-4 w-4 shrink-0 sm:h-5 sm:w-5"
          :class="{ 'lg:h-6 lg:w-6': !compact }"
        />
        <span>{{ correctLabel }}</span>
      </div>

      <div
        v-else-if="status === 'wrong'"
        class="flex items-center gap-1.5 text-xs font-bold text-rose-500 sm:gap-2 sm:text-sm"
        :class="{ 'lg:text-base': !compact, 'lg:text-sm': compact }"
      >
        <XCircle class="h-4 w-4 shrink-0 sm:h-5 sm:w-5" :class="{ 'lg:h-6 lg:w-6': !compact }" />
        <span>{{ wrongLabel }}</span>
      </div>

      <div v-else class="flex-1" />

      <button
        type="button"
        class="lingo-btn ml-auto text-sm"
        :class="[
          buttonVariantClass,
          compact
            ? 'lingo-btn--sm min-w-[88px] sm:min-w-[96px]'
            : 'lingo-btn--lg min-w-[100px] sm:min-w-[110px] sm:text-base lg:min-w-[120px]',
        ]"
        :disabled="disabled"
        :aria-disabled="disabled"
        @click="$emit('check')"
      >
        {{ buttonLabel }}
      </button>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { Check, CheckCircle, Smile, XCircle } from '@lucide/vue'
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
  ratePrompt: { type: String, default: '' },
  compact: { type: Boolean, default: false },
})

defineEmits(['check', 'rate'])

const { t } = useI18n()

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
