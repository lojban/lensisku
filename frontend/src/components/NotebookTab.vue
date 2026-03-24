<template>
   <button
    class="px-3 py-2 font-medium transition-colors flex items-center gap-2 border-b-2"
    :class="[
      active
        ? 'border-blue-500 text-blue-600'
        : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300',
    ]"
    @click="$emit('click')"
  >
     <slot name="icon" /> <span class="hidden md:block"> {{ label }} </span> </button
  >
</template>

<script setup lang="ts">
defineProps({
  active: Boolean,
  label: {
    type: String,
    default: '',
  },
})

defineEmits(['click'])
</script>

