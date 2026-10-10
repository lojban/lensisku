import { onBeforeUnmount, onMounted, ref, type Ref } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'

export const unsavedPromptOpen = ref(false)
let resolvePending: ((leave: boolean) => void) | undefined
export function resolveUnsavedPrompt(leave: boolean) {
  unsavedPromptOpen.value = false
  resolvePending?.(leave)
  resolvePending = undefined
}
/** Snapshot after initialization; successful saves and draft transfers explicitly reset it. */
export function useUnsavedChanges(value: Ref<string>) {
  const baseline = ref<string | null>(null)
  const reset = () => {
    baseline.value = value.value
  }
  const dirty = () => baseline.value !== null && baseline.value !== value.value
  const confirmLeave = () => {
    if (!dirty()) return true
    return new Promise<boolean>((resolve) => {
      resolvePending?.(false)
      resolvePending = resolve
      unsavedPromptOpen.value = true
    })
  }
  const beforeUnload = (event: BeforeUnloadEvent) => {
    if (!dirty()) return
    event.preventDefault()
    event.returnValue = ''
  }
  onBeforeRouteLeave(confirmLeave)
  onMounted(() => window.addEventListener('beforeunload', beforeUnload))
  onBeforeUnmount(() => window.removeEventListener('beforeunload', beforeUnload))
  return { reset, confirmLeave }
}
