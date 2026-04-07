<template>
   <component
    :is="tagComputed"
    :type="isNativeButton ? type || 'button' : undefined"
    :to="isRouterLink ? to : undefined"
    :href="isAnchor ? href : undefined"
    :disabled="disabled || loading"
    :class="buttonClasses"
    v-bind="$attrs"
    @click="handleClick"
    ><span
      class="inline-flex min-h-0 min-w-0 max-w-full items-center justify-center gap-2 [&_svg]:shrink-0"
      ><slot v-if="!loading" name="icon" /><span
        v-else
        class="inline-block shrink-0 rounded-full border-2 border-current border-t-transparent animate-spin"
        :class="spinnerSizeClass"
        aria-hidden="true"
      /><span v-if="$slots.default" class="min-w-0"><slot /></span></span></component
  >
</template>

<script setup lang="ts">
import { computed, type Component } from 'vue'
import { RouterLink } from 'vue-router'

/**
 * Resolves `variant` to a `ui-btn--*` class from the Tailwind brand book (`buttonUiThemeLayer`).
 * - Use the **suffix** after `ui-btn--` (e.g. `read`, `edit`, `toolbar`, `neutral`, `warning-orange`, `palette-teal`).
 * - Or pass a **full** class string starting with `ui-btn--`.
 */
function resolveUiBtnClass(variant: string): string {
  const v = variant.trim()
  if (!v) return 'ui-btn--neutral'
  if (v.startsWith('ui-btn--')) return v
  return `ui-btn--${v}`
}

/** Layout + typography overrides; pairs with `ui-btn--*` (theme layer may set compact h-6 / text-sm). */
const SIZE_CLASSES: Record<string, string> = {
  md: '',
  lg: '!h-10 !min-h-[2.5rem] !text-lg font-semibold leading-snug gap-2 rounded-full flex justify-center items-center transition-all disabled:opacity-75 disabled:cursor-not-allowed',
}

const props = defineProps({
  /**
   * Visual role: kebab-case **suffix** matching `tailwind.config.js` semantic classes
   * (`read`, `edit`, `toolbar`, `create`, `warning-orange`, `palette-teal`, …), or a full `ui-btn--…` string.
   */
  variant: {
    type: String,
    default: 'neutral',
  },
  /** 'button' | 'router-link' | 'a' */
  tag: { type: String, default: 'button' },
  to: { type: [String, Object], default: undefined },
  href: { type: String, default: undefined },
  type: { type: String, default: undefined },
  disabled: { type: Boolean, default: false },
  loading: { type: Boolean, default: false },
  /** `md` = default compact; `lg` = tall control + `text-lg` (e.g. auth form submit next to `h-10` inputs). */
  size: {
    type: String,
    default: 'md',
    validator: (v: string) => ['md', 'lg'].includes(v),
  },
  /** Extra class names (e.g. w-full, px-6) */
  class: { type: [String, Array, Object], default: '' },
})

const emit = defineEmits<{ click: [e: MouseEvent] }>()

const tagComputed = computed<Component | 'a' | 'button'>(() => {
  if (props.tag === 'router-link' || props.to) return RouterLink
  if (props.tag === 'a' || props.href) return 'a'
  return 'button'
})

const isRouterLink = computed(() => props.tag === 'router-link' || props.to != null)
const isAnchor = computed(() => props.tag === 'a' || props.href != null)
const isNativeButton = computed(() => tagComputed.value === 'button')

const buttonClasses = computed(() => {
  const base = resolveUiBtnClass(props.variant)
  const size = SIZE_CLASSES[props.size] ?? ''
  const extra = Array.isArray(props.class) ? props.class.join(' ') : props.class
  return [base, size, extra].filter(Boolean).join(' ')
})

const spinnerSizeClass = computed(() => (props.size === 'lg' ? 'h-6 w-6' : 'h-4 w-4'))

function handleClick(e: MouseEvent) {
  if (props.loading || props.disabled) return
  emit('click', e)
}
</script>
