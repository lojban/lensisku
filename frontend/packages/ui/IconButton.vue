<template>
  <Button
    tag="button"
    type="button"
    :variant="resolvedVariant"
    :class="rootClass"
    :disabled="disabled"
    :aria-label="ariaLabelComputed"
    @click="$emit('click', $event)"
  >
    <template #icon>
      <slot name="icon">
        <Plus :class="iconClasses" />
      </slot>
    </template>
    <template v-if="label">{{ label }}</template>
  </Button>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Plus } from 'lucide-vue-next'
import Button from './Button.vue'

/** Split `ui-btn--foo` from optional utilities (`w-full`, `mb-4`, …). */
function parseButtonClasses(input: string): { variant: string; utilities: string } {
  const tokens = input.trim().split(/\s+/).filter(Boolean)
  const ui = tokens.find((t) => t.startsWith('ui-btn--'))
  const variant = ui ? ui.slice('ui-btn--'.length) : 'primary'
  const utilities = tokens.filter((t) => !t.startsWith('ui-btn--')).join(' ')
  return { variant, utilities }
}

const props = defineProps({
  /** Optional; when omitted, button is icon-only (use ariaLabel for a11y) */
  label: { type: String, default: '' },
  /** Aria-label for icon-only buttons; falls back to label when present */
  ariaLabel: { type: String, default: '' },
  /** One `ui-btn--*` token plus optional Tailwind utilities (e.g. `w-full ui-btn--create mb-4`). */
  buttonClasses: { type: String, default: 'ui-btn--primary' },
  iconClasses: { type: String, default: 'h-4 w-4' },
  disabled: { type: Boolean, default: false },
})

defineEmits<{ click: [e: MouseEvent] }>()

const parsed = computed(() => parseButtonClasses(props.buttonClasses))

const resolvedVariant = computed(() => parsed.value.variant)

const rootClass = computed(() => ['icon-btn-ui-layout', parsed.value.utilities].filter(Boolean).join(' '))

const ariaLabelComputed = computed(() => props.ariaLabel || props.label || undefined)
</script>
