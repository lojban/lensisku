<template>
  <div class="flex flex-col h-full max-h-[calc(100vh-3rem)]">
    <div class="mb-4 flex items-start justify-between gap-3">
      <div>
        <h1 class="text-2xl font-bold text-gray-800">
          {{ $t('assistantChat.title') }}
        </h1>
        <p class="mt-1 text-sm text-gray-600">
          {{ $t('assistantChat.subtitle') }}
        </p>
      </div>
      <button
        v-if="messages.length > 0"
        type="button"
        class="btn-aqua-zinc flex items-center gap-1.5 px-2.5 py-1.5 text-sm shrink-0"
        :aria-label="$t('assistantChat.clearHistory')"
        @click="clearHistory"
      >
        <Trash2 class="w-4 h-4" />
        {{ $t('assistantChat.clearHistory') }}
      </button>
    </div>

    <div
      ref="scrollContainer"
      class="flex-1 overflow-y-auto rounded-lg border border-gray-200 bg-white p-4 space-y-4"
    >
      <div
        v-if="messages.length === 0"
        class="text-gray-500 text-sm"
      >
        {{ $t('assistantChat.emptyState') }}
      </div>

      <div
        v-for="(msg, index) in messages"
        :key="index"
        class="flex"
        :class="msg.role === 'user' ? 'justify-end' : 'justify-start'"
      >
        <div
          class="max-w-[80%] rounded-lg px-3 py-2 text-sm break-words"
          :class="msg.role === 'user'
            ? 'bg-blue-600 text-white whitespace-pre-wrap'
            : 'bg-gray-100 text-gray-900 assistant-markdown'"
        >
          <span v-if="msg.role === 'assistant'" class="block text-[11px] font-semibold text-gray-500 mb-1">
            {{ $t('assistantChat.assistantLabel') }}
          </span>
          <span v-else-if="msg.role === 'user'" class="block text-[11px] font-semibold text-blue-100 mb-1">
            {{ $t('assistantChat.userLabel') }}
          </span>
          <!-- Thought process: tool calls and results in italic gray (updates in real time over SSE) -->
          <div
            v-if="msg.role === 'assistant' && msg.steps && msg.steps.length > 0"
            class="thought-process mb-2 space-y-1"
          >
            <div
              v-for="(step, stepIdx) in msg.steps"
              :key="stepIdx"
              class="text-gray-500 text-xs italic"
            >
              <span>{{ step.action }}</span>
              <span class="block mt-0.5">{{ step.result }}</span>
            </div>
          </div>
          <!-- Thinking dots inside assistant bubble while streaming and no reply yet -->
          <div
            v-if="msg.role === 'assistant' && isLoading && index === messages.length - 1 && !msg.content"
            class="thinking-dots flex items-center gap-1 min-h-[1.25rem] mb-1"
            role="status"
            :aria-label="$t('assistantChat.thinking')"
          >
            <span class="thinking-dot" />
            <span class="thinking-dot" />
            <span class="thinking-dot" />
          </div>
          <LazyMathJax
            v-if="msg.role === 'assistant' && msg.content"
            :content="msg.content"
            :enable-markdown="true"
            :lang-id="locale"
          />
          <span v-else-if="msg.role === 'user'">{{ msg.content }}</span>
        </div>
      </div>

      <!-- Thinking indicator when no assistant message yet (e.g. before stream starts) -->
      <div
        v-if="isLoading && (messages.length === 0 || messages[messages.length - 1]?.role !== 'assistant')"
        class="flex justify-start"
        role="status"
        :aria-label="$t('assistantChat.thinking')"
      >
        <div class="max-w-[80%] rounded-lg px-3 py-2.5 bg-gray-100 text-gray-600 text-sm">
          <span class="block text-[11px] font-semibold text-gray-500 mb-1.5">
            {{ $t('assistantChat.assistantLabel') }}
          </span>
          <div class="thinking-dots flex items-center gap-1 min-h-[1.25rem]">
            <span class="thinking-dot" />
            <span class="thinking-dot" />
            <span class="thinking-dot" />
          </div>
        </div>
      </div>
    </div>

    <form
      class="mt-4 flex flex-col gap-2"
      @submit.prevent="handleSend"
    >
      <textarea
        v-model="input"
        class="textarea-field min-h-[80px] max-h-40 resize-y"
        :placeholder="$t('assistantChat.placeholder')"
        :disabled="isLoading"
      />

      <div class="flex items-center justify-between gap-3">
        <div v-if="error" class="flex items-center gap-2 min-w-0">
          <p class="text-xs text-red-600 truncate">
            {{ error }}
          </p>
          <button
            type="button"
            class="shrink-0 p-1 rounded text-red-600 hover:bg-red-50 focus:outline-none focus:ring-2 focus:ring-red-300"
            :aria-label="$t('assistantChat.retry')"
            :title="$t('assistantChat.retry')"
            @click="retryLast"
          >
            <RotateCw class="w-4 h-4" />
          </button>
        </div>
        <span v-else class="text-xs text-gray-500">
          {{ $t('assistantChat.hint') }}
        </span>

        <button
          type="submit"
          class="btn-aqua-emerald px-4 py-1.5 text-sm disabled:opacity-60 disabled:cursor-not-allowed"
          :disabled="isLoading || !input.trim()"
        >
          <span v-if="isLoading">
            {{ $t('assistantChat.sending') }}
          </span>
          <span v-else>
            {{ $t('assistantChat.send') }}
          </span>
        </button>
      </div>
    </form>
  </div>
</template>

<script setup>
import { ref, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { RotateCw, Trash2 } from 'lucide-vue-next'

import LazyMathJax from '@/components/LazyMathJax.vue'
import { getApiBaseUrl, getAuthHeaders } from '../api'

const { locale, t } = useI18n()
const messages = ref([])
const input = ref('')
const isLoading = ref(false)
const error = ref('')
const scrollContainer = ref(null)

const scrollToBottom = () => {
  nextTick(() => {
    if (scrollContainer.value) {
      scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight
    }
  })
}

watch(
  () => [messages.value.length, isLoading.value],
  () => scrollToBottom()
)

async function performRequest(msgList) {
  const payload = {
    messages: msgList.map((m) => ({
      role: m.role,
      content: m.content,
    })),
    locale: locale.value,
  }

  // Placeholder assistant message; steps and content will be updated from the stream
  messages.value.push({
    role: 'assistant',
    content: '',
    steps: [],
  })
  const assistantIndex = messages.value.length - 1

  const headers = {
    'Content-Type': 'application/json',
    ...getAuthHeaders(),
  }

  const response = await fetch(`${getApiBaseUrl()}/assistant/chat/stream`, {
    method: 'POST',
    headers,
    body: JSON.stringify(payload),
  })

  if (!response.ok) {
    const errText = await response.text()
    throw new Error(errText || `HTTP ${response.status}`)
  }

  if (!response.body) {
    throw new Error('Response body is null')
  }

  const reader = response.body.pipeThrough(new TextDecoderStream()).getReader()
  let buffer = ''

  while (true) {
    const { value, done } = await reader.read()
    if (value) buffer += value
    const lines = buffer.split(/\n/)
    buffer = lines.pop() ?? '' // keep incomplete line in buffer
    for (const line of lines) {
      const data = line.replace(/^data:\s*/, '').trim()
      if (!data || !data.startsWith('{')) continue
      try {
        const event = JSON.parse(data)
        if (event.type === 'step') {
          messages.value[assistantIndex].steps.push({
            action: event.action ?? '',
            result: event.result ?? '',
          })
        } else if (event.type === 'done') {
          messages.value[assistantIndex].content = event.reply ?? ''
        } else if (event.type === 'error') {
          messages.value[assistantIndex].content = event.error
            ? `_${t('assistantChat.error')}: ${event.error}_`
            : t('assistantChat.error')
        }
      } catch (_) {
        // ignore non-JSON or malformed lines
      }
    }
    if (done) break
  }
}

const handleSend = async () => {
  const content = input.value.trim()
  if (!content || isLoading.value) return

  error.value = ''
  messages.value.push({
    role: 'user',
    content,
  })
  input.value = ''
  isLoading.value = true

  try {
    await performRequest(messages.value)
  } catch (e) {
    console.error(e)
    error.value = t('assistantChat.error')
  } finally {
    isLoading.value = false
  }
}

const retryLast = async () => {
  if (isLoading.value || !error.value) return
  error.value = ''
  isLoading.value = true
  try {
    await performRequest(messages.value)
  } catch (e) {
    console.error(e)
    error.value = t('assistantChat.error')
  } finally {
    isLoading.value = false
  }
}

const clearHistory = () => {
  messages.value = []
  error.value = ''
}
</script>

<style scoped>
.assistant-markdown :deep(.mathjax-content) {
  display: block;
}
.assistant-markdown :deep(a) {
  @apply text-blue-600 hover:text-blue-800 hover:underline;
}
.assistant-markdown :deep(ul),
.assistant-markdown :deep(ol) {
  @apply my-1 pl-4;
}
.assistant-markdown :deep(p + p) {
  @apply mt-2;
}

/* Thinking indicator: subtle bouncing dots */
.thinking-dots {
  --dot-size: 6px;
}
.thinking-dot {
  width: var(--dot-size);
  height: var(--dot-size);
  border-radius: 50%;
  background: currentColor;
  opacity: 0.4;
  animation: thinking-bounce 1.4s ease-in-out infinite both;
}
.thinking-dot:nth-child(1) {
  animation-delay: 0s;
}
.thinking-dot:nth-child(2) {
  animation-delay: 0.2s;
}
.thinking-dot:nth-child(3) {
  animation-delay: 0.4s;
}
@keyframes thinking-bounce {
  0%,
  60%,
  100% {
    transform: translateY(0);
    opacity: 0.4;
  }
  30% {
    transform: translateY(-4px);
    opacity: 1;
  }
}
</style>
