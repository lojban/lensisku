<script setup lang="ts">
import { ref } from 'vue'

withDefaults(
  defineProps<{
    title: string
    /** Cap form width (markdown / wiki editors). */
    narrow?: boolean
  }>(),
  { narrow: false }
)

const scrollEl = ref<HTMLElement | null>(null)

function scrollToTop(behavior: ScrollBehavior = 'smooth') {
  scrollEl.value?.scrollTo({ top: 0, behavior: 'smooth' })
}

defineExpose({ scrollToTop })
</script>

<template>
  <div class="flex min-h-0 w-full flex-1 flex-col overflow-hidden">
    <header
      class="shrink-0 rounded-none border-b border-gray-200 bg-white px-3 py-2.5 sm:px-4 sm:py-3"
    >
      <div class="flex w-full min-w-0 flex-nowrap items-center justify-between gap-3">
        <h2
          class="my-0 min-w-0 flex-1 truncate text-xl font-bold text-gray-800 select-none sm:text-2xl"
        >
          {{ title }}
        </h2>
        <div
          v-if="$slots.trailing"
          class="flex shrink-0 flex-nowrap items-center justify-end gap-2"
        >
          <slot name="trailing" />
        </div>
      </div>
    </header>

    <div
      ref="scrollEl"
      class="min-h-0 flex-1 overflow-x-hidden overflow-y-auto overscroll-y-contain px-3 pb-4 pt-4 sm:px-4 sm:pb-6"
    >
      <div v-if="narrow" class="mx-auto w-full max-w-3xl">
        <slot />
      </div>
      <slot v-else />
    </div>
  </div>
</template>
