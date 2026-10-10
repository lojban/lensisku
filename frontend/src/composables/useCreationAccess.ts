import { computed, type Ref } from 'vue'
import { useAuth } from '@/composables/useAuth'

export function useCreationAccess(kind: 'discussion' | 'definition', editing?: Ref<boolean>) {
  const auth = useAuth()
  return computed(
    () =>
      auth.state.isLoggedIn &&
      auth.state.authorities.includes(
        kind === 'discussion'
          ? 'create_comment'
          : editing?.value
            ? 'edit_definition'
            : 'create_definition'
      )
  )
}
