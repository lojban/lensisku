<template>
  <div ref="rootRef" :class="rootClass">
    <div aria-haspopup="true" :aria-expanded="open" @click="open = !open">
      <slot name="trigger">
        <button
          type="button"
          class="w-full sm:w-auto h-9 px-3 hover:bg-gray-100 rounded-full inline-flex items-center justify-between sm:justify-center gap-2 shrink-0"
        >
          <span v-if="triggerLabel" class="text-sm text-gray-600">{{ triggerLabel }}</span>
          <EllipsisVertical class="w-4 h-4" />
        </button>
      </slot>
    </div>
    <div
      v-if="open"
      class="absolute right-0 mt-2 w-full sm:w-48 bg-white border rounded-lg shadow-lg py-1 z-30"
      @click="open = false"
    >
      <slot />
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { EllipsisVertical } from 'lucide-vue-next'

defineOptions({ inheritAttrs: false })

const props = defineProps({
  /** Optional label shown next to the three-dot icon when using default trigger */
  triggerLabel: { type: String, default: '' },
})

const open = ref(false)
const rootRef = ref(null)

const rootClass = 'relative inline-block'

function handleClickOutside(event) {
  if (rootRef.value && !rootRef.value.contains(event.target)) {
    open.value = false
  }
}

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside)
})
</script>
