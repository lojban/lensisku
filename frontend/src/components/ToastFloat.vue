<template>
  <Teleport to="body">
    <Transition name="toast-float">
      <div v-if="open" class="toast" role="presentation">
        <div
          class="toast__panel"
          :class="type === 'error' ? 'toast__panel--error' : 'toast__panel--success'"
          :role="type === 'error' ? 'alert' : 'status'"
          :aria-live="type === 'error' ? 'assertive' : 'polite'"
        >
          <component
            :is="type === 'success' ? CheckCircle2 : CircleAlert"
            class="toast__icon"
            :class="type === 'error' ? 'text-red-600' : 'text-green-600'"
            aria-hidden="true"
          />

          <div class="toast__body">
            <div class="toast__row">
              <div class="toast__message whitespace-pre-wrap break-words">
                <slot name="message">{{ message }}</slot>
              </div>

              <div class="toast__actions" :role="type === 'error' ? 'group' : undefined">
                <ClipboardButton
                  v-if="type === 'error' && message"
                  variant="unstyled"
                  class="toast__btn"
                  :content="message"
                  :title="copyLabel"
                  :announce-success="false"
                />
                <Button
                  variant="neutral"
                  type="button"
                  class="toast__btn"
                  :aria-label="closeLabel"
                  @click="dismiss"
                >
                  <X class="h-4 w-4" aria-hidden="true" />
                </Button>
              </div>
            </div>

            <div v-if="$slots.extra || extraComponent" class="toast__extra">
              <slot name="extra">
                <component
                  :is="extraComponent"
                  v-if="extraComponent"
                  v-bind="extraProps || {}"
                />
              </slot>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { ref, watch, onUnmounted, type Component, type PropType } from 'vue'
import { CheckCircle2, CircleAlert, X } from '@lucide/vue'

import ClipboardButton from '@/components/ClipboardButton.vue'

const props = defineProps({
  show: { type: Boolean, default: false },
  message: { type: String, default: '' },
  type: {
    type: String,
    default: 'success',
    validator: (v: unknown) => typeof v === 'string' && ['success', 'error'].includes(v),
  },
  duration: { type: Number, default: 3000 },
  closeLabel: { type: String, default: 'Close' },
  copyLabel: { type: String, default: 'Copy to clipboard' },
  extraComponent: { type: Object as PropType<Component | null>, default: null },
  extraProps: { type: Object as PropType<Record<string, unknown> | null>, default: null },
})

const emit = defineEmits<{ close: [] }>()

const open = ref(props.show)
let hideTimer: ReturnType<typeof setTimeout> | null = null

const clearTimer = () => {
  if (!hideTimer) return
  clearTimeout(hideTimer)
  hideTimer = null
}

const dismiss = () => {
  open.value = false
  clearTimer()
  emit('close')
}

watch(
  () => props.show,
  (visible) => {
    open.value = visible
    clearTimer()
    if (visible && props.duration > 0) {
      hideTimer = setTimeout(dismiss, props.duration)
    }
  }
)

onUnmounted(clearTimer)
</script>

<style scoped>
.toast {
  pointer-events: none;
  position: fixed;
  inset: 0;
  z-index: 65;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}

.toast__panel {
  pointer-events: auto;
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  width: 100%;
  max-width: min(90vw, 28rem);
  padding: 1rem 1.25rem;
  overflow: hidden;
  border-radius: 1rem;
  border: 1px solid rgb(229 231 235);
  background: #fff;
  color: rgb(31 41 55);
  box-shadow:
    0 0.75px 3px rgba(0, 0, 0, 0.04),
    0 6px 16px rgba(0, 0, 0, 0.06);
}

.toast__panel--success {
  border-color: rgb(134 239 172);
}

.toast__panel--error {
  border-color: rgb(252 165 165);
}

.toast__icon {
  width: 1.5rem;
  height: 1.5rem;
  flex-shrink: 0;
}

.toast__body {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 0.75rem;
}

.toast__row {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
}

.toast__message {
  min-width: 0;
  flex: 1;
  max-height: min(60dvh, 24rem);
  overflow-y: auto;
  overscroll-behavior: contain;
  font-size: 0.875rem;
  font-weight: 500;
  line-height: 1.375;
}

@media (min-width: 640px) {
  .toast__message {
    font-size: 1rem;
  }
}

.toast__actions {
  display: flex;
  flex-direction: row;
  flex-wrap: nowrap;
  flex-shrink: 0;
  align-items: center;
  gap: 0.25rem;
}

.toast__actions :deep(.toast__btn) {
  box-sizing: border-box !important;
  display: inline-flex !important;
  width: 2rem !important;
  min-width: 2rem !important;
  height: 2rem !important;
  min-height: 2rem !important;
  padding: 0 !important;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 9999px !important;
  color: rgb(107 114 128);
}

.toast__actions :deep(.toast__btn:hover) {
  background: rgb(243 244 246);
  color: rgb(31 41 55);
}

.toast__extra {
  border-top: 1px solid rgb(243 244 246);
  padding-top: 0.75rem;
}

.toast-float-enter-active,
.toast-float-leave-active {
  transition: opacity 0.22s ease;
}

.toast-float-enter-from,
.toast-float-leave-to {
  opacity: 0;
}

.toast-float-enter-active .toast__panel,
.toast-float-leave-active .toast__panel {
  transition:
    transform 0.22s cubic-bezier(0.34, 1.2, 0.64, 1),
    opacity 0.22s ease;
}

.toast-float-enter-from .toast__panel,
.toast-float-leave-to .toast__panel {
  opacity: 0;
  transform: scale(0.94) translateY(0.375rem);
}
</style>
