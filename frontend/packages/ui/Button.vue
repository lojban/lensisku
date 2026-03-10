<template>
  <component
    :is="tagComputed"
    :type="tagComputed === 'button' ? (type || 'button') : undefined"
    :to="tagComputed === 'router-link' ? to : undefined"
    :href="tagComputed === 'a' ? href : undefined"
    :disabled="disabled || loading"
    :class="buttonClasses"
    v-bind="$attrs"
    @click="handleClick"
  >
    <slot v-if="!loading" name="icon" />
    <component v-else :is="Spinner" class="w-4 h-4 shrink-0" />
    <span v-if="$slots.default" class="relative top-px"><slot /></span>
  </component>
</template>

<script setup>
import { computed } from 'vue'
import { RouterLink } from 'vue-router'

const VARIANT_CLASSES = {
  'aqua-white': 'btn-aqua-white',
  'aqua-orange': 'btn-aqua-orange',
  'aqua-amber': 'btn-aqua-amber',
  'aqua-yellow': 'btn-aqua-yellow',
  'aqua-lime': 'btn-aqua-lime',
  'aqua-teal': 'btn-aqua-teal',
  'aqua-emerald': 'btn-aqua-emerald',
  'aqua-cyan': 'btn-aqua-cyan',
  'aqua-sky': 'btn-aqua-sky',
  'aqua-blue': 'btn-aqua-blue',
  'aqua-indigo': 'btn-aqua-indigo',
  'aqua-violet': 'btn-aqua-violet',
  'aqua-purple': 'btn-aqua-purple',
  'aqua-fuchsia': 'btn-aqua-fuchsia',
  'aqua-pink': 'btn-aqua-pink',
  'aqua-rose': 'btn-aqua-rose',
  'aqua-slate': 'btn-aqua-slate',
  'aqua-zinc': 'btn-aqua-zinc',
  'aqua-gray': 'btn-aqua-gray',
  'aqua-red': 'btn-aqua-red',
  empty: 'btn-empty',
  cancel: 'btn-cancel',
  create: 'btn-create',
  createBase: 'btn-base',
}

const props = defineProps({
  variant: {
    type: String,
    default: 'aqua-white',
  },
  /** 'button' | 'router-link' | 'a' */
  tag: { type: String, default: 'button' },
  to: { type: [String, Object], default: undefined },
  href: { type: String, default: undefined },
  type: { type: String, default: undefined },
  disabled: { type: Boolean, default: false },
  loading: { type: Boolean, default: false },
  /** Extra class names (e.g. h-8, px-6) */
  class: { type: [String, Array, Object], default: '' },
})

const emit = defineEmits(['click'])

const tagComputed = computed(() => {
  if (props.tag === 'router-link' || props.to) return RouterLink
  if (props.tag === 'a' || props.href) return 'a'
  return 'button'
})

const buttonClasses = computed(() => {
  const base = VARIANT_CLASSES[props.variant] || props.variant || 'btn-aqua-white'
  const extra = Array.isArray(props.class) ? props.class.join(' ') : props.class
  return extra ? [base, extra].filter(Boolean).join(' ') : base
})

function handleClick(e) {
  if (props.loading || props.disabled) return
  emit('click', e)
}

const Spinner = {
  template: '<span class="inline-block w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" aria-hidden="true" />',
}
</script>
