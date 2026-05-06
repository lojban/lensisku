<template>
  <div :class="alertClasses" class="p-3 rounded-lg space-y-2">
    <div v-if="label || $slots.label" class="flex items-center gap-2 text-xs">
      <slot name="label"> {{ label }} </slot>
    </div>

    <div class="text-sm"><slot /></div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps({
  type: {
    type: String,
    default: 'info',
    validator: (value: unknown) =>
      typeof value === 'string' && ['info', 'success', 'warning', 'error', 'tip'].includes(value),
  },
  label: {
    type: String,
    default: '',
  },
})

const typeMap = {
  info: { bg: 'bg-blue-50', border: 'border border-blue-200', text: 'text-blue-800' },
  success: { bg: 'bg-green-50', border: 'border border-green-200', text: 'text-green-800' },
  warning: { bg: 'bg-yellow-50', border: 'border border-yellow-200', text: 'text-yellow-800' },
  error: { bg: 'bg-red-50', border: 'border border-red-200', text: 'text-red-800' },
  tip: { bg: 'bg-zinc-50', border: 'border border-zinc-200', text: 'text-zinc-800' },
}

const alertClasses = computed(() => {
  const currentType = typeMap[props.type]
  return `${currentType.bg} ${currentType.border} ${currentType.text}`
})
</script>
