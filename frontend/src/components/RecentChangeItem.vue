<template>
  <!-- Comment-type: reuse CommentItem so reactions and save button are shown -->
  <CommentItem
    v-if="change.change_type === 'comment'"
    :comment="mappedComment"
    :reply-enabled="true"
    :show-context="true"
    :valsi-id="change.valsi_id || 0"
    :definition-id="change.definition_id || 0"
    @reply="handleReply"
  />
  <!-- Other change types: existing layout -->
  <div
    v-else
    class="comment-item bg-white border rounded-lg p-3 my-2 hover:border-blue-300 transition-colors min-w-48"
  >
    <div class="flex flex-col md:flex-row justify-between gap-2">
      <div class="space-x-2">
        <span
          :class="getTypeClass(change.change_type)"
          class="inline-block px-2 py-1 text-xs font-medium rounded-full mb-2"
        >
          {{ getChangeTypeLabel(change.change_type) }}
        </span>
        <span class="text-xs text-gray-500 italic"> {{ formatTime(change.time) }} </span>
        <span class="text-xs text-gray-500 italic">
          {{ t('recentChanges.by') }}
          <RouterLink
            v-if="change.change_type !== 'message'"
            :to="`/user/${change.username}`"
            class="text-blue-600 hover:underline"
          >
            {{ change.username }}
          </RouterLink>
          <div v-else class="inline">{{ change.username }}</div>
        </span>
        <div class="text-sm">
          <RouterLink
            :to="getChangeLink(change)"
            class="font-medium text-blue-600 hover:text-blue-800 hover:underline flex items-center"
          >
            <template v-if="change.change_type === 'comment' && !change.word">
              <MessageCircle class="h-4 w-4 mr-1" />
              <span>{{ t('recentChanges.commentFallback') }}</span>
            </template>
            <span v-else>{{ change.word }}</span>
          </RouterLink>
          <span
            v-if="change.language_name && change.change_type === 'definition'"
            class="italic text-gray-600"
          >
            {{ t('recentChanges.in', { language: changeLanguage }) }}
          </span>
          <div
            v-if="change.change_type === 'definition' && visibleDiffChanges.length"
            class="mt-3 space-y-3 border-l-4 border-blue-200 pl-4"
          >
            <template v-for="diffChange in visibleDiffChanges" :key="diffChange.field">
            <div
              v-if="diffFieldHasVisibleContent(diffChange)"
              class="space-y-1"
            >
              <div class="text-xs font-medium text-gray-500">
                {{ formatFieldName(diffChange.field) }}:
              </div>
              <!-- Image diff: show the image safely -->
              <template v-if="diffChange.field === 'image'">
                <div
                  v-if="diffChange.change_type === 'added' && diffChange.image_url"
                  class="bg-green-50 p-2 rounded text-sm"
                >
                  <img
                    :src="diffChange.image_url"
                    :alt="t('components.recentChangeItem.imageAdded')"
                    class="max-h-40 max-w-full object-contain rounded"
                    loading="lazy"
                  />
                </div>
                <div
                  v-else-if="diffChange.change_type === 'removed'"
                  class="bg-red-50 text-red-700 p-2 rounded text-sm"
                >
                  {{ t('components.recentChangeItem.imageRemoved') }}
                </div>
              </template>
              <template v-else-if="isPlainTextField(diffChange.field)">
                <template v-if="diffChange.change_type === 'modified'">
                  <div
                    v-if="hasDiffContent(diffChange.old_value)"
                    class="bg-red-50 p-2 rounded text-sm mb-1 whitespace-pre-wrap"
                  >
                    {{ diffChange.old_value }}
                  </div>

                  <div
                    v-if="hasDiffContent(diffChange.new_value)"
                    class="bg-green-50 p-2 rounded text-sm whitespace-pre-wrap"
                  >
                    {{ diffChange.new_value }}
                  </div>
                </template>
                <template v-else-if="hasDiffContent(diffChange.new_value || diffChange.old_value)">
                  <div
                    :class="{
                      'bg-green-50 text-green-800': diffChange.change_type === 'added',
                      'bg-red-50 text-red-800': diffChange.change_type === 'removed',
                    }"
                    class="p-2 rounded text-sm whitespace-pre-wrap"
                  >
                    {{ diffChange.new_value || diffChange.old_value }}
                  </div>
                </template>
              </template>
              <template v-else>
                <template v-if="diffChange.change_type === 'modified'">
                  <div
                    v-if="hasDiffContent(diffChange.old_value)"
                    class="bg-red-50 p-2 rounded text-sm mb-1"
                  >
                    <LazyMathJax :content="diffChange.old_value" :enable-markdown="true" />
                  </div>

                  <div
                    v-if="hasDiffContent(diffChange.new_value)"
                    class="bg-green-50 p-2 rounded text-sm"
                  >
                    <LazyMathJax :content="diffChange.new_value" :enable-markdown="true" />
                  </div>
                </template>
                <template v-else-if="hasDiffContent(diffChange.new_value || diffChange.old_value)">
                  <div
                    :class="{
                      'bg-green-50 text-green-800': diffChange.change_type === 'added',
                      'bg-red-50 text-red-800': diffChange.change_type === 'removed',
                    }"
                    class="p-2 rounded text-sm"
                  >
                    <LazyMathJax
                      :content="diffChange.new_value || diffChange.old_value"
                      :enable-markdown="true"
                    />
                  </div>
                </template>
              </template>
            </div>
            </template>
          </div>

          <div
            v-else-if="change.change_type === 'message' && change.content"
            class="prose prose-sm max-w-none text-gray-700 mb-3"
          >
            <LazyMathJax :content="change.content" :enable-markdown="true" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { MessageCircle } from '@lucide/vue'
import { RouterLink, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { getTypeClass } from '@/utils/wordTypeUtils'
import { useDateFormat } from '@/composables/useDateFormat'

import CommentItem from '@/components/CommentItem.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'

const { t, locale } = useI18n()
const { formatTime } = useDateFormat()
const router = useRouter()

const props = defineProps({
  change: {
    type: Object,
    required: true,
  },
})

const changeLanguage = computed(() => {
  const c = props.change
  if (locale.value === 'jbo') {
    return c.language_lojban_name || c.language_name
  }
  return c.language_english_name || c.language_name
})

// Map recent-change (comment type) to the shape CommentItem expects (reactions, save, etc.)
const mappedComment = computed(() => {
  const c = props.change
  if (c.change_type !== 'comment') return null
  const rawContent = Array.isArray(c.content)
    ? c.content
    : [{ type: 'text', data: typeof c.content === 'string' ? c.content : '' }]
  // CommentItem derives subject from content parts with type 'header'; inject one if we have change.word
  const content =
    c.word && !rawContent.some((p) => p.type === 'header')
      ? [{ type: 'header', data: c.word }, ...rawContent]
      : rawContent
  return {
    comment_id: c.comment_id,
    thread_id: c.thread_id,
    definition_id: c.definition_id ?? null,
    valsi_id: c.valsi_id ?? null,
    username: c.username ?? null,
    time: c.time,
    content,
    subject: c.word ?? '', // also reflected in header part in content when injected
    reactions: c.reactions ?? [],
    is_bookmarked: c.is_bookmarked ?? false,
    comment_num: c.comment_num ?? 0,
    parent_id: c.parent_id ?? null,
    valsi_word: c.valsi_word ?? null,
    total_replies: 0,
    total_reactions: (c.reactions ?? []).reduce((sum, r) => sum + (r.count ?? 0), 0),
    definition: c.definition ?? null,
    parent_content: null,
  }
})

const getChangeLink = (change) => {
  if (change.change_type === 'comment') {
    return `/comments?thread_id=${change.thread_id}&scroll_to=${change.comment_id}&valsi_id=${change.valsi_id}&definition_id=${change.definition_id}`
  } else if (change.change_type === 'message') {
    return `/message/${change.comment_id}`
  }
  return `/valsi/${change.word.replace(/ /g, '_')}?highlight_definition_id=${change.definition_id}`
}

const handleReply = (commentId: number) => {
  const c = props.change
  if (c.change_type !== 'comment') return
  router.push(
    `/comments?thread_id=${c.thread_id}&scroll_to=${commentId}&valsi_id=${c.valsi_id || 0}&definition_id=${c.definition_id || 0}&reply_to=${commentId}`
  )
}

const getChangeTypeLabel = (changeType) => t(`recentChanges.changeTypes.${changeType}`)

const formatFieldName = (field) => {
  return field
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .map((word) =>
      t(`components.recentChangeItem.fields.${word}`, word.charAt(0).toUpperCase() + word.slice(1))
    )
    .join(' ')
}

const hasDiffContent = (value) => {
  if (value == null) return false
  if (typeof value === 'string') return value.trim() !== ''
  if (Array.isArray(value)) return value.length > 0
  return true
}

const diffFieldHasVisibleContent = (diffChange) => {
  if (!diffChange) return false
  if (diffChange.field === 'image') {
    return diffChange.change_type === 'added'
      ? Boolean(diffChange.image_url)
      : diffChange.change_type === 'removed'
  }
  return hasDiffContent(diffChange.old_value) || hasDiffContent(diffChange.new_value)
}

const visibleDiffChanges = computed(() => {
  const changes = props.change?.diff?.changes
  if (!Array.isArray(changes)) return []
  return changes.filter(diffFieldHasVisibleContent)
})

// Fields whose values are plain text — render as text only (no markdown/linkification)
const isPlainTextField = (field) =>
  field === 'gloss_keywords' || field === 'place_keywords' || field === 'rafsi'
</script>

<style scoped>
.comment-item {
  transform-style: preserve-3d;
}

.comment-item img.profile-image {
  backface-visibility: hidden;
  transform: translateZ(0);
}

.comment-item:hover img.profile-image {
  border-color: rgb(147, 197, 253);
}
</style>
