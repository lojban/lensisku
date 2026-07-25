<template>
  <div class="relative">
    <Component
      :is="selectedIcon"
      v-if="selectedIcon"
      class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-500 max-sm:left-1/2 max-sm:-translate-x-1/2"
      aria-hidden="true"
    />
    <select
      :id="resolvedId"
      ref="nativeSelect"
      :disabled="disabled"
      :name="name"
      :aria-label="label"
      :class="hostClass"
      :multiple="multiple"
      v-bind="$attrs"
      @change="onChange"
    >
      <option
        v-for="opt in normalizedOptions"
        :key="opt.value"
        :value="opt.value"
        :disabled="opt.disabled"
        :selected="isOptionSelected(opt.value)"
      >
        {{ opt.label }}
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { PropType } from 'vue'

interface Option {
  [key: string]: unknown
  value: string | number | null
  label: string
  disabled?: boolean
  icon?: unknown
}

type OptionInput = Option | string | number

type ModelValue = string | number | null | unknown[]

type ModelModifiers = {
  number?: boolean
}

const props = defineProps({
  modelValue: { type: [String, Number, Array] as PropType<ModelValue>, default: '' },
  options: { type: Array as PropType<OptionInput[]>, default: () => [] as OptionInput[] },
  optionValue: { type: String, default: 'value' },
  optionLabel: { type: String, default: 'label' },
  optionDisabled: { type: String, default: 'disabled' },
  optionIcon: { type: String, default: 'icon' },
  label: { type: String, default: undefined },
  id: { type: String, default: undefined },
  placeholder: { type: String, default: undefined },
  disabled: { type: Boolean, default: false },
  name: { type: String, default: undefined },
  size: { type: String, default: 'md', validator: (v: string) => ['md', 'lg'].includes(v) },
  selectClass: { type: [String, Array, Object], default: '' },
  multiple: { type: Boolean, default: false },
  modelModifiers: { type: Object as () => ModelModifiers, default: () => ({}) },
})

const emit = defineEmits<{ 'update:modelValue': [value: ModelValue] }>()

defineOptions({ name: 'UiSelect', inheritAttrs: false })

const nativeSelect = ref<HTMLSelectElement | null>(null)
const generatedId = `ui-select-${Math.random().toString(36).slice(2, 9)}`
const resolvedId = computed(() => props.id ?? generatedId)

const hostClass = computed(() => {
  const extra = Array.isArray(props.selectClass) ? props.selectClass.join(' ') : props.selectClass
  return [
    'w-full',
    selectedIcon.value ? 'pl-9 max-sm:pl-0 max-sm:pr-2 max-sm:w-11 max-sm:text-transparent' : '',
    extra,
  ]
    .filter(Boolean)
    .join(' ')
})

const normalizedOptions = computed<Option[]>(() => {
  return props.options.map((opt) => {
    if (typeof opt === 'string' || typeof opt === 'number') {
      return { value: String(opt), label: String(opt) }
    }
    return {
      value: opt[props.optionValue] as string | number | null,
      label: String(opt[props.optionLabel]),
      disabled: Boolean(opt[props.optionDisabled]),
      icon: opt[props.optionIcon],
    }
  })
})

const isMultiple = computed(() => props.multiple || Array.isArray(props.modelValue))

function isOptionSelected(value: string | number | null): boolean {
  if (isMultiple.value) {
    const selected = props.modelValue as unknown[]
    return selected.some((v) => String(v) === String(value))
  }
  return String(props.modelValue) === String(value)
}

const selectedIcon = computed(() => {
  const selected = normalizedOptions.value.find((opt) => isOptionSelected(opt.value))
  return selected?.icon
})

function normalizeValue(raw: string): string | number {
  if (props.modelModifiers.number) {
    const n = Number(raw)
    return Number.isNaN(n) ? raw : n
  }
  return raw
}

function onChange(e: Event) {
  const target = e.target as HTMLSelectElement | null
  if (!target) return

  if (isMultiple.value) {
    const selected = Array.from(target.selectedOptions).map((o) => normalizeValue(o.value))
    emit('update:modelValue', selected)
    return
  }

  const raw = target.value ?? ''
  emit('update:modelValue', normalizeValue(raw))
}

function focus() {
  nativeSelect.value?.focus()
}

defineExpose({ select: nativeSelect, focus })
</script>
