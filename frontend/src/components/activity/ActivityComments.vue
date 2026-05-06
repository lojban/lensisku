<template>
  <div class="space-y-4">
    <div v-if="comments.length === 0" class="text-center py-8 bg-gray-50 rounded-lg">
      <MessageSquare class="mx-auto h-12 w-12 text-blue-400" />
      <p class="text-gray-600">{{ t('components.activityComments.noComments') }}</p>
    </div>

    <div
      v-for="comment in comments"
      :key="comment.comment_id"
      class="cursor-pointer"
      @click="
        router.push(
          `/comments?thread_id=${comment.thread_id}&comment_id=${comment.parent_id}&scroll_to=${comment.comment_id}&valsi_id=${comment.valsi_id}&definition_id=${comment.definition_id || 0}`
        )
      "
    >
      <CommentItem
        :key="comment.comment_id"
        :comment="comment"
        :flat-style="true"
        :show-context="true"
        :valsi-id="comment.valsi_id"
        :definition-id="comment.definition_id"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { PropType } from 'vue'
import { MessageSquare } from 'lucide-vue-next'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import CommentItem from '@/components/CommentItem.vue'
import type { CommentItemApiComment } from '@/types/comment'

const router = useRouter()
const { t } = useI18n()
defineProps({
  comments: {
    type: Array as PropType<CommentItemApiComment[]>,
    required: true,
  },
  formatDate: {
    type: Function as PropType<(d: number | string) => string>,
    required: true,
  },
})
</script>
