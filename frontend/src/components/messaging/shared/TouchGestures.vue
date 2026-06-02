<template>
  <div
    ref="containerRef"
    :class="gestureClasses"
    @touchstart="handleTouchStart"
    @touchmove="handleTouchMove"
    @touchend="handleTouchEnd"
    @touchcancel="handleTouchEnd"
  >
    <slot />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useMobileOptimizations } from '@/utils/mobileOptimization'

interface Props {
  swipeLeftAction?: () => void
  swipeRightAction?: () => void
  swipeUpAction?: () => void
  swipeDownAction?: () => void
  swipeThreshold?: number
  preventDefault?: boolean
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  swipeThreshold: 50,
  preventDefault: true,
  disabled: false,
})

const { isMobile } = useMobileOptimizations()

// Touch state
const containerRef = ref<HTMLElement>()
const touchStart = ref({ x: 0, y: 0, time: 0 })
const touchEnd = ref({ x: 0, y: 0, time: 0 })
const isSwiping = ref(false)

const gestureClasses = computed(() => ({
  'touch-gestures': true,
  'touch-gestures--disabled': props.disabled || !isMobile.value,
  'touch-gestures--swiping': isSwiping.value,
}))

// Touch event handlers
const handleTouchStart = (event: TouchEvent) => {
  if (props.disabled || !isMobile.value) return

  const touch = event.changedTouches[0]
  touchStart.value = {
    x: touch.clientX,
    y: touch.clientY,
    time: Date.now(),
  }

  isSwiping.value = false

  if (props.preventDefault) {
    event.preventDefault()
  }
}

const handleTouchMove = (event: TouchEvent) => {
  if (props.disabled || !isMobile.value) return

  const touch = event.changedTouches[0]
  const deltaX = Math.abs(touch.clientX - touchStart.value.x)
  const deltaY = Math.abs(touch.clientY - touchStart.value.y)

  // Only consider it a swipe if there's significant movement
  if (deltaX > 10 || deltaY > 10) {
    isSwiping.value = true
  }

  if (props.preventDefault && isSwiping.value) {
    event.preventDefault()
  }
}

const handleTouchEnd = (event: TouchEvent) => {
  if (props.disabled || !isMobile.value) return

  const touch = event.changedTouches[0]
  touchEnd.value = {
    x: touch.clientX,
    y: touch.clientY,
    time: Date.now(),
  }

  if (isSwiping.value) {
    handleSwipe()
  }

  isSwiping.value = false

  if (props.preventDefault) {
    event.preventDefault()
  }
}

const handleSwipe = () => {
  const deltaX = touchEnd.value.x - touchStart.value.x
  const deltaY = touchEnd.value.y - touchStart.value.y
  const deltaTime = touchEnd.value.time - touchStart.value.time

  // Only handle swipe if it's fast enough and within time limit
  if (deltaTime > 500) return

  const absDeltaX = Math.abs(deltaX)
  const absDeltaY = Math.abs(deltaY)

  // Check if it's a horizontal or vertical swipe
  if (absDeltaX > absDeltaY) {
    // Horizontal swipe
    if (absDeltaX > props.swipeThreshold) {
      if (deltaX > 0 && props.swipeRightAction) {
        props.swipeRightAction()
      } else if (deltaX < 0 && props.swipeLeftAction) {
        props.swipeLeftAction()
      }
    }
  } else {
    // Vertical swipe
    if (absDeltaY > props.swipeThreshold) {
      if (deltaY > 0 && props.swipeDownAction) {
        props.swipeDownAction()
      } else if (deltaY < 0 && props.swipeUpAction) {
        props.swipeUpAction()
      }
    }
  }
}

// Haptic feedback for mobile devices
const triggerHapticFeedback = (type: 'light' | 'medium' | 'heavy' = 'light') => {
  if (!isMobile.value || !window.navigator.vibrate) return

  const patterns = {
    light: [10],
    medium: [20],
    heavy: [40],
  }

  window.navigator.vibrate(patterns[type])
}

// Expose haptic feedback for parent components
defineExpose({
  triggerHapticFeedback,
})

// Lifecycle
onMounted(() => {
  // Add passive event listeners for better performance
  if (containerRef.value && isMobile.value) {
    containerRef.value.addEventListener('touchstart', handleTouchStart, {
      passive: !props.preventDefault,
    })
    containerRef.value.addEventListener('touchmove', handleTouchMove, {
      passive: !props.preventDefault,
    })
  }
})

onUnmounted(() => {
  if (containerRef.value && isMobile.value) {
    containerRef.value.removeEventListener('touchstart', handleTouchStart)
    containerRef.value.removeEventListener('touchmove', handleTouchMove)
  }
})
</script>

<style scoped>
.touch-gestures {
  @apply relative;
  touch-action: pan-y pinch-zoom;
}

.touch-gestures--disabled {
  touch-action: auto;
}

.touch-gestures--swiping {
  @apply select-none;
}

/* Visual feedback for swiping */
.touch-gestures--swiping::after {
  content: '';
  @apply absolute inset-0 bg-blue-500 opacity-5 pointer-events-none;
  transition: opacity 0.2s ease-out;
}

/* Prevent text selection during swipe */
.touch-gestures--swiping * {
  @apply select-none;
  user-select: none;
  -webkit-user-select: none;
}

/* Improve touch target size on mobile */
@media (max-width: 768px) {
  .touch-gestures {
    min-height: 44px; /* iOS touch target minimum */
  }
}

/* Smooth transitions */
.touch-gestures {
  @apply transition-transform duration-200 ease-out;
}

/* Reduce motion for users who prefer it */
@media (prefers-reduced-motion: reduce) {
  .touch-gestures {
    transition: none;
  }
}
</style>
