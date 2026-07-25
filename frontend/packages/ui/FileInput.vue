<template>
  <input
    :id="resolvedId"
    ref="input"
    type="file"
    v-bind="$attrs"
    @change="$emit('change', $event)"
  />
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

const props = defineProps({
  id: { type: String, default: undefined },
})

defineOptions({ inheritAttrs: false })

defineEmits<{ change: [e: Event] }>()

const input = ref<HTMLInputElement | null>(null)
const generatedId = `ui-file-${Math.random().toString(36).slice(2, 9)}`
const resolvedId = computed(() => props.id ?? generatedId)

defineExpose({ input })
</script>
