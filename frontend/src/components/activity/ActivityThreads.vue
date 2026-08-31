<template>
  <div class="space-y-4">
    <div v-if="threads.length" class="space-y-4">
      <div
        v-for="thread in threads"
        :key="
          thread.source === 'comment'
            ? thread.thread_id
            : thread.source === 'wiki'
              ? 'wiki-' + (thread.summary?.page_id || '')
              : 'mail-' + (thread.cleaned_subject || '')
        "
        class="surface-activity-row"
        @click="goToThread(thread)"
      >
        <!-- Comment thread -->
        <template v-if="thread.source === 'comment'">
          <div class="flex flex-wrap gap-2 items-center mb-2">
            <SourceTypeBadge
              v-if="thread.import_source === 'jbotcan'"
              type="jbotcan"
              label="jbotcan"
            />
            <SourceTypeBadge
              v-else-if="thread.import_source === 'freeforums'"
              type="freeforums"
            />
            <h3 class="font-medium text-gray-800">
              <template
                v-if="
                  thread.first_comment_content?.some(
                    (p) => p.type === 'text' && p.data?.startsWith('![')
                  )
                "
              >
                <Image class="w-4 h-4 inline-block mr-1" /> {{ t('activityThreads.imageComment') }}
              </template>
              <template v-else>
                <LazyMathJax
                  :content="
                    thread.first_comment_subject ||
                    thread.first_comment_content?.find((p) => p.type === 'text')?.data ||
                    '-'
                  "
                  :enable-markdown="true"
                  class="inline"
                />
              </template>
              <span class="text-sm font-normal text-gray-400 italic">
                · {{ t('activityThreads.by') }} {{ thread.username }}</span
              >
            </h3>
          </div>

          <div
            class="flex items-center text-sm text-blue-500 hover:text-blue-700 hover:underline pb-2 border-b"
          >
            <span>{{ thread.total_replies }} {{ t('activityThreads.comments') }}</span>
          </div>

          <div class="text-sm text-gray-600 space-y-2">
            <div class="flex items-center gap-2 text-xs text-gray-400 italic">
              <span>{{ t('activityThreads.by') }} {{ thread.last_comment_username }}</span>
              <span>·</span>
              <span>{{ formatDateForThread(thread.time || thread.last_activity_time) }}</span>
              <span>·</span> <span>{{ formatTime(thread.time || thread.last_activity_time) }}</span>
            </div>

            <div v-if="thread.simple_content" class="activity-quote-snippet">
              <LazyMathJax :content="thread.simple_content" :enable-markdown="true" />
            </div>

            <div v-else class="flex items-center gap-2 text-gray-400 pt-1">
              <MessageSquareMore class="w-4 h-4" />
              <span class="text-sm">{{ t('activityThreads.noContent') }}</span>
            </div>
          </div>

          <div v-if="auth.state.isLoggedIn" class="flex justify-end mt-2">
            <Button
              variant="reply"
              class="inline-flex items-center gap-2"
              @click.stop="replyToThread(thread)"
            >
              <Reply class="w-4 h-4" />
              <span>{{ t('components.commentItem.reply') }}</span>
            </Button>
          </div>
        </template>
        <!-- Mail thread -->
        <template v-else-if="thread.source === 'mail'">
          <div class="flex flex-wrap gap-2 items-center mb-2">
            <h3 class="font-medium text-gray-800">
              <LazyMathJax
                :content="thread.subject || thread.cleaned_subject || '-'"
                :enable-markdown="true"
                class="inline"
              />
            </h3>
            <span class="text-sm font-normal text-gray-400 italic">
              · {{ t('activityThreads.by') }} {{ thread.from_address }}</span
            >
          </div>

          <div
            class="flex items-center text-sm text-blue-500 hover:text-blue-700 hover:underline pb-2 border-b"
          >
            <span>{{ thread.message_count }} {{ t('activityThreads.messages') }}</span>
          </div>

          <div v-if="thread.content_preview" class="activity-quote-snippet">
            <LazyMathJax :content="thread.content_preview" :enable-markdown="true" />
          </div>

          <div v-else class="flex items-center gap-2 text-gray-400 pt-1 text-sm">
            <MessageSquareMore class="w-4 h-4" /> <span>{{ t('activityThreads.noContent') }}</span>
          </div>
        </template>
        <!-- Wiki thread -->
        <template v-else-if="thread.source === 'wiki'">
          <div class="flex flex-wrap gap-2 items-center mb-2">
            <SourceTypeBadge type="wiki" />
            <h3 class="font-medium text-gray-800">
              {{ thread.summary?.title || '-' }}
            </h3>
          </div>

          <div class="flex items-center text-xs text-gray-400 italic pb-2 border-b">
            <span>{{ formatDateForThread(thread.last_activity_time) }}</span>
            <span>&nbsp;·&nbsp;</span>
            <span>{{ formatTime(thread.last_activity_time) }}</span>
          </div>

          <div v-if="thread.summary?.content_preview" class="activity-quote-snippet">
            <LazyMathJax :content="thread.summary.content_preview" :enable-markdown="true" />
          </div>

          <div v-else class="flex items-center gap-2 text-gray-400 pt-1 text-sm">
            <MessageSquareMore class="w-4 h-4" /> <span>{{ t('activityThreads.noContent') }}</span>
          </div>
        </template>
      </div>
    </div>

    <div v-else class="text-center py-8 bg-gray-50 rounded-lg border border-gray-200">
      <p class="text-sm text-gray-600">{{ t('activityThreads.noWavesFound') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { MessageSquareMore, Image, Reply } from '@lucide/vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import type { PropType } from 'vue'

import LazyMathJax from '@/components/LazyMathJax.vue'
import SourceTypeBadge from '@/components/SourceTypeBadge.vue'
import { useAuth } from '@/composables/useAuth'

type ContentPart = { type: string; data?: string }

type WikiSummary = {
  page_id: number
  namespace: number
  title: string
  last_edited: string
  content_preview: string
  article_url: string
}

/** Row from activity / waves API (comment thread vs mail thread vs wiki). */
type ActivityThreadRow = {
  source: 'comment' | 'mail' | 'wiki'
  import_source?: string | null
  thread_id?: number
  comment_id?: number
  valsi_id?: number | string
  definition_id?: number | string
  cleaned_subject?: string
  subject?: string
  first_comment_content?: ContentPart[]
  first_comment_subject?: string
  username?: string
  total_replies?: number
  last_comment_username?: string
  time?: number
  last_activity_time?: number
  simple_content?: string
  from_address?: string
  message_count?: number
  content_preview?: string
  summary?: WikiSummary
}

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const auth = useAuth()

function goToMailThread(subject: string) {
  const locale = route.path.split('/')[1] || 'en'
  router.push({ name: `ThreadView-${locale}`, params: { subject: subject || '' } })
}

function goToThread(thread: ActivityThreadRow) {
  if (thread.source === 'comment') {
    router.push(
      `/comments?thread_id=${thread.thread_id}&scroll_to=${thread.comment_id}&valsi_id=${thread.valsi_id || ''}&definition_id=${thread.definition_id || ''}`
    )
  } else if (thread.source === 'wiki' && thread.summary?.article_url) {
    router.push(thread.summary.article_url)
  } else {
    goToMailThread(thread.cleaned_subject || thread.subject)
  }
}

function replyToThread(thread: ActivityThreadRow) {
  if (thread.source !== 'comment') return
  router.push(
    `/comments?thread_id=${thread.thread_id}&scroll_to=${thread.comment_id}&valsi_id=${thread.valsi_id || ''}&definition_id=${thread.definition_id || ''}&reply_to=${thread.comment_id}`
  )
}

defineProps({
  threads: {
    type: Array as PropType<ActivityThreadRow[]>,
    required: true,
  },
  formatDateForThread: {
    type: Function as PropType<(ts: number) => string>,
    required: true,
  },
  formatTime: {
    type: Function as PropType<(ts: number) => string>,
    required: true,
  },
})
</script>
