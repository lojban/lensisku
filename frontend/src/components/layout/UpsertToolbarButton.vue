<script setup lang="ts">
import { useAttrs } from 'vue'

import { Button } from '@packages/ui'

defineOptions({ inheritAttrs: false })

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

const attrs = useAttrs()
</script>

<template>
  <Button
    v-bind="attrs"
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
