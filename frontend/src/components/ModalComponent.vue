<template>

  <Teleport to="body">
    <div
      v-if="show"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-[60]"
      @click="close"
    >

      <div
        class="bg-white rounded-lg max-w-2xl w-full p-3 sm:p-5 max-h-[90vh] flex flex-col overflow-hidden"
        @click.stop
      >

        <div class="mb-4 flex shrink-0 items-center justify-between">

          <h3 class="text-lg font-medium select-none"> {{ title }} </h3>
           <button class="text-gray-400 hover:text-gray-600" @click="close">
             <span class="text-xl font-medium"> <X class="h-6 w-6" :title="t('modal.close')" /> </span
            > </button
          >
        </div>
        <div class="modal-scroll-body">
          <slot />
        </div>
        <div v-if="$slots.footer" class="shrink-0 border-t pt-4 mt-4"> <slot name="footer" /> </div>

      </div>

    </div>
  </Teleport>

</template>

<script setup lang="ts">
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

