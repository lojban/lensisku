import { ref, provide, inject, type Ref, type InjectionKey } from 'vue'

/** Default global success toast visibility: 2.9 × 2 seconds (milliseconds). */
export const DEFAULT_SUCCESS_TOAST_DURATION_MS = 2.9 * 2 * 1000

export interface SuccessToastPayload {
  message: string
  duration?: number
}

const SUCCESS_TOAST_KEY = Symbol('successToast') as InjectionKey<{
  successToast: Ref<SuccessToastPayload | null>
  showSuccess: (message: string, durationMs?: number) => void
  clearSuccess: () => void
}>

export function provideSuccessToast() {
  const successToast = ref<SuccessToastPayload | null>(null)

  const showSuccess = (message: string, durationMs?: number) => {
    successToast.value = durationMs != null ? { message, duration: durationMs } : { message }
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
