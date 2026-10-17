<script setup lang="ts">
import { ref } from 'vue'
import PublicDomainNotice from '@/components/PublicDomainNotice.vue'

withDefaults(
  defineProps<{
    title: string
    /** Cap form width (markdown / wiki editors). */
    narrow?: boolean
    /** Let a long-form editor fill the remaining scroll viewport, above the public-domain notice. */
    fillContent?: boolean
  }>(),
  { narrow: false, fillContent: false }
)

const scrollEl = ref<HTMLElement | null>(null)
const emit = defineEmits<{
  scroll: [scrollTop: number, viewportHeight: number]
}>()

function assignScrollEl(el: Element | null) {
  scrollEl.value = el instanceof HTMLElement ? el : null
}

function scrollToTop(behavior: 'auto' | 'instant' | 'smooth' = 'smooth') {
  scrollEl.value?.scrollTo({ top: 0, behavior })
}

function handleScroll(event: Event) {
  const element = event.currentTarget as HTMLElement
  emit('scroll', element.scrollTop, element.clientHeight)
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
      :ref="fillContent ? undefined : assignScrollEl"
      class="flex min-h-0 flex-1 flex-col overscroll-y-contain pb-0"
      :class="
        fillContent
          ? 'overflow-hidden px-0 sm:px-4 [&_.public-domain-notice]:mt-0'
          : 'overflow-x-hidden overflow-y-auto px-3 sm:px-4'
      "
      @scroll="fillContent ? undefined : handleScroll($event)"
    >
      <div
        v-if="narrow"
        :ref="fillContent ? assignScrollEl : undefined"
        class="mx-auto w-full max-w-3xl"
        :class="
          fillContent
            ? 'flex min-h-0 flex-1 flex-col overflow-x-hidden overflow-y-auto overscroll-y-contain'
            : ''
        "
        @scroll="fillContent ? handleScroll($event) : undefined"
      >
        <slot />
      </div>
      <slot v-else />
      <PublicDomainNotice />
    </div>
    <footer
      v-if="$slots.actions"
      class="shrink-0 border-t border-gray-200 bg-white px-3 py-2 sm:px-4"
    >
      <div class="mx-auto flex w-full max-w-3xl items-center justify-end gap-2">
        <slot name="actions" />
      </div>
    </footer>
  </div>
</template>
