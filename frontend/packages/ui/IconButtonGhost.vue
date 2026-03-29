<template>
  <button
    type="button"
    :class="buttonClass"
    :aria-label="ariaLabelComputed"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps({
  ariaLabel: { type: String, default: '' },
  disabled: { type: Boolean, default: false },
  compact: { type: Boolean, default: false },
})

defineEmits<{ click: [e: MouseEvent] }>()

const ariaLabelComputed = computed(() => props.ariaLabel || undefined)

const buttonClass = computed(() =>
  ['icon-btn-ghost', props.compact && 'icon-btn-ghost--compact'].filter(Boolean).join(' '),
)
</script>
