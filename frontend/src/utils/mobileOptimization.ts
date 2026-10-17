import { ref, computed, readonly, onMounted, onUnmounted } from 'vue'

export interface ViewportSize {
  width: number
  height: number
  isMobile: boolean
  isTablet: boolean
  isDesktop: boolean
  orientation: 'portrait' | 'landscape'
}

export function useViewport() {
  const width = ref(window.innerWidth)
  const height = ref(window.innerHeight)

  const isMobile = computed(() => width.value < 768)
  const isTablet = computed(() => width.value >= 768 && width.value < 1024)
  const isDesktop = computed(() => width.value >= 1024)
  const orientation = computed(() => (width.value > height.value ? 'landscape' : 'portrait'))

  const updateViewport = () => {
    width.value = window.innerWidth
    height.value = window.innerHeight
  }

  onMounted(() => {
    window.addEventListener('resize', updateViewport)
    window.addEventListener('orientationchange', updateViewport)
  })

  onUnmounted(() => {
    window.removeEventListener('resize', updateViewport)
    window.removeEventListener('orientationchange', updateViewport)
  })

  return {
    width: readonly(width),
    height: readonly(height),
    isMobile,
    isTablet,
    isDesktop,
    orientation,
  }
}

export function useTouchGestures() {
  const touchStartX = ref(0)
  const touchStartY = ref(0)
  const touchEndX = ref(0)
  const touchEndY = ref(0)

  const handleTouchStart = (event: TouchEvent) => {
    touchStartX.value = event.changedTouches[0].screenX
    touchStartY.value = event.changedTouches[0].screenY
  }

  const handleTouchEnd = (event: TouchEvent) => {
    touchEndX.value = event.changedTouches[0].screenX
    touchEndY.value = event.changedTouches[0].screenY
  }

  const swipeDirection = computed(() => {
    const deltaX = touchEndX.value - touchStartX.value
    const deltaY = touchEndY.value - touchStartY.value

    if (Math.abs(deltaX) > Math.abs(deltaY)) {
      return deltaX > 0 ? 'right' : 'left'
    } else {
      return deltaY > 0 ? 'down' : 'up'
    }
  })

  const swipeDistance = computed(() => {
    const deltaX = Math.abs(touchEndX.value - touchStartX.value)
    const deltaY = Math.abs(touchEndY.value - touchStartY.value)
    return Math.max(deltaX, deltaY)
  })

  return {
    touchStartX: readonly(touchStartX),
    touchStartY: readonly(touchStartY),
    touchEndX: readonly(touchEndX),
    touchEndY: readonly(touchEndY),
    swipeDirection,
    swipeDistance,
    handleTouchStart,
    handleTouchEnd,
  }
}

export function useMobileOptimizations() {
  const { isMobile, isTablet, orientation } = useViewport()
  const { swipeDirection, swipeDistance, handleTouchStart, handleTouchEnd } = useTouchGestures()

  // Mobile-specific optimizations
  const preventZoom = () => {
    if (isMobile.value) {
      const viewport = document.querySelector('meta[name="viewport"]')
      if (viewport) {
        viewport.setAttribute(
          'content',
          'width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no'
        )
      }
    }
  }

  const enableSmoothScroll = () => {
    document.documentElement.style.scrollBehavior = 'smooth'
  }

  const optimizeImages = () => {
    const images = document.querySelectorAll('img')
    images.forEach((img) => {
      img.loading = 'lazy'
      if (isMobile.value) {
        img.style.maxWidth = '100%'
        img.style.height = 'auto'
      }
    })
  }

  const optimizeInputs = () => {
    const inputs = document.querySelectorAll('input, textarea')
    inputs.forEach((input) => {
      if (isMobile.value) {
        ;(input as HTMLElement).style.fontSize = '16px' // Prevent zoom on iOS
        input.addEventListener('focus', () => {
          document.body.style.position = 'fixed'
        })
        input.addEventListener('blur', () => {
          document.body.style.position = ''
        })
      }
    })
  }

  onMounted(() => {
    preventZoom()
    enableSmoothScroll()
    optimizeImages()
    optimizeInputs()
  })

  return {
    isMobile,
    isTablet,
    orientation,
    swipeDirection,
    swipeDistance,
    handleTouchStart,
    handleTouchEnd,
    preventZoom,
    enableSmoothScroll,
    optimizeImages,
    optimizeInputs,
  }
}

export const debounce = <T extends (...args: unknown[]) => unknown>(
  func: T,
  wait: number
): ((...args: Parameters<T>) => void) => {
  let timeout: number
  return (...args: Parameters<T>) => {
    clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}
