<template>
  <section ref="rootEl" class="bg-white border rounded-xl p-3 sm:p-4" @focusout="onFocusOut">
    <button
      v-if="!expanded"
      type="button"
      class="w-full text-left text-gray-600 rounded-lg bg-gray-50 px-4 py-3 hover:bg-blue-50 focus-visible:outline-blue-600"
      @click="expand"
    >
      {{ t('creation.startDiscussion') }}
    </button>
    <template v-else>
      <p v-if="!canPost" class="text-sm text-amber-800 mb-3" role="status">
        {{ t('creation.permissionRequired') }}
      </p>
      <CommentForm
        v-else
        :key="editorKey"
        autofocus
        :initial-values="initialValues"
        optional-title
        require-body
        :submit-label="t('creation.postDiscussion')"
        :is-submitting="submitting"
        disable-border
        @draft-change="updateDraft"
        @submit="post"
      />
      <p v-if="error" role="alert" class="text-sm text-red-700 mt-2">{{ error }}</p>
    </template>
  </section>
</template>
<script setup lang="ts">
import { invalidateActivityFeed } from '@/composables/useActivityFeed'
import { computed, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { addComment } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useLocalePath } from '@/composables/useLocalePath'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'
import {
  readDiscussionDraft,
  saveDiscussionDraft,
  clearDiscussionDraft,
  type DiscussionDraft,
} from '@/utils/discussionDraft'
import CommentForm from '@/components/CommentForm.vue'
const { t } = useI18n()
const router = useRouter()
const localePath = useLocalePath()
const auth = useAuth()
const emit = defineEmits<{ posted: [] }>()
const rootEl = ref<HTMLElement | null>(null)
const expanded = ref(false)
const submitting = ref(false)
const error = ref('')
const draft = ref(readDiscussionDraft())
const initialValues = ref({ subject: draft.value.subject, content: draft.value.markdown })
const editorKey = ref(0)
const guard = useUnsavedChanges(computed(() => JSON.stringify(draft.value)))
guard.reset()
const canPost = computed(() => auth.state.authorities.includes('create_comment'))
/** Composer root, or Crepe UI that may render outside it (menus/tooltips). */
function isComposerInteraction(target: EventTarget | null): boolean {
  if (!(target instanceof Node)) return false
  if (rootEl.value?.contains(target)) return true
  return (
    target instanceof Element &&
    Boolean(
      target.closest(
        '.milkdown-tooltip, .tippy-box, .top-bar-heading-dropdown, [data-milkdown-tooltip]'
      )
    )
  )
}
function expand() {
  if (!auth.state.isLoggedIn) {
    void router.push(localePath('/comments/new-thread'))
    return
  }
  initialValues.value = { subject: draft.value.subject, content: draft.value.markdown }
  expanded.value = true
}
function collapse() {
  if (!expanded.value || submitting.value) return
  initialValues.value = { subject: draft.value.subject, content: draft.value.markdown }
  expanded.value = false
}
function onFocusOut(event: FocusEvent) {
  if (!expanded.value || submitting.value) return
  // Crepe toolbar buttons blur the editor without taking focus (relatedTarget null).
  // Stay open until focus lands on a real control outside, or an outside pointer.
  const next = event.relatedTarget
  if (!next || isComposerInteraction(next)) return
  collapse()
}
function onPointerDownOutside(event: PointerEvent) {
  if (!expanded.value || submitting.value) return
  if (isComposerInteraction(event.target)) return
  collapse()
}
watch(expanded, (isExpanded) => {
  if (isExpanded) {
    document.addEventListener('pointerdown', onPointerDownOutside, true)
  } else {
    document.removeEventListener('pointerdown', onPointerDownOutside, true)
  }
})
onUnmounted(() => {
  document.removeEventListener('pointerdown', onPointerDownOutside, true)
})
function updateDraft(value: DiscussionDraft) {
  draft.value = value
  saveDiscussionDraft(value)
}
async function post(form: { subject: string; content: unknown[] }) {
  if (submitting.value || !canPost.value) return
  submitting.value = true
  error.value = ''
  try {
    const response = await addComment(form)
    if (!response.data?.thread_id) throw new Error(t('creation.postError'))
    invalidateActivityFeed()
    clearDiscussionDraft()
    draft.value = { subject: '', markdown: '' }
    guard.reset()
    initialValues.value = { subject: '', content: '' }
    editorKey.value++
    expanded.value = false
    emit('posted')
  } catch (err) {
    error.value =
      (err as { response?: { data?: { error?: string } } }).response?.data?.error ||
      t('creation.postError')
  } finally {
    submitting.value = false
  }
}
</script>
