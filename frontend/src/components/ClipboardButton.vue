<template>
  <Button
    :variant="buttonVariant"
    type="button"
    class="inline-flex items-center justify-center"
    :title="currentTitle"
    :aria-label="currentTitle"
    @click.stop="copyToClipboard"
  >
    <Check v-if="copied" class="w-4 h-4" aria-hidden="true" />
    <ClipboardCopy v-else class="w-4 h-4" aria-hidden="true" />
  </Button>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { ref, computed, onUnmounted } from 'vue'
import { ClipboardCopy, Check } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

import { useError } from '@/composables/useError'
import { useSuccessToast } from '@/composables/useSuccessToast'

const props = defineProps({
  content: {
    type: String,
    required: true,
  },
  title: {
    type: String,
    default: 'Copy to clipboard',
  },
  variant: {
    type: String,
    default: 'button',
    validator: (value: string) => ['button', 'ghost', 'unstyled'].indexOf(value) !== -1,
  },
})

const emit = defineEmits(['copied', 'error'])

const { t } = useI18n()
const { showSuccess } = useSuccessToast()
const { showError } = useError()

const copied = ref(false)
const copiedTimer = ref<ReturnType<typeof setTimeout> | null>(null)

const currentTitle = computed(() =>
  copied.value ? t('components.error.copiedToClipboard') : props.title
)

const buttonVariant = computed(() => {
  if (props.variant === 'ghost') return 'assistant-bubble-action'
  if (props.variant === 'unstyled') return 'neutral'
  return 'empty'
})

const copyToClipboard = async () => {
  try {
    await navigator.clipboard.writeText(props.content)
    copied.value = true
    showSuccess(t('components.error.copiedToClipboard'))
    emit('copied')
    if (copiedTimer.value) clearTimeout(copiedTimer.value)
    copiedTimer.value = setTimeout(() => {
      copied.value = false
      copiedTimer.value = null
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
    showError(t('components.error.failedToCopy'))
    emit('error', err)
  }
}

onUnmounted(() => {
  if (copiedTimer.value) clearTimeout(copiedTimer.value)
})
</script>
