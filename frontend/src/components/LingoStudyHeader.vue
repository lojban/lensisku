<template>

  <header class="lingo-study-header-bar">
     <button
      type="button"
      class="icon-btn-header-back icon-btn-header-back--compact"
      :aria-label="t('flashcardStudy.endSession')"
      @click="$emit('exit')"
    >
       <X class="h-5 w-5 sm:h-5 sm:w-5" /> </button
    > <!-- Progress bar: fills remaining width so green reaches end of row -->
    <div class="min-w-0 flex-1 pl-2 pr-0 sm:pl-3 sm:pr-0">

      <div
        class="relative h-3 w-full overflow-hidden rounded-full bg-slate-200 sm:h-3.5"
        role="progressbar"
        :aria-valuenow="percentage"
        aria-valuemin="0"
        aria-valuemax="100"
      >

        <div
          class="h-full rounded-full bg-green-500 transition-all duration-300"
          :style="{ width: `${Math.min(100, Math.max(0, percentage))}%` }"
        />

      </div>

    </div>
     <!-- Optional slot for hearts/points (e.g. hearts count) -->
    <div v-if="$slots.extra" class="flex shrink-0 items-center"> <slot name="extra" /> </div>

  </header>

</template>

<script setup lang="ts">
import { X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

defineProps({
  percentage: {
    type: Number,
    default: 0,
  },
})

defineEmits(['exit'])

const { t } = useI18n()
</script>

