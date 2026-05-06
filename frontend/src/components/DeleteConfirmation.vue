<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { computed } from 'vue'

const { t } = useI18n()

const props = defineProps({
  show: {
    type: Boolean,
    required: true,
  },
  title: {
    type: String,
    default: '',
  },
  message: {
    type: String,
    default: '',
  },
  isDeleting: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['confirm', 'cancel'])

const translatedTitle = computed(() => props.title || t('deleteConfirmation.deleteDefinition'))

const translatedMessage = computed(
  () => props.message || t('deleteConfirmation.definitionWarning', { word: 'Untitled entry' })
)

// Keyboard navigation handler
function handleKeydown(e) {
  if (e.key === 'Escape') {
    emit('cancel')
  }
}
</script>

<template>
  <div
    v-if="show"
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
    tabindex="-1"
    role="dialog"
    aria-modal="true"
    @keydown.escape="handleKeydown"
  >
    <div class="bg-white rounded-lg p-6 max-w-md w-full">
      <h3 class="text-lg font-medium mb-4">{{ translatedTitle }}</h3>

      <p class="text-gray-600 mb-6">{{ translatedMessage }}</p>

      <div class="sr-only" aria-live="polite">
        {{
          isDeleting
            ? t('deleteConfirmation.deletionInProgress')
            : t('deleteConfirmation.readyForDeletion')
        }}
      </div>

      <div class="flex justify-end gap-3">
        <button class="ui-btn--cancel" @click="$emit('cancel')">
          {{ t('deleteConfirmation.cancel') }}
        </button>
        <button :disabled="isDeleting" class="ui-btn--delete" @click="$emit('confirm')">
          {{ isDeleting ? t('deleteConfirmation.deleting') : t('deleteConfirmation.delete') }}
        </button>
      </div>
    </div>
  </div>
</template>
