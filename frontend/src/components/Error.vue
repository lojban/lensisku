<template>
   <ToastFloat
    :show="!!message"
    :message="displayMessage"
    type="error"
    :duration="0"
    :close-label="t('modal.close')"
    @close="$emit('close')"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import ToastFloat from './ToastFloat.vue'

const props = defineProps({
  message: String,
  details: String,
})

const { t } = useI18n()

const displayMessage = computed(() => {
  return props.details ? `${t(props.message)}\n\n${t(props.details)}` : t(props.message)
})

defineEmits(['close'])
</script>

