<template>
  <!--
    Tab-matched loading layouts: same wrappers, gaps, and card chrome as the
    real Activity* components so content swap avoids CLS.
  -->
  <div role="status" aria-live="polite" aria-busy="true" class="min-w-0">
    <span class="sr-only">{{ label }}</span>

    <!-- News / recent changes: date group + space-y-3 list (ActivityChanges) -->
    <div v-if="variant === 'news' || variant === 'changes'">
      <div class="mb-8">
        <div class="mb-4 border-t pt-4">
          <div class="skeleton-pulse">
            <div class="skeleton-bone h-4 w-36" />
          </div>
        </div>
        <div class="space-y-3">
          <SkeletonCommentItem
            v-for="n in count"
            :key="`change-${n}`"
            :show-context="variant === 'news'"
          />
        </div>
      </div>
    </div>

    <!-- Discussion waves: space-y-4 + surface-activity-row (ActivityThreads) -->
    <div v-else-if="variant === 'threads'" class="space-y-4">
      <div class="space-y-4">
        <SkeletonThreadRow v-for="n in count" :key="`thread-${n}`" />
      </div>
    </div>

    <!-- All comments: space-y-4 wrappers around CommentItem (ActivityComments) -->
    <div v-else-if="variant === 'all_comments'" class="space-y-4">
      <div v-for="n in count" :key="`comment-${n}`">
        <SkeletonCommentItem :show-context="true" />
      </div>
    </div>

    <!-- All definitions: DictionaryEntries grid gap-4 mb-6 (ActivityDefinitions) -->
    <div v-else-if="variant === 'all_definitions'" class="space-y-4">
      <div class="dictionary-entries space-y-4">
        <div class="mb-6 grid gap-4">
          <SkeletonDefinitionCard v-for="n in definitionCount" :key="`def-${n}`" />
        </div>
      </div>
    </div>

    <!-- Fallback: keep prior activity-row look -->
    <div v-else class="space-y-4">
      <SkeletonThreadRow v-for="n in count" :key="`fallback-${n}`" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import SkeletonCommentItem from '@/components/activity/SkeletonCommentItem.vue'
import SkeletonDefinitionCard from '@/components/activity/SkeletonDefinitionCard.vue'
import SkeletonThreadRow from '@/components/activity/SkeletonThreadRow.vue'

const props = defineProps({
  variant: {
    type: String,
    required: true,
  },
  /** Typical viewport fill without overshooting a short page. */
  count: {
    type: Number,
    default: 5,
  },
})

const { t } = useI18n()

/** Definition cards are taller — fewer placeholders still fill the first screen. */
const definitionCount = computed(() => Math.max(3, Math.min(props.count, 4)))

const label = computed(() => t('loading'))
</script>
