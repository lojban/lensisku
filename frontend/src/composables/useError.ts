import { ref, provide, inject, type Ref, type InjectionKey } from 'vue'

export interface AppErrorValue {
  message: string
  details: unknown
}

const ERROR_KEY = Symbol('error') as InjectionKey<{
  error: Ref<AppErrorValue | null>
  showError: (message: string, details?: unknown) => void
  clearError: () => void
}>

export function provideError() {
  const error = ref<AppErrorValue | null>(null)
  const showError = (message: string, details: unknown = null) => {
    error.value = { message, details }
  }
  const clearError = () => {
    error.value = null
  }

  const ctx = {
    error,
    showError,
    clearError,
  }

  provide(ERROR_KEY, ctx)

  return ctx
}

export function useError() {
  const context = inject(ERROR_KEY)
  if (!context) {
    throw new Error('useError must be used within a component that has called provideError')
  }
  return context
}
