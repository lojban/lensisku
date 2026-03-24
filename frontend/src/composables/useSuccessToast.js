import { ref, provide, inject } from 'vue'

/** Default global success toast visibility: 2.9 × 2 seconds (milliseconds). */
export const DEFAULT_SUCCESS_TOAST_DURATION_MS = 2.9 * 2 * 1000

const SUCCESS_TOAST_KEY = Symbol('successToast')

export function provideSuccessToast() {
  const successToast = ref(null)

  /**
   * @param {string} message
   * @param {number} [durationMs] ToastFloat duration (ms); defaults to 2.9×2s via DEFAULT_SUCCESS_TOAST_DURATION_MS in App when omitted
   */
  const showSuccess = (message, durationMs) => {
    successToast.value = durationMs != null ? { message, duration: durationMs } : { message }
  }

  const clearSuccess = () => {
    successToast.value = null
  }

  provide(SUCCESS_TOAST_KEY, {
    successToast,
    showSuccess,
    clearSuccess,
  })

  return {
    successToast,
    showSuccess,
    clearSuccess,
  }
}

export function useSuccessToast() {
  const ctx = inject(SUCCESS_TOAST_KEY, null)
  if (!ctx) {
    throw new Error('useSuccessToast must be used within App (provideSuccessToast)')
  }
  return ctx
}
