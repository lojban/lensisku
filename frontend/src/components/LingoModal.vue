<template>
  <Teleport to="body">
    <div v-if="show" class="lingo-modal-backdrop" role="presentation" @click="close">
      <div
        class="lingo-modal-panel"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="titleId"
        @click.stop
      >
        <div class="lingo-modal-header">
          <h3 :id="titleId" class="lingo-modal-title">{{ title }}</h3>
          <button
            type="button"
            class="lingo-icon-btn lingo-icon-btn--muted h-8 w-8"
            :aria-label="t('modal.close')"
            @click="close"
          >
            <X class="h-5 w-5" />
          </button>
        </div>
        <div class="lingo-modal-body">
          <slot />
        </div>
        <div v-if="$slots.footer" class="lingo-modal-footer">
          <slot name="footer" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { X } from '@lucide/vue'
import { useI18n } from 'vue-i18n'

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

const emit = defineEmits(['close'])
const { t } = useI18n()
const titleId = 'lingo-modal-title'

function close() {
  emit('close')
}
</script>
