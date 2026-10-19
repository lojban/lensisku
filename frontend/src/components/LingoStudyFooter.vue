<template>
  <footer
    class="lingo-study-footer sticky bottom-0 z-30 border-t-2 transition-colors shrink-0"
    :class="[footerClass, compact ? 'lingo-study-footer--compact' : '']"
  >
    <div
      class="mx-auto flex h-full max-w-[1140px] items-center justify-between gap-3 px-3 py-2 sm:px-4 sm:py-2.5 lg:px-6"
      :class="compact ? 'min-h-[56px] sm:min-h-[60px]' : 'min-h-[72px] sm:py-3 lg:min-h-[88px]'"
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
import { CheckCircle, XCircle } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps({
  status: {
    type: String,
    default: 'none',
    validator: (v: unknown) =>
      typeof v === 'string' && ['none', 'correct', 'wrong', 'completed'].includes(v),
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
  compact: { type: Boolean, default: false },
})

defineEmits(['check'])

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
