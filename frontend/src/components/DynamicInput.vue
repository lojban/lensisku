<template>

  <div>
     <input
      v-if="!useTextarea"
      :id="id"
      v-model="inputValue"
      type="text"
      class="block w-full input-field disabled:bg-gray-100 disabled:cursor-not-allowed disabled:opacity-75 h-10"
      :disabled="isAnalyzing || isSubmitting || prefilledWord || isEditMode"
      :readonly="prefilledWord || isEditMode"
      @input="handleInput"
      @paste="handlePaste"
    /> <textarea
      v-else
      :id="id"
      v-model="inputValue"
      rows="5"
      maxlength="4000"
      class="textarea-field"
      :disabled="isAnalyzing || isSubmitting || prefilledWord || isEditMode"
      :readonly="prefilledWord || isEditMode"
      @input="handleInput"
      @paste="handlePaste"
    />
  </div>

</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps({
  id: {
    type: String,
    default: '0',
  },
  modelValue: {
    type: String,
    default: '',
  },
  isAnalyzing: Boolean,
  isSubmitting: Boolean,
  prefilledWord: Boolean,
  isEditMode: Boolean,
})

const emit = defineEmits(['update:modelValue', 'input', 'clear-analysis'])

const inputValue = ref(props.modelValue)
const useTextarea = ref(false)

const checkLength = (text) => {
  useTextarea.value = text.length > 200
}

const handleInput = (event) => {
  const value = event.target.value
  checkLength(value)
  emit('update:modelValue', value)
  emit('input', event)
  emit('clear-analysis')
}

const handlePaste = async (event) => {
  event.preventDefault()
  const pastedText = event.clipboardData.getData('text')
  const currentValue = inputValue.value || ''
  const cursorPos = event.target.selectionStart

  const newValue =
    currentValue.slice(0, cursorPos) + pastedText + currentValue.slice(event.target.selectionEnd)

  inputValue.value = newValue
  checkLength(newValue)
  emit('update:modelValue', newValue)
  emit('clear-analysis')
}

watch(
  () => props.modelValue,
  (newValue) => {
    inputValue.value = newValue
    checkLength(newValue)
  }
)
</script>

