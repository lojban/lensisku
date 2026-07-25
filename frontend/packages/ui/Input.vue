<template>
  <input
    :id="resolvedId"
    ref="nativeInput"
    :value="displayValue"
    :type="type"
    :placeholder="placeholder"
    :disabled="disabled"
    :readonly="readonly"
    :autocomplete="autocomplete"
    :name="name"
    :class="hostClass"
    v-bind="$attrs"
    @input="onInput"
    @change="onChange"
  />
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

type Modifiers = {
  lazy?: boolean
  number?: boolean
  trim?: boolean
}

const props = defineProps({
  modelValue: { type: [String, Number], default: '' },
  type: { type: String, default: 'text' },
  placeholder: { type: String, default: '' },
  disabled: { type: Boolean, default: false },
  readonly: { type: Boolean, default: false },
  autocomplete: { type: String, default: undefined },
  name: { type: String, default: undefined },
  id: { type: String, default: undefined },
  size: { type: String, default: 'md', validator: (v: string) => ['md', 'lg'].includes(v) },
  inputClass: { type: [String, Array, Object], default: '' },
  modelModifiers: { type: Object as () => Modifiers, default: () => ({}) },
})

const emit = defineEmits<{ 'update:modelValue': [value: string | number] }>()

defineOptions({ name: 'UiInput', inheritAttrs: false })

const nativeInput = ref<HTMLInputElement | null>(null)
const generatedId = `ui-input-${Math.random().toString(36).slice(2, 9)}`
const resolvedId = computed(() => props.id ?? generatedId)

const hostClass = computed(() => {
  const extra = Array.isArray(props.inputClass) ? props.inputClass.join(' ') : props.inputClass
  return ['w-full', extra].filter(Boolean).join(' ')
})

const displayValue = computed(() => {
  if (props.modelValue == null) return ''
  return String(props.modelValue)
})

function normalizeValue(raw: string): string | number {
  let value: string | number = raw
  if (props.modelModifiers.trim) value = value.trim()
  if (props.modelModifiers.number) value = value === '' ? NaN : Number(value)
  return value
}

function onInput(e: Event) {
  if (props.modelModifiers.lazy) return
  const target = e.target as HTMLInputElement | null
  emit('update:modelValue', normalizeValue(target?.value ?? ''))
}

function onChange(e: Event) {
  const target = e.target as HTMLInputElement | null
  emit('update:modelValue', normalizeValue(target?.value ?? ''))
}

function focus() {
  nativeInput.value?.focus()
}

defineExpose({ input: nativeInput, focus })
</script>
