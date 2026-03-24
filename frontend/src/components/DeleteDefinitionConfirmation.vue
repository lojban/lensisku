<template>
  <div
    v-if="show"
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
  >
    <div class="bg-white rounded-lg p-6 max-w-md w-full">
      <h3 class="text-lg font-medium mb-4">
        {{ t('deleteConfirmation.deleteDefinition') }}
      </h3>
      <p class="text-gray-600 mb-6">
        {{ t('deleteConfirmation.definitionWarning', { word: definitionWord }) }}
      </p>
      <div class="flex justify-end gap-3">
        <button class="btn-cancel" @click="$emit('cancel')">
          {{ t('deleteConfirmation.cancel') }}
        </button>
        <button :disabled="isDeleting" class="btn-delete" @click="$emit('confirm')">
          {{
            isDeleting ? t('deleteConfirmation.deleting') : t('deleteConfirmation.deleteDefinition')
          }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  show: {
    type: Boolean,
    required: true,
  },
  title: {
    type: String,
    default: 'Delete Definition',
  },
  definitionWord: {
    type: String,
    required: true,
  },
  isDeleting: {
    type: Boolean,
    default: false,
  },
})

defineEmits(['confirm', 'cancel'])
</script>
