<script setup lang="ts">
import { ref } from 'vue'
import PublicDomainNotice from '@/components/PublicDomainNotice.vue'

withDefaults(
  defineProps<{
    /** Page heading. Omit it when the toolbar should show actions only. */
    title?: string
    /** Cap form width (markdown / wiki editors). */
    narrow?: boolean
    /** Let a long-form editor fill the remaining scroll viewport, above the public-domain notice. */
    fillContent?: boolean
  }>(),
  { title: '', narrow: false, fillContent: false }
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
          v-if="title"
          class="my-0 min-w-0 flex-1 truncate text-xl font-bold text-gray-800 select-none sm:text-2xl"
        >
          {{ title }}
        </h2>
        <div
          v-if="$slots.trailing"
          class="ml-auto flex shrink-0 flex-nowrap items-center justify-end gap-2"
        >
          <slot name="trailing" />
        </div>
      </div>
    </header>

    <div
      v-if="fillContent"
      class="flex min-h-0 w-full flex-1 flex-col overflow-hidden overscroll-y-contain pb-0 [&_.public-domain-notice]:mt-0"
    >
      <div
        :ref="assignScrollEl"
        class="flex min-h-0 w-full flex-1 flex-col overflow-x-hidden overflow-y-auto overscroll-y-contain"
        @scroll="handleScroll($event)"
      >
        <slot />
      </div>
      <PublicDomainNotice />
    </div>
    <div v-else :ref="assignScrollEl" class="upsert-scroll" @scroll="handleScroll($event)">
      <div class="upsert-scroll__stack">
        <div
          class="upsert-scroll__main px-3 pt-4 pb-4 sm:px-4"
          :class="narrow ? 'mx-auto max-w-3xl' : ''"
        >
          <slot />
        </div>
        <div class="upsert-scroll__spacer" aria-hidden="true"></div>
        <PublicDomainNotice />
      </div>
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
