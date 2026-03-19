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
        class="flex flex-col gap-3"
        :class="msg.role === 'user' ? 'items-end' : 'items-start'"
      >
        <!-- User message: single bubble -->
        <div
          v-if="msg.role === 'user'"
          class="max-w-[80%] rounded-lg px-3 py-2 text-sm break-words bg-blue-600 text-white whitespace-pre-wrap"
        >
          <span class="block text-[11px] font-semibold text-blue-100 mb-1">
            {{ $t('assistantChat.userLabel') }}
          </span>
          <span>{{ msg.content }}</span>
        </div>

        <!-- Assistant: one bubble per reply (multi-model) or single bubble (legacy) -->
        <template v-if="msg.role === 'assistant'">
          <div
            v-for="(reply, replyIdx) in assistantReplies(msg)"
            :key="replyIdx"
            class="max-w-[80%] rounded-lg px-3 py-2 text-sm break-words bg-gray-100 text-gray-900 assistant-markdown"
          >
            <span class="block text-[11px] font-semibold text-gray-500 mb-1">
              {{ reply.modelName || (reply.model ? formatModelLabel(reply.model) : '') || $t('assistantChat.assistantLabel') }}
            </span>
            <!-- Thought process: steps with optional folded tool output -->
            <div
              v-if="reply.steps && reply.steps.length > 0"
              class="thought-process mb-2 space-y-2"
            >
              <AssistantThoughtStep
                v-for="(step, stepIdx) in reply.steps"
                :key="stepIdx"
                :step="step"
                :show-raw-output="isStepOutputVisible(stepKey(index, replyIdx, stepIdx))"
                @toggle-raw="toggleStepOutput(stepKey(index, replyIdx, stepIdx))"
              />
            </div>
            <!-- Thinking dots while streaming and no reply yet for this model -->
            <div
              v-if="isLoading && index === messages.length - 1 && !reply.content"
              class="thinking-dots flex items-center gap-1 min-h-[1.25rem] mb-1"
              role="status"
              :aria-label="$t('assistantChat.thinking')"
            >
              <span class="thinking-dot" />
              <span class="thinking-dot" />
              <span class="thinking-dot" />
            </div>
            <LazyMathJax
              v-if="reply.content"
              :content="reply.content"
              :enable-markdown="true"
              :lang-id="locale"
            />
          </div>
          <!-- Legacy single bubble when no replies array yet -->
          <div
            v-if="assistantReplies(msg).length === 0"
            class="max-w-[80%] rounded-lg px-3 py-2 text-sm break-words bg-gray-100 text-gray-900 assistant-markdown"
          >
            <span class="block text-[11px] font-semibold text-gray-500 mb-1">
              {{ $t('assistantChat.assistantLabel') }}
            </span>
            <div
              v-if="msg.steps && msg.steps.length > 0"
              class="thought-process mb-2 space-y-2"
            >
              <AssistantThoughtStep
                v-for="(step, stepIdx) in msg.steps"
                :key="stepIdx"
                :step="step"
                :show-raw-output="isStepOutputVisible(stepKey('legacy', index, stepIdx))"
                @toggle-raw="toggleStepOutput(stepKey('legacy', index, stepIdx))"
              />
            </div>
            <div
              v-if="isLoading && index === messages.length - 1 && !msg.content"
              class="thinking-dots flex items-center gap-1 min-h-[1.25rem] mb-1"
              role="status"
              :aria-label="$t('assistantChat.thinking')"
            >
              <span class="thinking-dot" />
              <span class="thinking-dot" />
              <span class="thinking-dot" />
            </div>
            <LazyMathJax
              v-if="msg.content"
              :content="msg.content"
              :enable-markdown="true"
              :lang-id="locale"
            />
          </div>
        </template>
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
        @keydown.enter.exact.prevent="handleSend"
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

import AssistantThoughtStep from '@/components/AssistantThoughtStep.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import { useSeoHead } from '@/composables/useSeoHead'
import { getApiBaseUrl, getAuthHeaders } from '../api'

const { locale, t } = useI18n()
useSeoHead({ title: t('assistantChat.title') })
const messages = ref([])
const input = ref('')
const isLoading = ref(false)
const error = ref('')
const scrollContainer = ref(null)
/** Keys: step id string (e.g. "0-0-0"); value: true if output is expanded */
const stepShowOutput = ref({})

/** Returns array of reply objects for an assistant message (multi-model or legacy single). */
function assistantReplies(msg) {
  if (!msg || msg.role !== 'assistant') return []
  if (msg.replies && msg.replies.length > 0) return msg.replies
  if (msg.content || (msg.steps && msg.steps.length > 0)) {
    return [{ model: null, modelName: null, steps: msg.steps || [], content: msg.content || '' }]
  }
  return []
}

function toggleStepOutput(key) {
  stepShowOutput.value[key] = !stepShowOutput.value[key]
}

function isStepOutputVisible(key) {
  return !!stepShowOutput.value[key]
}

function stepKey(...parts) {
  return parts.join('-')
}

/** Short display name for OpenRouter model id (e.g. "provider/model-name" → "model-name"). */
function formatModelLabel(modelId) {
  if (!modelId) return ''
  const last = modelId.split('/').pop()
  return last || modelId
}

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

  // Placeholder assistant message; steps and content (or per-model replies) updated from the stream
  messages.value.push({
    role: 'assistant',
    content: '',
    steps: [],
    replies: [], // [{ model, steps, content }] when backend sends model in events
  })
  const assistantIndex = messages.value.length - 1

  function getOrCreateReply(modelId, modelName = null) {
    const msg = messages.value[assistantIndex]
    if (!msg.replies) msg.replies = []
    let r = msg.replies.find((x) => x.model === modelId)
    if (!r) {
      r = { model: modelId, modelName: modelName || null, steps: [], content: '' }
      msg.replies.push(r)
    } else if (modelName != null && modelName !== r.modelName) {
      r.modelName = modelName
    }
    return r
  }

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
        const modelId = event.model ?? null
        const modelName = event.model_name ?? null
        const msg = messages.value[assistantIndex]
        const steps = modelId ? getOrCreateReply(modelId, modelName).steps : msg.steps
        if (event.type === 'step_start') {
          if (modelId) getOrCreateReply(modelId, modelName)
          const idx =
            typeof event.index === 'number' ? event.index : steps.length
          while (steps.length < idx) {
            steps.push({
              action: '',
              result: '…',
              tool_output: undefined,
            })
          }
          if (steps.length === idx) {
            steps.push({
              action: event.action ?? '',
              result: '…',
              tool_output: undefined,
            })
          } else {
            const existing = steps[idx]
            steps[idx] = {
              action: event.action ?? existing?.action ?? '',
              result: '…',
              tool_output: existing?.tool_output,
            }
          }
        } else if (event.type === 'step') {
          if (modelId) getOrCreateReply(modelId, modelName)
          const idx =
            typeof event.index === 'number'
              ? event.index
              : Math.max(0, steps.length - 1)
          const stepPayload = {
            action: event.action ?? (steps[idx]?.action ?? ''),
            result: event.result ?? '',
            tool_output: event.tool_output ?? steps[idx]?.tool_output,
          }
          while (steps.length < idx) {
            steps.push({
              action: stepPayload.action,
              result: '…',
              tool_output: undefined,
            })
          }
          if (steps.length === idx) {
            steps.push(stepPayload)
          } else {
            steps[idx] = stepPayload
          }
        } else if (event.type === 'done') {
          if (modelId) {
            const reply = getOrCreateReply(modelId, modelName)
            reply.content = event.reply ?? ''
          } else {
            msg.content = event.reply ?? ''
          }
        } else if (event.type === 'error') {
          const errContent = event.error
            ? `_${t('assistantChat.error')}: ${event.error}_`
            : t('assistantChat.error') + (event.raw_response ? `\n\n**Debug:**\n\`\`\`\n${event.raw_response}\n\`\`\`` : '')
          if (modelId) {
            getOrCreateReply(modelId).content = errContent
          } else {
            msg.content = errContent
          }
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
