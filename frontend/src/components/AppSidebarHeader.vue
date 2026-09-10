<template>
  <div class="app-sidebar-header">
    <Button
      v-if="showClose"
      variant="assistant-icon-soft"
      type="button"
      :aria-label="closeLabel"
      @click="$emit('close')"
    >
      <X class="h-5 w-5" />
    </Button>
    <div v-if="$slots.default" class="app-sidebar-header__body">
      <slot />
    </div>
    <span v-else-if="title" class="app-sidebar-header__title">{{ title }}</span>
  </div>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { X } from '@lucide/vue'

defineProps({
  /** Show the mobile/drawer close control. */
  showClose: {
    type: Boolean,
    default: false,
  },
  /** Accessible label for the close button. */
  closeLabel: {
    type: String,
    default: 'Close',
  },
  /** Plain title when no default slot is provided. */
  title: {
    type: String,
    default: '',
  },
})

defineEmits<{
  close: []
}>()
</script>
