import { ref, provide, inject, markRaw, type Ref, type InjectionKey, type Component } from 'vue'

/** Default global success toast visibility: 2.9 × 2 seconds (milliseconds). */
export const DEFAULT_SUCCESS_TOAST_DURATION_MS = 2.9 * 2 * 1000

export interface SuccessToastPayload {
  message: string
  duration?: number
  /** Rendered below the message inside {@link ToastFloat} (e.g. action buttons). */
  extraComponent?: Component
  extraProps?: Record<string, unknown>
}

export interface SuccessToastOptions {
  duration?: number
  extraComponent?: Component
  extraProps?: Record<string, unknown>
}

const SUCCESS_TOAST_KEY = Symbol('successToast') as InjectionKey<{
  successToast: Ref<SuccessToastPayload | null>
  showSuccess: {
    (message: string, durationMs?: number): void
    (message: string, options: SuccessToastOptions): void
  }
  clearSuccess: () => void
}>

export function provideSuccessToast() {
  const successToast = ref<SuccessToastPayload | null>(null)

  function showSuccess(message: string, durationMs?: number): void
  function showSuccess(message: string, options: SuccessToastOptions): void
  function showSuccess(message: string, durationOrOptions?: number | SuccessToastOptions): void {
    if (durationOrOptions == null) {
      successToast.value = { message }
      return
    }
    if (typeof durationOrOptions === 'number') {
      successToast.value = { message, duration: durationOrOptions }
      return
    }
    const ec = durationOrOptions.extraComponent
    successToast.value = {
      message,
      duration: durationOrOptions.duration,
      extraComponent: ec != null ? markRaw(ec) : undefined,
      extraProps: durationOrOptions.extraProps,
    }
  }

  const clearSuccess = () => {
    successToast.value = null
  }

  const ctx = {
    successToast,
    showSuccess,
    clearSuccess,
  }

  provide(SUCCESS_TOAST_KEY, ctx)

  return ctx
}

export function useSuccessToast() {
  const ctx = inject(SUCCESS_TOAST_KEY, null)
  if (!ctx) {
    throw new Error('useSuccessToast must be used within App (provideSuccessToast)')
  }
  return ctx
}
