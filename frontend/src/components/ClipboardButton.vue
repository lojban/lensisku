<template>
  <button
    class="inline-flex items-center justify-center ui-btn--empty"
    :title="title"
    @click.stop="copyToClipboard"
  >
    <ClipboardCopy class="w-4 h-4" />
  </button>
</template>

<script setup lang="ts">
import { ClipboardCopy } from 'lucide-vue-next'
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
})

const emit = defineEmits(['copied', 'error'])

const { t } = useI18n()
const { showSuccess } = useSuccessToast()
const { showError } = useError()

const copyToClipboard = async () => {
  try {
    await navigator.clipboard.writeText(props.content)
    showSuccess(t('components.error.copiedToClipboard'))
    emit('copied')
  } catch (err) {
    console.error('Failed to copy:', err)
    showError('components.error.failedToCopy')
    emit('error', err)
  }
}
</script>
