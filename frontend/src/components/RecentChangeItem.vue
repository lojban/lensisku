<template>
  <!-- Comment-type: reuse CommentItem so reactions and save button are shown -->
  <CommentItem
    v-if="change.change_type === 'comment'"
    :comment="mappedComment"
    :reply-enabled="false"
    :show-context="true"
    :valsi-id="change.valsi_id || 0"
    :definition-id="change.definition_id || 0"
  />
  <!-- Other change types: existing layout -->
  <div
    v-else
    class="comment-item bg-white border rounded-lg p-3 my-2 hover:border-blue-300 transition-colors min-w-48"
  >
    <div class="flex flex-col md:flex-row justify-between gap-2">
      <div class="space-x-2">
        <span :class="getTypeClass(change.change_type)"
          class="inline-block px-2 py-1 text-xs font-medium rounded-full mb-2">
          {{ change.change_type }}
        </span>
        <span class="text-xs text-gray-500">
          {{ formatTime(change.time) }}
        </span>
        <span class="text-xs text-gray-600 italic">
          {{ t('recentChanges.by') }}
          <RouterLink v-if="change.change_type !== 'message'" :to="`/user/${change.username}`"
            class="text-blue-600 hover:underline">
            {{ change.username }}
          </RouterLink>
          <div v-else class="inline">
            {{ change.username }}
          </div>
        </span>
        <div class="text-sm">
          <RouterLink :to="getChangeLink(change)"
            class="font-medium text-blue-600 hover:text-blue-800 hover:underline flex items-center">
            <template v-if="change.change_type === 'comment' && !change.word">
              <MessageCircle class="h-4 w-4 mr-1" />
              <span>{{ t('recentChanges.commentFallback') }}</span>
            </template>
            <span v-else>{{ change.word }}</span>
          </RouterLink>
          <span v-if="change.language_name && change.change_type === 'definition'" class="italic text-gray-600">
            {{ t('recentChanges.in') }} {{ change.language_name }}
          </span>
          <div v-if="change.change_type === 'definition' && change.diff"
            class="mt-3 space-y-3 border-l-4 border-blue-200 pl-4">
            <div v-for="diffChange in change.diff.changes" :key="diffChange.field" class="space-y-1">
              <div class="text-xs font-medium text-gray-500">
                {{ formatFieldName(diffChange.field) }}:
              </div>
              <template v-if="isPlainTextField(diffChange.field)">
                <template v-if="diffChange.change_type === 'modified'">
                  <div class="bg-red-50 p-2 rounded text-sm mb-1 whitespace-pre-wrap">{{ diffChange.old_value || '' }}</div>
                  <div class="bg-green-50 p-2 rounded text-sm whitespace-pre-wrap">{{ diffChange.new_value || '' }}</div>
                </template>
                <template v-else>
                  <div :class="{
                    'bg-green-50 text-green-800': diffChange.change_type === 'added',
                    'bg-red-50 text-red-800': diffChange.change_type === 'removed',
                  }" class="p-2 rounded text-sm whitespace-pre-wrap">{{ diffChange.new_value || diffChange.old_value || '' }}</div>
                </template>
              </template>
              <template v-else>
                <template v-if="diffChange.change_type === 'modified'">
                  <div class="bg-red-50 p-2 rounded text-sm mb-1">
                    <LazyMathJax :content="diffChange.old_value" :enable-markdown="true" />
                  </div>
                  <div class="bg-green-50 p-2 rounded text-sm">
                    <LazyMathJax :content="diffChange.new_value" :enable-markdown="true" />
                  </div>
                </template>
                <template v-else>
                  <div :class="{
                    'bg-green-50 text-green-800': diffChange.change_type === 'added',
                    'bg-red-50 text-red-800': diffChange.change_type === 'removed',
                  }" class="p-2 rounded text-sm">
                    <LazyMathJax :content="diffChange.new_value || diffChange.old_value" :enable-markdown="true" />
                  </div>
                </template>
              </template>
            </div>
          </div>
          <div v-else-if="change.change_type === 'message' && change.content"
            class="prose prose-sm max-w-none text-gray-700 mb-3">
            <LazyMathJax :content="change.content" :enable-markdown="true" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { MessageCircle } from 'lucide-vue-next';
import { RouterLink } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { getTypeClass } from '@/utils/wordTypeUtils';

import CommentItem from '@/components/CommentItem.vue';
import LazyMathJax from '@/components/LazyMathJax.vue';

const { t, locale } = useI18n();

const props = defineProps({
  change: {
    type: Object,
    required: true
  }
});

// Map recent-change (comment type) to the shape CommentItem expects (reactions, save, etc.)
const mappedComment = computed(() => {
  const c = props.change;
  if (c.change_type !== 'comment') return null;
  const rawContent = Array.isArray(c.content)
    ? c.content
    : [{ type: 'text', data: typeof c.content === 'string' ? c.content : '' }];
  // CommentItem derives subject from content parts with type 'header'; inject one if we have change.word
  const content =
    c.word && !rawContent.some((p) => p.type === 'header')
      ? [{ type: 'header', data: c.word }, ...rawContent]
      : rawContent;
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
    definition: null,
    parent_content: null
  };
});

const getChangeLink = (change) => {
  if (change.change_type === 'comment') {
    return `/comments?thread_id=${change.thread_id}&scroll_to=${change.comment_id}&valsi_id=${change.valsi_id}&definition_id=${change.definition_id}`
  } else if (change.change_type === 'message') {
    return `/message/${change.comment_id}`
  }
  return `/valsi/${change.word.replace(/ /g, '_')}?highlight_definition_id=${change.definition_id}`
}

const formatTime = (timestamp) =>
  new Date(timestamp * 1000).toLocaleTimeString(locale.value, {
    hour: '2-digit',
    minute: '2-digit',
  })

// Remove the local getTypeClass implementation

const formatFieldName = (field) => {
  return field
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .map(word => t(`components.recentChangeItem.fields.${word}`, word.charAt(0).toUpperCase() + word.slice(1))) // Use t() with fallback
    .join(' ');
}

// Fields whose values are plain keyword/gloss lists — render as text only (no markdown/linkification)
const isPlainTextField = (field) =>
  field === 'gloss_keywords' || field === 'place_keywords'
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