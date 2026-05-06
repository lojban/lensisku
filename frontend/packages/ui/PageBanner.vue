<template>
  <div v-if="show" :class="bannerClasses" role="status"><slot /></div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps({
  show: { type: Boolean, default: false },
  variant: {
    type: String,
    default: 'error',
    validator: (v: string) => ['error', 'warning'].includes(v),
  },
})

const bannerClasses = computed(() => {
  if (props.variant === 'warning') {
    return 'page-banner-fixed page-banner--warning'
  }
  return 'page-banner-fixed page-banner--error'
})
</script>
