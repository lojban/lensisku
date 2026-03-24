<template>

  <div class="bg-white rounded-lg shadow-md p-6">

    <h2 class="text-2xl font-bold mb-6 text-gray-800"> {{ t('newThreadPage.title') }} </h2>

    <p class="text-gray-600 text-sm mb-2"> {{ t('newThreadPage.description') }} </p>
     <CommentForm
      ref="commentFormRef"
      :is-reply="false"
      :is-submitting="isSubmitting"
      @submit="createNewThread"
      @cancel="cancelCreation"
    />
  </div>

</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { addComment } from '@/api'
import CommentForm from '@/components/CommentForm.vue'
import { useSeoHead } from '@/composables/useSeoHead'

const { t, locale } = useI18n()

useSeoHead({ title: computed(() => t('newThreadPage.pageTitle')) })

const router = useRouter()
const isSubmitting = ref(false)
const commentFormRef = ref(null)

onMounted(() => {
  commentFormRef.value?.focusSubject()
})

const createNewThread = async (formData) => {
  try {
    isSubmitting.value = true
    const response = await addComment({
      subject: formData.subject,
      content: formData.content,
    })

    if (response.data?.thread_id) {
      router.push({
        path: '/comments',
        query: {
          thread_id: response.data.thread_id,
          scroll_to: response.data.comment_id,
        },
      })
    }
  } catch (error) {
    console.error('Error creating new thread:', error)
  } finally {
    isSubmitting.value = false
  }
}

const cancelCreation = () => {
  router.push('/comments')
}
</script>

<style scoped>
.shadow-md {
  box-shadow:
    0 4px 6px -1px rgba(0, 0, 0, 0.1),
    0 2px 4px -1px rgba(0, 0, 0, 0.06);
}
</style>

