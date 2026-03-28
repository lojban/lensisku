<template>

  <footer
    class="lingo-study-footer sticky bottom-0 z-30 border-t-2 transition-colors shrink-0"
    :class="[footerClass, compact ? 'lingo-study-footer--compact' : '']"
  >

    <div
      class="mx-auto flex h-full max-w-[1140px] items-center justify-between gap-3 px-3 py-2 sm:px-4 sm:py-2.5 lg:px-6"
      :class="compact ? 'min-h-[56px] sm:min-h-[60px]' : 'min-h-[72px] sm:py-3 lg:min-h-[88px]'"
    >
       <!-- Status message (correct / wrong) -->
      <div
        v-if="status === 'correct'"
        class="flex items-center text-xs font-bold text-green-600 sm:text-sm"
        :class="{ 'lg:text-base': !compact, 'lg:text-sm': compact }"
      >
         <CheckCircle
          class="mr-1.5 h-4 w-4 shrink-0 sm:mr-2 sm:h-5 sm:w-5"
          :class="{ 'lg:h-6 lg:w-6': !compact }"
        /> <span>{{ correctLabel }}</span
        >
      </div>

      <div
        v-else-if="status === 'wrong'"
        class="flex items-center text-xs font-bold text-rose-500 sm:text-sm"
        :class="{ 'lg:text-base': !compact, 'lg:text-sm': compact }"
      >
         <XCircle
          class="mr-1.5 h-4 w-4 shrink-0 sm:mr-2 sm:h-5 sm:w-5"
          :class="{ 'lg:h-6 lg:w-6': !compact }"
        /> <span>{{ wrongLabel }}</span
        >
      </div>

      <div v-else class="flex-1" />
       <!-- Main action button --> <button
        type="button"
        :disabled="disabled"
        :aria-disabled="disabled"
        class="ui-btn--aqua-default ml-auto text-sm"
        :class="[
          buttonClass,
          compact
            ? 'min-w-[88px] py-2 sm:min-w-[96px] sm:py-2'
            : 'min-w-[100px] py-2.5 sm:min-w-[110px] sm:py-3 sm:text-base lg:min-w-[120px]',
        ]"
        @click="$emit('check')"
      >
         {{ buttonLabel }} </button
      >
    </div>

  </footer>

</template>

<script setup lang="ts">
import { CheckCircle, XCircle } from 'lucide-vue-next'
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

const buttonClass = computed(() => {
  if (props.status === 'wrong') return 'ui-btn--danger-rose'
  return 'ui-btn--auth-signup'
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

