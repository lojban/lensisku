<template>
  <label :for="resolvedId" :class="['inline-flex items-center gap-2 cursor-pointer', hostClass]">
    <input
      :id="resolvedId"
      ref="nativeCheckbox"
      type="checkbox"
      :checked="isChecked"
      :disabled="disabled"
      :name="name"
      :value="value"
      v-bind="$attrs"
      @change="onChange"
    />
    <slot />
  </label>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { PropType } from 'vue'

const props = defineProps({
  modelValue: { type: [Boolean, Array] as PropType<boolean | unknown[]>, default: false },
  disabled: { type: Boolean, default: false },
  name: { type: String, default: undefined },
  value: { type: [String, Number], default: undefined },
  label: { type: String, default: undefined },
  id: { type: String, default: undefined },
  size: { type: String, default: 'md', validator: (v: string) => ['md', 'lg'].includes(v) },
  checkboxClass: { type: [String, Array, Object], default: '' },
})

const emit = defineEmits<{ 'update:modelValue': [value: boolean | unknown[]] }>()

defineOptions({ name: 'UiCheckbox', inheritAttrs: false })

const nativeCheckbox = ref<HTMLInputElement | null>(null)
const generatedId = `ui-checkbox-${Math.random().toString(36).slice(2, 9)}`
const resolvedId = computed(() => props.id ?? generatedId)

const isArrayModel = computed(() => Array.isArray(props.modelValue))

const isChecked = computed(() => {
  if (isArrayModel.value) {
    return (props.modelValue as unknown[]).includes(props.value)
  }
  return Boolean(props.modelValue)
})

const hostClass = computed(() => {
  const extra = Array.isArray(props.checkboxClass)
    ? props.checkboxClass.join(' ')
    : props.checkboxClass
  return [extra].filter(Boolean).join(' ')
})

function onChange(e: Event) {
  const target = e.target as HTMLInputElement | null
  const checked = target?.checked ?? false

  if (isArrayModel.value) {
    const arr = [...(props.modelValue as unknown[])]
    const idx = arr.indexOf(props.value)
    if (checked && idx === -1) arr.push(props.value)
    if (!checked && idx !== -1) arr.splice(idx, 1)
    emit('update:modelValue', arr)
    return
  }

  emit('update:modelValue', checked)
}

function focus() {
  nativeCheckbox.value?.focus()
}

defineExpose({ input: nativeCheckbox, focus })
</script>
