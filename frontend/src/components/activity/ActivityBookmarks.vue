<template>

  <div class="space-y-4">

    <div
      v-for="comment in comments"
      :key="comment.comment_id"
      class="cursor-pointer"
      @click="
        $router.push(
          `/comments?thread_id=${comment.thread_id}&comment_id=${comment.parent_id}&scroll_to=${comment.comment_id}&valsi_id=${comment.valsi_id}&definition_id=${comment.definition_id || 0}`
        )
      "
    >
       <CommentItem
        :comment="comment"
        :valsi-id="comment.valsi_id"
        :natlang-word-id="comment.natlang_word_id"
        :definition-id="comment.definition_id"
      />
    </div>

    <div
      v-if="comments.length === 0"
      class="text-center py-8 sm:py-12 px-4 bg-gray-50 rounded-lg border border-gray-200"
    >

      <p class="text-gray-600"> {{ t('activityBookmarks.noBookmarks') }} </p>

    </div>

  </div>

</template>

<script setup lang="ts">
import type { PropType } from 'vue'
import CommentItem from '@/components/CommentItem.vue'
import { useI18n } from 'vue-i18n'
import type { CommentItemApiComment } from '@/types/comment'

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

