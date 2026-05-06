<template>
  <Teleport to="body">
    <Transition name="toast-float">
      <div v-if="show" class="toast-float-shell" role="presentation">
        <div
          class="toast-float-panel"
          :class="panelVariantClass"
          :role="type === 'error' ? 'alert' : 'status'"
          :aria-live="type === 'error' ? 'assertive' : 'polite'"
        >
          <div class="toast-float-body">
            <component
              :is="iconComponent"
              class="toast-float-icon h-6 w-6"
              :class="iconVariantClass"
              aria-hidden="true"
            />
            <div class="flex min-w-0 flex-1 flex-col gap-3">
              <div class="flex items-center gap-2">
                <div class="toast-float-message whitespace-pre-wrap break-words">
                  <slot name="message">{{ message }}</slot>
                </div>
                <button
                  type="button"
                  class="toast-float-close"
                  :aria-label="closeLabel"
                  @click="closeToast"
                >
                  <X class="h-4 w-4" aria-hidden="true" />
                </button>
              </div>

              <div v-if="$slots.extra || extraComponent" class="toast-float-extra">
                <slot name="extra">
                  <component :is="extraComponent" v-if="extraComponent" v-bind="extraProps || {}" />
                </slot>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, onUnmounted, computed, type Component } from 'vue'
import type { PropType } from 'vue'
import { CheckCircle2, CircleAlert, X } from 'lucide-vue-next'

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

const iconComponent = computed(() => (props.type === 'success' ? CheckCircle2 : CircleAlert))

const panelVariantClass = computed(() =>
  props.type === 'success' ? 'toast-float-panel--success' : 'toast-float-panel--error'
)

const iconVariantClass = computed(() =>
  props.type === 'success' ? 'toast-float-icon--success' : 'toast-float-icon--error'
)

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
.toast-float-enter-active,
.toast-float-leave-active {
  transition: opacity 0.22s ease;
}

.toast-float-enter-from,
.toast-float-leave-to {
  opacity: 0;
}

.toast-float-enter-active .toast-float-panel,
.toast-float-leave-active .toast-float-panel {
  transition:
    transform 0.22s cubic-bezier(0.34, 1.2, 0.64, 1),
    opacity 0.22s ease;
}

.toast-float-enter-from .toast-float-panel,
.toast-float-leave-to .toast-float-panel {
  opacity: 0;
  transform: scale(0.94) translateY(0.375rem);
}
</style>
