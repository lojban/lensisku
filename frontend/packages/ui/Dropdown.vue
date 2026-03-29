<template>

  <div ref="rootRef" :class="rootClass">

    <div aria-haspopup="true" :aria-expanded="open" @click="open = !open">
       <slot name="trigger" :open="open"
        > <button
          type="button"
          class="w-full sm:w-auto h-9 px-3 hover:bg-gray-100 rounded-full inline-flex items-center justify-between sm:justify-center gap-2 shrink-0"
        >
           <span v-if="triggerLabel" class="text-sm text-gray-600">{{ triggerLabel }}</span
          > <EllipsisVertical class="w-4 h-4" /> </button
        > </slot
      >
    </div>

    <Teleport to="body">
      <div
        v-if="open"
        ref="panelRef"
        class="fixed z-50 w-fit min-w-0 max-w-[calc(100vw-1rem)] overflow-y-auto bg-white border rounded-lg shadow-lg py-1"
        :style="panelStyle"
        @click="open = false"
      >
        <div class="w-fit whitespace-nowrap">
          <slot />
        </div>
      </div>
    </Teleport>

  </div>

</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { EllipsisVertical } from 'lucide-vue-next'

defineOptions({ inheritAttrs: false })

defineProps({
  /** Optional label shown next to the three-dot icon when using default trigger */
  triggerLabel: { type: String, default: '' },
})

const VIEWPORT_MARGIN = 8

/** Keep the teleported panel at or below the app sticky header so it is not clipped (same z-layer as header). */
function getAppHeaderBottom(): number {
  const header = document.querySelector('#app > header')
  if (header instanceof HTMLElement) {
    return header.getBoundingClientRect().bottom
  }
  return VIEWPORT_MARGIN
}

const open = ref(false)
const rootRef = ref<HTMLElement | null>(null)
const panelRef = ref<HTMLElement | null>(null)
const panelStyle = ref<Record<string, string>>({})

const rootClass = 'relative inline-block'

function updatePanelPosition() {
  const root = rootRef.value
  const panel = panelRef.value
  if (!root || !panel) return

  const trigger = root.firstElementChild
  if (!(trigger instanceof HTMLElement)) return

  const tr = trigger.getBoundingClientRect()
  const pr = panel.getBoundingClientRect()
  if (pr.width <= 0 || pr.height <= 0) return

  const vw = window.innerWidth
  const vh = window.innerHeight
  const minTop = getAppHeaderBottom() + VIEWPORT_MARGIN

  // Prefer aligning the menu’s right edge with the trigger (compact controls on the left).
  let left = tr.right - pr.width
  left = Math.max(VIEWPORT_MARGIN, Math.min(left, vw - pr.width - VIEWPORT_MARGIN))

  let top = tr.bottom + VIEWPORT_MARGIN
  let maxHeight: string | undefined

  if (top + pr.height > vh - VIEWPORT_MARGIN) {
    const above = tr.top - pr.height - VIEWPORT_MARGIN
    if (above >= minTop) {
      top = above
    } else {
      top = minTop
      maxHeight = `${vh - minTop - VIEWPORT_MARGIN}px`
    }
  } else if (top < minTop) {
    top = minTop
    maxHeight = `${vh - minTop - VIEWPORT_MARGIN}px`
  }

  panelStyle.value = {
    top: `${top}px`,
    left: `${left}px`,
    ...(maxHeight ? { maxHeight } : {}),
  }
}

function handleClickOutside(event: MouseEvent) {
  const el = event.target
  if (!(el instanceof Node)) return
  // Panel is teleported to <body>; count it as inside the control.
  if (rootRef.value?.contains(el) || panelRef.value?.contains(el)) {
    return
  }
  open.value = false
}

function onViewportChange() {
  if (open.value) {
    updatePanelPosition()
  }
}

watch(open, async (isOpen) => {
  if (isOpen) {
    panelStyle.value = {}
    await nextTick()
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        updatePanelPosition()
      })
    })
    window.addEventListener('resize', onViewportChange)
    window.addEventListener('scroll', onViewportChange, true)
  } else {
    panelStyle.value = {}
    window.removeEventListener('resize', onViewportChange)
    window.removeEventListener('scroll', onViewportChange, true)
  }
})

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside)
  window.removeEventListener('resize', onViewportChange)
  window.removeEventListener('scroll', onViewportChange, true)
})
</script>

