<template>
  <div class="lingo-result-card w-full rounded-2xl border-2 overflow-hidden" :class="cardClass">
    <div
      class="rounded-t-xl px-3 py-1.5 text-center text-xs font-bold uppercase text-white"
      :class="headerClass"
    >
      {{ headerText }}
    </div>

    <div
      class="flex items-center justify-center rounded-b-xl bg-white px-4 py-5 text-lg font-bold sm:py-6"
      :class="valueClass"
    >
      <Trophy v-if="variant === 'points'" class="mr-1.5 h-7 w-7 shrink-0 sm:h-8 sm:w-8" />
      <Heart v-else class="mr-1.5 h-7 w-7 shrink-0 sm:h-8 sm:w-8" />
      <span v-if="value !== Infinity">{{ value }}</span>
      <InfinityIcon v-else class="h-6 w-6 stroke-[2.5]" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Heart, Trophy, Infinity as InfinityIcon } from 'lucide-vue-next'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps({
  variant: {
    type: String,
    default: 'points',
    validator: (v) => v === 'points' || v === 'hearts',
  },
  value: {
    type: Number,
    default: 0,
  },
  headerText: {
    type: String,
    default: '',
  },
})

const { t } = useI18n()

const cardClass = computed(() =>
  props.variant === 'points' ? 'border-orange-400' : 'border-rose-500'
)

const headerClass = computed(() => (props.variant === 'points' ? 'bg-orange-400' : 'bg-rose-500'))

const valueClass = computed(() =>
  props.variant === 'points' ? 'text-orange-500' : 'text-rose-500'
)

const headerText = computed(() => {
  if (props.headerText) return props.headerText
  return props.variant === 'hearts' ? t('lingo.heartsLeft') : t('lingo.totalXP')
})
</script>

<style scoped>
.lingo-result-card {
  -webkit-tap-highlight-color: transparent;
}
</style>
