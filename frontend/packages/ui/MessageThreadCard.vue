<template>
  <div
    class="message-thread-card"
    :class="{ 'message-thread-card--clickable': clickable }"
    @click="onClick"
  >
    <slot />
  </div>
</template>

<script setup lang="ts">
const props = defineProps({
  /** Adds pointer cursor; emit `click` when the row should navigate. */
  clickable: { type: Boolean, default: false },
})

const emit = defineEmits<{ click: [e: MouseEvent] }>()

function onClick(e: MouseEvent) {
  if (props.clickable) emit('click', e)
}
</script>
