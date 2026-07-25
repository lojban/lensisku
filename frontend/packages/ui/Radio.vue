<template>
  <label :for="resolvedId" :class="['inline-flex items-center gap-2 cursor-pointer', hostClass]">
    <input
      :id="resolvedId"
      ref="nativeRadio"
      type="radio"
      :checked="isChecked"
      :value="value"
      :name="name"
      :disabled="disabled"
      v-bind="$attrs"
      @change="onChange"
    />
    <slot />
  </label>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

const props = defineProps({
  modelValue: { type: [String, Number, Boolean], default: undefined },
  value: { type: [String, Number, Boolean], default: 'on' },
  disabled: { type: Boolean, default: false },
  name: { type: String, default: undefined },
  label: { type: String, default: undefined },
  id: { type: String, default: undefined },
  size: { type: String, default: 'md', validator: (v: string) => ['md', 'lg'].includes(v) },
  radioClass: { type: [String, Array, Object], default: '' },
})

const emit = defineEmits<{ 'update:modelValue': [value: string | number | boolean] }>()

defineOptions({ name: 'UiRadio', inheritAttrs: false })

const nativeRadio = ref<HTMLInputElement | null>(null)
const generatedId = `ui-radio-${Math.random().toString(36).slice(2, 9)}`
const resolvedId = computed(() => props.id ?? generatedId)

const isChecked = computed(() => props.modelValue === props.value)

const hostClass = computed(() => {
  const extra = Array.isArray(props.radioClass) ? props.radioClass.join(' ') : props.radioClass
  return [extra].filter(Boolean).join(' ')
})

function onChange(e: Event) {
  const target = e.target as HTMLInputElement | null
  if (target?.checked) {
    emit('update:modelValue', props.value)
  }
}

function focus() {
  nativeRadio.value?.focus()
}

defineExpose({ input: nativeRadio, focus })
</script>
