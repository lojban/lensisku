<template>
  <textarea
    :id="resolvedId"
    ref="nativeTextarea"
    :value="displayValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :readonly="readonly"
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
  modelValue: { type: String, default: '' },
  placeholder: { type: String, default: '' },
  disabled: { type: Boolean, default: false },
  readonly: { type: Boolean, default: false },
  name: { type: String, default: undefined },
  id: { type: String, default: undefined },
  size: { type: String, default: 'md', validator: (v: string) => ['md', 'lg'].includes(v) },
  textareaClass: { type: [String, Array, Object], default: '' },
  modelModifiers: { type: Object as () => Modifiers, default: () => ({}) },
})

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

defineOptions({ name: 'UiTextarea', inheritAttrs: false })

const generatedId = `ui-textarea-${Math.random().toString(36).slice(2, 9)}`
const resolvedId = computed(() => props.id ?? generatedId)
const nativeTextarea = ref<HTMLTextAreaElement | null>(null)

const hostClass = computed(() => {
  const extra = Array.isArray(props.textareaClass)
    ? props.textareaClass.join(' ')
    : props.textareaClass
  return ['w-full', extra].filter(Boolean).join(' ')
})

const displayValue = computed(() => props.modelValue ?? '')

function normalizeValue(raw: string): string {
  let value = raw
  if (props.modelModifiers.trim) value = value.trim()
  return value
}

function onInput(e: Event) {
  if (props.modelModifiers.lazy) return
  const target = e.target as HTMLTextAreaElement | null
  emit('update:modelValue', normalizeValue(target?.value ?? ''))
}

function onChange(e: Event) {
  const target = e.target as HTMLTextAreaElement | null
  emit('update:modelValue', normalizeValue(target?.value ?? ''))
}

function focus() {
  nativeTextarea.value?.focus()
}

defineExpose({ textarea: nativeTextarea, focus })
</script>
