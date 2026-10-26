import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import { useRoute, useRouter } from 'vue-router'

export type ListMode = 'learn' | 'edit'

/** `?mode=edit` → edit; anything else (missing, invalid, `add_flashcard`, …) → learn. */
export function parseListMode(raw: unknown): ListMode {
  const v = Array.isArray(raw) ? raw[0] : raw
  return v === 'edit' ? 'edit' : 'learn'
}

/**
 * Learn/Edit mode of a course card list, kept in `?mode=` so it is linkable. Switching uses
 * `router.replace` with only the query changed: same route record → same component instance,
 * no remount, no refetch, no route overlay (that only fires on path changes).
 * Edit mode is owner-only; non-owners always get learn.
 */
export function useListMode(isOwner: MaybeRefOrGetter<boolean>) {
  const route = useRoute()
  const router = useRouter()
  const requested = computed(() => parseListMode(route.query.mode))
  const isEditMode = computed(() => toValue(isOwner) && requested.value === 'edit')
  const mode = computed<ListMode>(() => (isEditMode.value ? 'edit' : 'learn'))
  function setMode(next: ListMode) {
    if (next === mode.value) return
    const query = { ...route.query }
    if (next === 'edit') query.mode = 'edit'
    else delete query.mode
    void router.replace({ query, hash: route.hash })
  }
  return { mode, isEditMode, setMode }
}
