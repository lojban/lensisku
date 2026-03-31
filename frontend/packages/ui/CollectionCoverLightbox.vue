<template>
  <button
    type="button"
    :class="[
      imageUrl ? 'cursor-zoom-in max-w-full' : 'max-w-full',
      'inline-flex shrink-0 border-0 bg-transparent p-0 text-inherit',
    ]"
    :disabled="!imageUrl"
    :aria-label="ariaLabel || alt || 'Collection cover'"
    @click="onThumbClick"
  >
    <slot />
  </button>
  <Teleport to="body">
    <Transition name="collection-cover-lightbox-fade">
      <div
        v-if="open && imageUrl"
        class="fixed inset-0 z-[10050] flex items-center justify-center bg-black/85 p-4 sm:p-8"
        role="dialog"
        aria-modal="true"
        :aria-label="ariaLabel || alt || 'Collection cover'"
        @click.self="close"
      >
        <button
          type="button"
          class="absolute top-3 right-3 sm:top-5 sm:right-5 rounded-full bg-white/15 p-2 text-white transition-colors hover:bg-white/25 focus:outline-none focus-visible:ring-2 focus-visible:ring-white/80"
          :aria-label="closeAriaLabel"
          @click.stop="close"
        >
          <X class="h-6 w-6" aria-hidden="true" />
        </button>
        <img
          :src="imageUrl"
          :alt="alt"
          class="max-h-[min(92vh,100%)] max-w-[min(92vw,100%)] h-auto w-auto rounded-lg object-contain shadow-2xl ring-1 ring-white/10"
          @click.stop="close"
        />
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { X } from 'lucide-vue-next'
import { ref, watch, onMounted, onUnmounted } from 'vue'

const props = withDefaults(
  defineProps<{
    /** When set, thumbnail is clickable and opens full-screen preview. */
    imageUrl: string | null | undefined
    alt?: string
    /** Accessible name for the overlay dialog. */
    ariaLabel?: string
    /** Screen-reader label for the close control. */
    closeAriaLabel?: string
  }>(),
  {
    alt: '',
    ariaLabel: '',
    closeAriaLabel: 'Close',
  }
)

const open = ref(false)

function close() {
  open.value = false
}

function onThumbClick() {
  if (!props.imageUrl) return
  open.value = true
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && open.value) {
    e.preventDefault()
    close()
  }
}

watch(open, (v) => {
  if (typeof document === 'undefined') return
  document.body.style.overflow = v ? 'hidden' : ''
})

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
  if (typeof document !== 'undefined') {
    document.body.style.overflow = ''
  }
})
</script>

<style scoped>
.collection-cover-lightbox-fade-enter-active,
.collection-cover-lightbox-fade-leave-active {
  transition: opacity 0.2s ease;
}
.collection-cover-lightbox-fade-enter-from,
.collection-cover-lightbox-fade-leave-to {
  opacity: 0;
}
</style>
