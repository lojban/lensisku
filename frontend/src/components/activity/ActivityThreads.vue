<template>
  <div class="space-y-4">
    <div v-if="threads.length" class="space-y-4">
      <div
        v-for="thread in threads"
        :key="
          thread.source === 'comment' ? thread.thread_id : 'mail-' + (thread.cleaned_subject || '')
        "
        class="space-y-2 bg-white p-4 rounded-lg border border-gray-200 hover:border-blue-200 transition-colors cursor-pointer"
        @click="
          thread.source === 'comment'
            ? router.push(
                `/comments?thread_id=${thread.thread_id}&scroll_to=${thread.comment_id}&valsi_id=${thread.valsi_id || ''}&definition_id=${thread.definition_id || ''}`
              )
            : goToMailThread(thread.cleaned_subject || thread.subject)
        "
      >
        <!-- Comment thread -->
        <template v-if="thread.source === 'comment'">
          <div class="flex flex-wrap gap-2 items-center mb-2">
            <h3 class="font-medium text-gray-800">
              <template
                v-if="
                  thread.first_comment_content?.some(
                    (p) => p.type === 'text' && p.data?.startsWith('![')
                  )
                "
              >
                <Image class="w-4 h-4 inline-block mr-1" />
                {{ t('activityThreads.imageComment') }}
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
              <span>·</span>
              <span>{{ formatTime(thread.time || thread.last_activity_time) }}</span>
            </div>
            <div
              v-if="thread.simple_content"
              class="border-l-2 border-gray-300 pl-2 text-gray-500 [&_img]:max-h-48 [&_img]:object-contain"
            >
              <LazyMathJax :content="thread.simple_content" :enable-markdown="true" />
            </div>
            <div v-else class="flex items-center gap-2 text-gray-400 pt-1">
              <MessageSquareMore class="w-4 h-4" />
              <span class="text-sm">{{ t('activityThreads.noContent') }}</span>
            </div>
          </div>
        </template>
        <!-- Mail thread -->
        <template v-else>
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
          <div
            v-if="thread.content_preview"
            class="text-sm text-gray-600 border-l-2 border-gray-300 pl-2 text-gray-500 [&_img]:max-h-48 [&_img]:object-contain"
          >
            <LazyMathJax :content="thread.content_preview" :enable-markdown="true" />
          </div>
          <div v-else class="flex items-center gap-2 text-gray-400 pt-1 text-sm">
            <MessageSquareMore class="w-4 h-4" />
            <span>{{ t('activityThreads.noContent') }}</span>
          </div>
        </template>
      </div>
    </div>
    <div v-else class="text-center py-8 bg-gray-50 rounded-lg border border-gray-200">
      <p class="text-sm text-gray-600">
        {{ t('activityThreads.noWavesFound') }}
      </p>
    </div>
  </div>
</template>

<script setup>
import { MessageSquareMore, Image } from 'lucide-vue-next'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'

import LazyMathJax from '@/components/LazyMathJax.vue'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()

function goToMailThread(subject) {
  const locale = route.path.split('/')[1] || 'en'
  router.push({ name: `ThreadView-${locale}`, params: { subject: subject || '' } })
}

defineProps({
  threads: {
    type: Array,
    required: true,
  },
  formatDateForThread: {
    type: Function,
    required: true,
  },
  formatTime: {
    type: Function,
    required: true,
  },
})
</script>
