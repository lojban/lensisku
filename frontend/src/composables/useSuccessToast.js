import { ref, provide, inject } from 'vue'

const SUCCESS_TOAST_KEY = Symbol('successToast')

export function provideSuccessToast() {
  const successToast = ref(null)

  const showSuccess = (message) => {
    successToast.value = { message }
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
