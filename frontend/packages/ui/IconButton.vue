<template>
   <button
    type="button"
    class="icon-btn-ui-layout"
    :class="buttonClasses"
    :aria-label="ariaLabelComputed"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
     <slot name="icon"> <CirclePlus :class="iconClasses" /> </slot> <span
      v-if="label"
      class="inline-flex items-center gap-2"
      >{{ label }}</span
    > </button
  >
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { CirclePlus } from 'lucide-vue-next'

const props = defineProps({
  /** Optional; when omitted, button is icon-only (use ariaLabel for a11y) */
  label: { type: String, default: '' },
  /** Aria-label for icon-only buttons; falls back to label when present */
  ariaLabel: { type: String, default: '' },
  buttonClasses: { type: String, default: 'ui-btn--primary' },
  iconClasses: { type: String, default: 'h-4 w-4' },
  disabled: { type: Boolean, default: false },
})

defineEmits<{ click: [e: MouseEvent] }>()

const ariaLabelComputed = computed(() => props.ariaLabel || props.label || undefined)
</script>

