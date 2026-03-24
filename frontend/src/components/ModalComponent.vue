<template>
  <div
    v-if="show"
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-[60]"
    @click="close"
  >
    <div
      class="bg-white rounded-lg max-w-2xl w-full p-4 sm:p-6 max-h-[90vh] flex flex-col overflow-hidden"
      @click.stop
    >
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-medium select-none">
          {{ title }}
        </h3>
        <button class="text-gray-400 hover:text-gray-600" @click="close">
          <span class="text-xl font-medium">
            <X class="h-6 w-6" :title="t('modal.close')" />
          </span>
        </button>
      </div>
      <slot />

      <!-- Footer slot -->
      <div v-if="$slots.footer" class="border-t pt-4 mt-4">
        <slot name="footer" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
import { X } from 'lucide-vue-next'

const emit = defineEmits(['close'])
defineProps({
  show: {
    type: Boolean,
    required: true,
  },
  title: {
    type: String,
    default: '',
  },
})

const close = () => {
  emit('close')
}
</script>
