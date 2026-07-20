<template>
  <div :class="containerClasses" :style="containerStyles">
    <slot />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMobileOptimizations } from '@/utils/mobileOptimization'

interface Props {
  maxWidth?: 'sm' | 'md' | 'lg' | 'xl' | 'full'
  padding?: 'none' | 'sm' | 'md' | 'lg'
  centerContent?: boolean
  fullHeight?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  maxWidth: 'lg',
  padding: 'md',
  centerContent: true,
  fullHeight: false,
})

const { isMobile, isTablet: _isTablet } = useMobileOptimizations()

const containerClasses = computed(() => {
  const classes = ['responsive-container w-full transition-all duration-200 ease-out']

  // Max width classes
  const maxWidthClasses = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg',
    xl: 'max-w-xl',
    full: 'max-w-full',
  }
  classes.push(maxWidthClasses[props.maxWidth])

  // Padding classes
  const paddingClasses = {
    none: '',
    sm: isMobile.value ? 'px-2 py-1' : 'px-4 py-2',
    md: isMobile.value ? 'px-4 py-2' : 'px-6 py-4',
    lg: isMobile.value ? 'px-6 py-3' : 'px-8 py-6',
  }
  classes.push(paddingClasses[props.padding])

  // Layout classes
  if (props.centerContent) {
    classes.push('mx-auto')
  }

  if (props.fullHeight) {
    classes.push('min-h-screen')
  }

  // Mobile-specific classes
  if (isMobile.value) {
    classes.push('touch-pan-y')
  }

  return classes.join(' ')
})

const containerStyles = computed(() => {
  const styles: Record<string, string> = {}

  if (isMobile.value) {
    styles['-webkit-overflow-scrolling'] = 'touch'
    styles['overscroll-behavior-y'] = 'contain'
  }

  return styles
})
</script>

<style scoped>
.responsive-container {
  width: 100%;
  transition: all 0.2s ease-out;
}

/* Mobile-specific optimizations */
@media (max-width: 768px) {
  .responsive-container {
    user-select: none;
    touch-action: manipulation;
  }

  .responsive-container * {
    touch-action: manipulation;
  }

  .responsive-container input,
  .responsive-container textarea,
  .responsive-container select {
    font-size: 1rem;
    line-height: 1.5rem;
  }
}

/* Smooth transitions */
.responsive-container {
  transform: translate3d(0, 0, 0);
}
</style>
