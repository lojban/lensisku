<template>
  <UpsertPageLayout narrow :title="t('newThreadPage.title')">
    <template #trailing>
      <Button
        variant="cancel"
        :disabled="isSubmitting"
        @click="router.push(localePath('/activity'))"
      >
        {{ t('creation.cancel') }}
      </Button>
      <UpsertToolbarButton
        type="submit"
        form="new-discussion-form"
        :disabled="
          !canCreate || isSubmitting || !draft.markdown.trim() || draft.markdown.length > 10280
        "
        :loading="isSubmitting"
      >
        {{ t('creation.postDiscussion') }}
      </UpsertToolbarButton>
    </template>

    <p
      class="mb-4 rounded-lg border border-blue-100 bg-blue-50 px-3 py-2.5 text-sm leading-relaxed text-blue-800"
    >
      {{ t('newThreadPage.description') }}
    </p>
    <p v-if="!canCreate" class="text-sm text-amber-800" role="status">
      {{ t('creation.permissionRequired') }}
    </p>
    <CommentForm
      v-else
      form-id="new-discussion-form"
      hide-submit
      class="discussion-page-editor mb-0"
      :initial-values="initialValues"
      optional-title
      require-body
      :submit-label="t('creation.postDiscussion')"
      :is-submitting="isSubmitting"
      @draft-change="updateDraft"
      @submit="createNewThread"
    />
    <p v-if="error" role="alert" class="text-sm text-red-700 mt-3">{{ error }}</p>
  </UpsertPageLayout>
</template>
<script setup lang="ts">
import { invalidateActivityFeed } from '@/composables/useActivityFeed'
import { ref, computed, defineAsyncComponent } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { addComment } from '@/api'
import { Button } from '@packages/ui'
import UpsertToolbarButton from '@/components/layout/UpsertToolbarButton.vue'
import UpsertPageLayout from '@/components/layout/UpsertPageLayout.vue'
import { useSeoHead } from '@/composables/useSeoHead'
import { useLocalePath } from '@/composables/useLocalePath'
import { useCreationAccess } from '@/composables/useCreationAccess'
import { useUnsavedChanges } from '@/composables/useUnsavedChanges'
import {
  readDiscussionDraft,
  saveDiscussionDraft,
  clearDiscussionDraft,
  type DiscussionDraft,
} from '@/utils/discussionDraft'
const CommentForm = defineAsyncComponent(() => import('@/components/CommentForm.vue'))
const { t } = useI18n()
useSeoHead({ title: () => t('creation.discussion') })
const router = useRouter()
const localePath = useLocalePath()
const canCreate = useCreationAccess('discussion')
const isSubmitting = ref(false)
const error = ref('')
const draft = ref(readDiscussionDraft())
const initialValues = { subject: draft.value.subject, content: draft.value.markdown }
const guard = useUnsavedChanges(computed(() => JSON.stringify(draft.value)))
guard.reset()
function updateDraft(value: DiscussionDraft) {
  draft.value = value
  saveDiscussionDraft(value)
}
async function createNewThread(formData: { subject: string; content: unknown[] }) {
  if (isSubmitting.value || !canCreate.value) return
  isSubmitting.value = true
  error.value = ''
  try {
    const response = await addComment(formData)
    if (!response.data?.thread_id) throw new Error(t('creation.postError'))
    invalidateActivityFeed()
    clearDiscussionDraft()
    guard.reset()
    await router.push({
      path: localePath('/comments'),
      query: { thread_id: response.data.thread_id, scroll_to: response.data.comment_id },
    })
  } catch (err) {
    error.value =
      (err as { response?: { data?: { error?: string } } }).response?.data?.error ||
      t('creation.postError')
  } finally {
    isSubmitting.value = false
  }
}
</script>

<style scoped>
@media (max-width: 640px) {
  .discussion-page-editor :deep(.milkdown-block-handle) {
    display: none;
  }
}
</style>
