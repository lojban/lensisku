<template>
  <Button
    :tag="tag"
    :to="to"
    :href="href"
    :type="resolvedType"
    :variant="resolvedVariant"
    :size="size"
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
    <template v-if="hasLabel" #default>{{ label }}</template>
  </Button>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Plus } from 'lucide-vue-next'
import Button from './Button.vue'

/** Split `ui-btn--foo` from optional utilities (`w-full`, `mb-4`, …). */
function parseButtonClasses(input: string): { variant: string; utilities: string } {
  const tokens = input.trim().split(/\s+/).filter(Boolean)
  const variantIdx = tokens.findIndex((t) => t.startsWith('ui-btn--'))
  const variantToken = variantIdx >= 0 ? tokens[variantIdx] : null
  const variant = variantToken ? variantToken.slice('ui-btn--'.length) : 'primary'
  // Preserve other utility classes, including extra `ui-btn--*` tokens like `ui-btn--group-item`.
  const utilities = tokens.filter((_, idx) => idx !== variantIdx).join(' ')
  return { variant, utilities }
}

const props = defineProps({
  /** Underlying element/tag (`button`, `router-link`, `a`). */
  tag: { type: String, default: 'button' },
  /** Router destination when `tag="router-link"` (or when using implicit router mode). */
  to: { type: [String, Object], default: undefined },
  /** Anchor href when `tag="a"` (or when using implicit anchor mode). */
  href: { type: String, default: undefined },
  /** Native button type (defaults to `button` only for native button usage). */
  type: { type: String, default: undefined },
  /** Optional; when omitted, button is icon-only (use ariaLabel for a11y) */
  label: { type: String, default: '' },
  /** Aria-label for icon-only buttons; falls back to label when present */
  ariaLabel: { type: String, default: '' },
  /** One `ui-btn--*` token plus optional Tailwind utilities (e.g. `w-full ui-btn--create mb-4`). */
  buttonClasses: { type: String, default: 'ui-btn--primary' },
  /** Matches `Button` sizes (`md` compact, `lg` tall + `text-lg`). */
  size: { type: String, default: 'md', validator: (v: string) => ['md', 'lg'].includes(v) },
  iconClasses: { type: String, default: 'h-4 w-4' },
  /** `inline` keeps icon + label in one row, `stacked` puts icon above label. */
  contentLayout: {
    type: String,
    default: 'inline',
    validator: (v: string) => ['inline', 'stacked'].includes(v),
  },
  disabled: { type: Boolean, default: false },
})

defineEmits<{ click: [e: MouseEvent] }>()

const parsed = computed(() => parseButtonClasses(props.buttonClasses))

const resolvedVariant = computed(() => parsed.value.variant)

const contentLayoutClass = computed(() =>
  props.contentLayout === 'stacked'
    ? '!h-auto !min-h-0 !rounded-md [&>span]:flex-col [&>span]:items-center [&>span]:justify-center [&>span]:text-center'
    : ''
)

const rootClass = computed(() =>
  ['icon-btn-ui-layout', contentLayoutClass.value, parsed.value.utilities].filter(Boolean).join(' ')
)

const ariaLabelComputed = computed(() => props.ariaLabel || props.label || undefined)
const hasLabel = computed(() => props.label.trim().length > 0)
const resolvedType = computed(() =>
  props.tag === 'button' && !props.to && !props.href ? props.type || 'button' : props.type
)
</script>
