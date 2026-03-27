<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="show"
        class="fixed left-1/2 top-1/2 z-[65] mx-auto w-fit max-w-[min(90vw,28rem)] -translate-x-1/2 -translate-y-1/2 rounded-lg px-4 py-3 shadow-lg"
        :class="{
          'border border-green-400 bg-green-100 text-green-800': type === 'success',
          'border border-red-400 bg-red-100 text-red-800': type === 'error',
        }"
      >
        <div class="flex flex-col gap-2 text-lg">
          <div class="flex items-start justify-end gap-2">
            <div class="min-w-0 flex-1 whitespace-pre-wrap break-words">
              <slot name="message">{{ message }}</slot>
            </div>
            <button
              type="button"
              class="shrink-0 rounded px-1 leading-none text-current hover:opacity-80"
              :aria-label="closeLabel"
              @click="closeToast"
            >
              &times;
            </button>
          </div>
          <!-- Injected Vue components (or pass #extra from parent). Default: extraComponent + extraProps. -->
          <slot name="extra">
            <component
              :is="extraComponent"
              v-if="extraComponent"
              v-bind="extraProps || {}"
            />
          </slot>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, onUnmounted, type Component } from 'vue'
import type { PropType } from 'vue'

const props = defineProps({
  show: {
    type: Boolean,
    default: false,
  },
  message: {
    type: String,
    default: '',
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
  closeLabel: {
    type: String,
    default: 'Close',
  },
  extraComponent: {
    type: Object as PropType<Component | null>,
    default: null,
  },
  extraProps: {
    type: Object as PropType<Record<string, unknown> | null>,
    default: null,
  },
})

const emit = defineEmits(['close'])

const show = ref(props.show)

let hideTimer: ReturnType<typeof setTimeout> | null = null

const closeToast = () => {
  show.value = false
  emit('close')
}

watch(
  () => props.show,
  (newVal) => {
    show.value = newVal
    if (hideTimer) {
      clearTimeout(hideTimer)
      hideTimer = null
    }
    if (newVal && props.duration > 0) {
      hideTimer = setTimeout(() => {
        closeToast()
      }, props.duration)
    }
  }
)

onUnmounted(() => {
  if (hideTimer) {
    clearTimeout(hideTimer)
  }
})
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

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
