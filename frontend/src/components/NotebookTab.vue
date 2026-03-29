<template>
   <button
    type="button"
    class="ui-tab"
    :class="active ? 'ui-tab--active' : 'ui-tab--inactive'"
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

