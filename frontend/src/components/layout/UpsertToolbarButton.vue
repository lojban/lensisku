<script setup lang="ts">
import { Button } from '@packages/ui'

withDefaults(
  defineProps<{
    variant?: string
    type?: string
    form?: string
    disabled?: boolean
    loading?: boolean
  }>(),
  {
    variant: 'create',
    type: 'button',
  }
)
</script>

<template>
  <Button
    :variant="variant"
    :type="type"
    :form="form"
    :disabled="disabled"
    :loading="loading"
    size="md"
  >
    <template v-if="$slots.icon" #icon>
      <slot name="icon" />
    </template>
    <slot />
  </Button>
</template>
