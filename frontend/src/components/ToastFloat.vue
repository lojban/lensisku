<template>
   <Teleport to="body"
    > <Transition name="fade"
      >
      <div
        v-if="show"
        class="fixed left-1/2 top-1/2 -translate-y-1/2 -translate-x-1/2 w-fit max-w-[90vw] mx-auto px-4 py-2 rounded-lg shadow-lg z-[65]"
        :class="{
          'border-green-400 bg-green-100 text-green-800': type === 'success',
          'border-red-400 bg-red-100 text-red-800': type === 'error',
        }"
      >

        <div class="flex items-center align-center gap-2 text-lg">
           {{ message }} <button @click="closeToast">&times;</button>
        </div>

      </div>
       </Transition
    > </Teleport
  >
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps({
  show: {
    type: Boolean,
    default: false,
  },
  message: {
    type: String,
    required: true,
  },
  type: {
    type: String,
    default: 'success',
    validator: (value: unknown) =>
      typeof value === 'string' && ['success', 'error'].includes(value),
  },
  duration: {
    type: Number,
    default: 3000,
  },
})

const emit = defineEmits(['close'])

const show = ref(props.show)

const closeToast = () => {
  show.value = false
  emit('close')
}

watch(
  () => props.show,
  (newVal) => {
    show.value = newVal
    if (newVal) {
      setTimeout(() => {
        closeToast()
      }, props.duration)
    }
  }
)
</script>

<style scoped>
@keyframes fade-in-up {
  from {
    opacity: 0;
    transform: translateY(1rem);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fade-in-up {
  animation: fade-in-up 0.3s ease-out;
}
</style>

