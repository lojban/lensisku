<template>
  <!--
    Full-height route: shell fills App.vue main (already viewport − header).
    Mobile uses height:100% of that slot — not 100dvh−header again — so dvh/svh
    mismatch cannot overflow and confuse mobile Firefox scroll position.
    One scroll region (messages); composer is a normal flex footer.
  -->
  <div class="assistant-root flex min-h-0 w-full min-w-0 flex-1 gap-0 overflow-hidden md:h-full md:gap-4">
    <!-- Mobile drawer backdrop -->
    <div
      v-if="!isDesktop && sidebarOpen"
      class="fixed inset-0 z-40 bg-black/40 backdrop-blur-[1px] md:hidden transition-opacity"
      aria-hidden="true"
      @click="sidebarOpen = false"
    />

    <!-- Chat history sidebar -->
    <aside
      :class="[
        'assistant-sidebar flex min-h-0 flex-shrink-0 flex-col border-r border-gray-200 bg-gradient-to-b from-slate-50 to-gray-50/90 z-50',
        isDesktop
          ? 'relative h-full w-[min(100%,18rem)] md:w-72'
          : 'fixed bottom-0 left-0 top-14 w-[min(18rem,88vw)] shadow-2xl transition-transform duration-200 ease-out sm:top-12 md:relative md:h-full md:translate-x-0 md:shadow-none',
        !isDesktop && !sidebarOpen ? '-translate-x-full pointer-events-none' : 'translate-x-0',
      ]"
      :aria-label="$t('assistantChat.searchChats')"
    >
      <!-- Row 1: close (mobile) + new chat -->
      <div class="flex items-center gap-2 border-b border-gray-200/60 bg-white/40 px-2 py-2">
        <button
          v-if="!isDesktop"
          type="button"
          class="shrink-0 rounded-lg p-1.5 text-gray-600 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-400/50"
          :aria-label="$t('assistantChat.closeChatHistory')"
          @click="sidebarOpen = false"
        >
          <X class="h-5 w-5" />
        </button>
        <button
          type="button"
          class="flex min-w-0 flex-1 items-center justify-center gap-2 rounded-lg border border-dashed border-gray-300 bg-white/80 py-2.5 text-sm font-medium text-gray-700 hover:border-blue-400 hover:bg-blue-50/60 hover:text-blue-800 focus:outline-none focus:ring-2 focus:ring-blue-400/40 transition-colors"
          @click="startNewChat"
        >
          <Plus class="h-4 w-4 shrink-0" />
          {{ $t('assistantChat.newChat') }}
        </button>
      </div>

      <!-- Row 2: search -->
      <div class="border-b border-gray-200/60 bg-white/40 px-2 py-2">
        <div class="relative">
          <Search
            class="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400"
            aria-hidden="true"
          />
          <input
            v-model="chatSearchQuery"
            type="search"
            autocomplete="off"
            class="w-full rounded-lg border border-gray-200 bg-white py-2 pl-9 pr-3 text-sm text-gray-900 placeholder:text-gray-400 focus:border-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400/30"
            :placeholder="$t('assistantChat.searchChatsPlaceholder')"
          >
        </div>
      </div>

      <div
        class="flex-1 overflow-y-auto overflow-x-hidden min-h-0 p-2 space-y-1"
        role="list"
      >
        <p
          v-if="filteredSessions.length === 0"
          class="px-2 py-6 text-center text-sm text-gray-500"
        >
          {{ $t('assistantChat.noChatsMatch') }}
        </p>
        <button
          v-for="session in filteredSessions"
          :key="session.id"
          type="button"
          role="listitem"
          class="group w-full text-left rounded-lg px-2.5 py-2.5 transition-colors focus:outline-none focus:ring-2 focus:ring-inset focus:ring-blue-400/50"
          :class="
            session.id === activeSessionId
              ? 'bg-blue-100/90 border border-blue-200/80 shadow-sm'
              : 'border border-transparent hover:bg-gray-100/90'
          "
          :aria-current="session.id === activeSessionId ? 'true' : undefined"
          :aria-label="
            session.id === activeSessionId
              ? $t('assistantChat.activeChat') + ': ' + session.title
              : session.title
          "
          @click="selectSession(session.id)"
        >
          <div class="flex items-start gap-2">
            <MessageSquare
              class="w-4 h-4 mt-0.5 shrink-0"
              :class="session.id === activeSessionId ? 'text-blue-600' : 'text-gray-400'"
              aria-hidden="true"
            />
            <div class="min-w-0 flex-1">
              <p class="text-sm font-medium text-gray-900 line-clamp-2 leading-snug">
                {{ session.title }}
              </p>
              <p class="mt-0.5 text-[11px] text-gray-500">
                {{ formatSessionTime(session.updatedAt) }}
              </p>
            </div>
            <button
              type="button"
              class="shrink-0 p-1 rounded text-gray-400 hover:text-red-600 hover:bg-red-50 focus:outline-none focus:ring-2 focus:ring-red-300 md:opacity-0 md:group-hover:opacity-100 md:group-focus-within:opacity-100"
              :aria-label="$t('assistantChat.deleteChat')"
              @click="deleteSession(session.id, $event)"
            >
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </div>
        </button>
      </div>
    </aside>

    <!-- Main column -->
    <div class="assistant-main flex min-h-0 min-w-0 flex-1 flex-col gap-3 overflow-hidden px-3 pt-3">
      <div class="flex shrink-0 items-start gap-3">
        <button
          type="button"
          class="md:hidden shrink-0 p-2 rounded-lg border border-gray-200 bg-white text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-400/50"
          :aria-label="$t('assistantChat.openChatHistory')"
          :aria-expanded="sidebarOpen"
          @click="sidebarOpen = !sidebarOpen"
        >
          <PanelLeft class="w-5 h-5" />
        </button>
        <div class="flex-1 min-w-0">
          <div class="flex items-start justify-between gap-3 min-w-0">
            <h1 class="text-2xl font-bold text-gray-800 min-w-0">
              {{ $t('assistantChat.title') }}
            </h1>
            <button
              type="button"
              class="hidden sm:inline-flex shrink-0 btn-aqua-emerald items-center gap-1.5 px-2.5 py-1.5 text-sm"
              @click="startNewChat"
            >
              <Plus class="w-4 h-4" />
              {{ $t('assistantChat.newChat') }}
            </button>
          </div>
        </div>
      </div>

      <div
        ref="scrollContainer"
        class="assistant-messages relative min-h-0 flex-1 overflow-x-hidden rounded-lg border border-gray-200 bg-white [overscroll-behavior-y:contain]"
        :class="isRestoringScroll ? 'overflow-hidden' : 'overflow-y-auto'"
        @scroll.passive="onScrollAreaScroll"
      >
        <!-- Until localStorage is read, avoid wrong empty state / CLS -->
        <div
          v-if="!loaded"
          class="p-4 space-y-3 animate-pulse"
          aria-hidden="true"
        >
          <div class="h-7 w-48 rounded-md bg-gray-200" />
          <div class="h-28 w-full max-w-lg rounded-md bg-gray-100" />
          <div class="h-6 w-40 rounded-md bg-gray-200" />
        </div>

        <template v-else>
          <!-- Non-empty: solid cover + skeleton until scroll restored; real thread stays invisible (layout preserved) -->
          <div
            v-if="isRestoringScroll"
            class="pointer-events-none absolute inset-0 z-10 flex min-h-full flex-col gap-3 bg-white p-4"
            aria-hidden="true"
          >
            <div class="h-4 w-3/4 max-w-md animate-pulse rounded-md bg-gray-200" />
            <div class="h-20 w-full max-w-md animate-pulse rounded-md bg-gray-100" />
            <div class="h-4 w-2/3 animate-pulse rounded-md bg-gray-200" />
            <div class="h-24 w-full max-w-md animate-pulse rounded-md bg-gray-100" />
            <div class="h-4 w-1/2 animate-pulse rounded-md bg-gray-200" />
          </div>

          <div
            class="p-4 space-y-4"
            :class="messagePaneVisibleClass"
            :aria-hidden="isRestoringScroll"
          >
            <div
              v-if="showEmptyChatHint"
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
              class="max-w-[80%] min-w-0 rounded-lg px-3 py-2 text-sm break-words bg-gray-100 text-gray-900 assistant-markdown"
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
                  :lang-id="locale"
                  :show-raw-output="isStepOutputVisible(stepKey(index, replyIdx, stepIdx))"
                  @toggle-raw="toggleStepOutput(stepKey(index, replyIdx, stepIdx))"
                />
              </div>
              <!-- Thinking dots while streaming and no reply yet for this model -->
              <div
                v-if="isStreamingThisSession && index === messages.length - 1 && !reply.content"
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
              class="max-w-[80%] min-w-0 rounded-lg px-3 py-2 text-sm break-words bg-gray-100 text-gray-900 assistant-markdown"
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
                  :lang-id="locale"
                  :show-raw-output="isStepOutputVisible(stepKey('legacy', index, stepIdx))"
                  @toggle-raw="toggleStepOutput(stepKey('legacy', index, stepIdx))"
                />
              </div>
              <div
                v-if="isStreamingThisSession && index === messages.length - 1 && !msg.content"
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
              v-if="!isRestoringScroll && isStreamingThisSession && (messages.length === 0 || messages[messages.length - 1]?.role !== 'assistant')"
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
        </template>
      </div>

      <form
        class="assistant-composer -mx-3 flex shrink-0 flex-col gap-2 border-t border-gray-100 bg-white px-3 pb-2 pt-3"
        @submit.prevent="handleSend"
      >
        <textarea
          v-model="input"
          class="textarea-field min-h-[80px] max-h-40 resize-y"
          :placeholder="$t('assistantChat.placeholder')"
          :disabled="isStreamingThisSession"
          @keydown.enter.exact.prevent="handleSend"
        />

        <div class="flex items-center justify-between gap-3 min-w-0">
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
          <span v-else class="min-w-0 flex-1 text-xs text-gray-500 truncate pr-2">
            {{ $t('assistantChat.hint') }}
          </span>

          <button
            type="submit"
            class="assistant-send-btn btn-aqua-cyan shrink-0 min-w-max px-4 py-1.5 text-sm disabled:opacity-60 disabled:cursor-not-allowed"
            :disabled="isStreamingThisSession || !input.trim()"
          >
            <span v-if="isStreamingThisSession">
              {{ $t('assistantChat.sending') }}
            </span>
            <span v-else>
              {{ $t('assistantChat.send') }}
            </span>
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, nextTick, computed, onMounted, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { useDebounceFn, useMediaQuery, onKeyStroke } from '@vueuse/core'
import { RotateCw, Trash2, Plus, PanelLeft, X, Search, MessageSquare } from 'lucide-vue-next'

import AssistantThoughtStep from '@/components/AssistantThoughtStep.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import { useSeoHead } from '@/composables/useSeoHead'
import { getApiBaseUrl, getAuthHeaders } from '../api'

const STORAGE_KEY = 'lensisku-assistant-chats-v1'
const MAX_SESSIONS = 100

const { locale, t } = useI18n()
useSeoHead({ title: t('assistantChat.title') })

const isDesktop = useMediaQuery('(min-width: 768px)')
const sidebarOpen = ref(false)
const chatSearchQuery = ref('')
const sessions = ref([])
const activeSessionId = ref(null)
const messages = ref([])
const loaded = ref(false)
const input = ref('')
/** Session id currently receiving a stream (only one in-flight request). */
const streamingSessionId = ref(null)
const isStreamingThisSession = computed(
  () =>
    streamingSessionId.value !== null && streamingSessionId.value === activeSessionId.value
)
const error = ref('')
const scrollContainer = ref(null)
/** When true, skip auto scroll-to-bottom (restore / session switch). */
const suppressAutoScroll = ref(false)
/** After scroll restore: show message pane (opacity 1). Hidden until then for non-empty chats. */
const chatMessagesReady = ref(true)
/** Keys: step id string (e.g. "0-0-0"); value: true if output is expanded */
const stepShowOutput = ref({})
/** Set before debounced persist so `updatedAt` bumps only on user send or assistant done/error. */
const pendingBumpUpdatedAt = ref(false)

/** True while localStorage is applied but scroll not yet restored for a non-empty thread. */
const isRestoringScroll = computed(
  () => loaded.value && messages.value.length > 0 && !chatMessagesReady.value
)

/** Only after restore: avoid flashing “empty chat” before messages hydrate from storage. */
const showEmptyChatHint = computed(
  () => loaded.value && chatMessagesReady.value && messages.value.length === 0
)

const messagePaneVisibleClass = computed(() => {
  if (!loaded.value) return 'invisible'
  if (messages.value.length === 0) return ''
  return chatMessagesReady.value
    ? 'transition-opacity duration-200 ease-out opacity-100'
    : 'invisible pointer-events-none'
})

const filteredSessions = computed(() => {
  const q = chatSearchQuery.value.trim().toLowerCase()
  const list = [...sessions.value].sort((a, b) => b.updatedAt - a.updatedAt)
  if (!q) return list
  return list.filter((s) => {
    if (s.title.toLowerCase().includes(q)) return true
    const blob = (s.messages || [])
      .map((m) => {
        if (m.role === 'user') return m.content || ''
        return assistantPlainText(m)
      })
      .join(' ')
      .toLowerCase()
    return blob.includes(q)
  })
})

function createId() {
  if (typeof crypto !== 'undefined' && crypto.randomUUID) {
    return crypto.randomUUID()
  }
  return `${Date.now()}-${Math.random().toString(36).slice(2, 11)}`
}

function assistantPlainText(msg) {
  if (!msg || msg.role !== 'assistant') return ''
  if (msg.replies && msg.replies.length > 0) {
    return msg.replies
      .map((r) => {
        const parts = [r.content || '']
        if (r.steps && r.steps.length > 0) {
          parts.push(
            ...r.steps.map((s) => `${s.action || ''} ${s.result || ''} ${s.tool_output || ''}`)
          )
        }
        return parts.join(' ')
      })
      .join(' ')
  }
  const legacy = [msg.content || '']
  if (msg.steps && msg.steps.length > 0) {
    legacy.push(...msg.steps.map((s) => `${s.action || ''} ${s.result || ''} ${s.tool_output || ''}`))
  }
  return legacy.join(' ')
}

function deriveTitle(msgs) {
  const first = msgs.find((m) => m.role === 'user')
  if (!first?.content?.trim()) return t('assistantChat.newChat')
  const text = first.content.replace(/\s+/g, ' ').trim()
  return text.length > 60 ? `${text.slice(0, 57)}…` : text
}

function formatSessionTime(ts) {
  try {
    return new Intl.DateTimeFormat(locale.value || undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(new Date(ts))
  } catch {
    return ''
  }
}

function loadFromStorage() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) {
      createInitialSession()
      return
    }
    const data = JSON.parse(raw)
    sessions.value = Array.isArray(data.sessions) ? data.sessions : []
    activeSessionId.value = data.activeId || sessions.value[0]?.id
    if (!sessions.value.length) {
      createInitialSession()
      return
    }
    let active = sessions.value.find((s) => s.id === activeSessionId.value)
    if (!active) {
      activeSessionId.value = sessions.value[0].id
      active = sessions.value[0]
    }
    messages.value = JSON.parse(JSON.stringify(active?.messages || []))
  } catch {
    createInitialSession()
  }
}

function createInitialSession() {
  const id = createId()
  sessions.value = [
    {
      id,
      title: t('assistantChat.newChat'),
      updatedAt: Date.now(),
      messages: [],
      scrollTop: 0,
      primaryModelId: null,
    },
  ]
  activeSessionId.value = id
  messages.value = []
}

function readScrollTopForPersist() {
  if (scrollContainer.value) return scrollContainer.value.scrollTop
  const s = sessions.value.find((x) => x.id === activeSessionId.value)
  return typeof s?.scrollTop === 'number' ? s.scrollTop : 0
}

function persistToStorage(options = {}) {
  if (!loaded.value || !activeSessionId.value) return
  const idx = sessions.value.findIndex((s) => s.id === activeSessionId.value)
  if (idx < 0) return
  const preserveUpdatedAt = options.preserveUpdatedAt === true
  const prevUpdatedAt = sessions.value[idx].updatedAt
  let updatedAt = prevUpdatedAt
  if (!preserveUpdatedAt && pendingBumpUpdatedAt.value) {
    updatedAt = Date.now()
  }
  pendingBumpUpdatedAt.value = false
  const session = {
    ...sessions.value[idx],
    messages: JSON.parse(JSON.stringify(messages.value)),
    updatedAt,
    title: deriveTitle(messages.value),
    scrollTop: readScrollTopForPersist(),
  }
  const others = sessions.value.filter((s) => s.id !== activeSessionId.value)
  const merged = [session, ...others]
  merged.sort((a, b) => b.updatedAt - a.updatedAt)
  sessions.value = merged.slice(0, MAX_SESSIONS)
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        version: 1,
        sessions: sessions.value,
        activeId: activeSessionId.value,
      })
    )
  } catch (e) {
    console.warn('assistant chat persist failed', e)
  }
}

const debouncedPersist = useDebounceFn(persistToStorage, 400)
const debouncedScrollPersist = useDebounceFn(() => {
  if (!loaded.value || !activeSessionId.value || !scrollContainer.value) return
  const idx = sessions.value.findIndex((s) => s.id === activeSessionId.value)
  if (idx < 0) return
  const st = scrollContainer.value.scrollTop
  const cur = sessions.value[idx]
  if (cur.scrollTop === st) return
  sessions.value.splice(idx, 1, { ...cur, scrollTop: st })
  debouncedPersist()
}, 200)

function onScrollAreaScroll() {
  debouncedScrollPersist()
}

/** When mobile browser chrome or visual viewport changes, max scroll range changes — reclamp so top of thread stays reachable. */
function clampMessageScroll() {
  const el = scrollContainer.value
  if (!el) return
  const max = Math.max(0, el.scrollHeight - el.clientHeight)
  if (el.scrollTop > max) el.scrollTop = max
}

function applyStoredScrollForActiveSession() {
  if (messages.value.length === 0) {
    chatMessagesReady.value = true
    return
  }
  const s = sessions.value.find((x) => x.id === activeSessionId.value)
  const raw = typeof s?.scrollTop === 'number' ? s.scrollTop : 0
  const apply = () => {
    const el = scrollContainer.value
    if (!el) return
    const max = Math.max(0, el.scrollHeight - el.clientHeight)
    el.scrollTop = Math.min(Math.max(0, raw), max)
  }
  nextTick(() => {
    apply()
    requestAnimationFrame(() => {
      apply()
      setTimeout(() => {
        apply()
        chatMessagesReady.value = true
      }, 200)
    })
  })
}

watch(
  messages,
  () => {
    if (!loaded.value) return
    debouncedPersist()
  },
  { deep: true }
)

onMounted(() => {
  suppressAutoScroll.value = true
  loadFromStorage()

  // Full-height route: keep layout viewport at top (Firefox mobile can offset when inner height mismatches dvh/svh).
  if (typeof window !== 'undefined') {
    window.scrollTo(0, 0)
  }

  window.addEventListener('resize', clampMessageScroll)
  const vv = typeof window !== 'undefined' ? window.visualViewport : null
  if (vv) {
    vv.addEventListener('resize', clampMessageScroll)
    vv.addEventListener('scroll', clampMessageScroll)
  }

  if (messages.value.length === 0) {
    loaded.value = true
    chatMessagesReady.value = true
    suppressAutoScroll.value = false
    return
  }
  chatMessagesReady.value = false
  // Keep `loaded` false for one frame so we never paint the v-else branch before messages are ready to reveal.
  nextTick(() => {
    loaded.value = true
    nextTick(() => {
      applyStoredScrollForActiveSession()
      nextTick(() => {
        suppressAutoScroll.value = false
      })
    })
  })
})

onBeforeUnmount(() => {
  if (loaded.value) persistToStorage()
  window.removeEventListener('resize', clampMessageScroll)
  const vv = typeof window !== 'undefined' ? window.visualViewport : null
  if (vv) {
    vv.removeEventListener('resize', clampMessageScroll)
    vv.removeEventListener('scroll', clampMessageScroll)
  }
})

onKeyStroke('Escape', () => {
  if (!isDesktop.value) sidebarOpen.value = false
})

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
  if (suppressAutoScroll.value) return
  nextTick(() => {
    if (scrollContainer.value) {
      scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight
    }
  })
}

watch(
  () => [messages.value.length, isStreamingThisSession.value],
  () => scrollToBottom()
)

/** Messages array to mutate for a session (live view when active, stored copy when not). */
function getSessionMessagesForMutation(sessionId) {
  if (activeSessionId.value === sessionId) {
    return messages.value
  }
  const s = sessions.value.find((x) => x.id === sessionId)
  return s?.messages ?? null
}

function selectSession(id) {
  if (id === activeSessionId.value) return
  persistToStorage({ preserveUpdatedAt: true })
  suppressAutoScroll.value = true
  activeSessionId.value = id
  const s = sessions.value.find((x) => x.id === id)
  const nextMsgs = s ? JSON.parse(JSON.stringify(s.messages)) : []
  chatMessagesReady.value = nextMsgs.length === 0
  messages.value = nextMsgs
  error.value = ''
  sidebarOpen.value = false
  nextTick(() => {
    applyStoredScrollForActiveSession()
    nextTick(() => {
      suppressAutoScroll.value = false
    })
  })
}

function startNewChat() {
  persistToStorage({ preserveUpdatedAt: true })
  suppressAutoScroll.value = true
  chatMessagesReady.value = true
  const id = createId()
  const newSession = {
    id,
    title: t('assistantChat.newChat'),
    updatedAt: Date.now(),
    messages: [],
    scrollTop: 0,
    primaryModelId: null,
  }
  sessions.value = [newSession, ...sessions.value.filter((s) => s.id !== id)].slice(0, MAX_SESSIONS)
  activeSessionId.value = id
  messages.value = []
  error.value = ''
  sidebarOpen.value = false
  nextTick(() => {
    if (scrollContainer.value) scrollContainer.value.scrollTop = 0
    suppressAutoScroll.value = false
    persistToStorage()
  })
}

/** OpenAI-shaped segments replayed to the backend (assistant + tool + assistant …). */
function pruneApiSegment(seg) {
  const o = { role: seg.role, content: seg.content ?? '' }
  if (seg.tool_calls != null) o.tool_calls = seg.tool_calls
  if (seg.tool_call_id != null) o.tool_call_id = seg.tool_call_id
  if (seg.name != null) o.name = seg.name
  return o
}

function pickPrimaryReply(msg, primaryModelId) {
  if (!msg || msg.role !== 'assistant') return null
  if (msg.replies?.length) {
    if (primaryModelId) {
      const hit = msg.replies.find((r) => r.model === primaryModelId)
      if (hit) return hit
    }
    return msg.replies[0]
  }
  return {
    model: null,
    modelName: null,
    steps: msg.steps || [],
    content: msg.content || '',
    apiTrace: msg.apiTrace,
  }
}

function buildPayloadMessages(msgList, session) {
  const primary = session?.primaryModelId ?? null
  const out = []
  for (const m of msgList) {
    if (m.role === 'user') {
      out.push({ role: 'user', content: m.content ?? '' })
      continue
    }
    if (m.role === 'assistant') {
      const reply = pickPrimaryReply(m, primary)
      if (reply?.apiTrace?.length) {
        for (const seg of reply.apiTrace) {
          out.push(pruneApiSegment(seg))
        }
      } else {
        const text = reply?.content ?? m.content ?? ''
        out.push({ role: 'assistant', content: text })
      }
    }
  }
  return out
}

function ensureSessionPrimaryModel(sessionId, modelId) {
  if (!modelId) return
  const idx = sessions.value.findIndex((s) => s.id === sessionId)
  if (idx < 0) return
  const s = sessions.value[idx]
  if (s.primaryModelId) return
  sessions.value.splice(idx, 1, { ...s, primaryModelId: modelId })
}

function deleteSession(id, e) {
  e?.stopPropagation?.()
  if (sessions.value.length <= 1) {
    messages.value = []
    chatMessagesReady.value = true
    error.value = ''
    persistToStorage()
    return
  }
  const next = sessions.value.filter((s) => s.id !== id)
  sessions.value = next
  if (activeSessionId.value === id) {
    suppressAutoScroll.value = true
    activeSessionId.value = next[0].id
    const switchedMsgs = JSON.parse(JSON.stringify(next[0].messages))
    chatMessagesReady.value = switchedMsgs.length === 0
    messages.value = switchedMsgs
    nextTick(() => {
      applyStoredScrollForActiveSession()
      nextTick(() => {
        suppressAutoScroll.value = false
      })
    })
  }
  error.value = ''
  persistToStorage()
}

async function performRequest(sessionId, options = {}) {
  const appendAssistant = options.appendAssistant !== false

  const msgList = getSessionMessagesForMutation(sessionId)
  if (!msgList || msgList.length === 0) return

  const session = sessions.value.find((s) => s.id === sessionId)
  const payloadMessages = buildPayloadMessages(msgList, session)

  if (appendAssistant) {
    msgList.push({
      role: 'assistant',
      content: '',
      steps: [],
      replies: [],
    })
  }

  const payload = {
    messages: payloadMessages,
    locale: locale.value,
  }

  const assistantIndex = msgList.length - 1

  function getOrCreateReply(modelId, modelName = null) {
    const list = getSessionMessagesForMutation(sessionId)
    if (!list || list.length <= assistantIndex) return null
    const msg = list[assistantIndex]
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
        const list = getSessionMessagesForMutation(sessionId)
        const msg = list?.[assistantIndex]
        if (!msg) continue
        const reply = modelId ? getOrCreateReply(modelId, modelName) : null
        if (modelId && !reply) continue
        if (!modelId && !msg.steps) msg.steps = []
        const steps = modelId ? reply.steps : msg.steps
        if (event.type === 'assistant_tool_calls') {
          if (modelId) {
            const r = getOrCreateReply(modelId, modelName)
            if (r) {
              if (!r.apiTrace) r.apiTrace = []
              r.apiTrace.push({
                role: 'assistant',
                content: event.content ?? '',
                tool_calls: event.tool_calls ?? null,
              })
            }
          } else {
            if (!msg.apiTrace) msg.apiTrace = []
            msg.apiTrace.push({
              role: 'assistant',
              content: event.content ?? '',
              tool_calls: event.tool_calls ?? null,
            })
          }
        } else if (event.type === 'step_start') {
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
            tool_call_id: event.tool_call_id ?? steps[idx]?.tool_call_id,
            tool_content_plain: event.tool_content_plain ?? steps[idx]?.tool_content_plain,
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
          const plain = event.tool_content_plain
          const tcId = event.tool_call_id
          if (plain != null && plain !== '') {
            const toolSeg = {
              role: 'tool',
              content: plain,
              tool_call_id: tcId ?? null,
              name: 'jbovlaste_semantic_search',
            }
            const dupCheck = (trace) =>
              trace?.some(
                (s) =>
                  s.role === 'tool' && tcId != null && s.tool_call_id === tcId
              )
            if (modelId) {
              const r = getOrCreateReply(modelId, modelName)
              if (r) {
                if (!r.apiTrace) r.apiTrace = []
                if (!dupCheck(r.apiTrace)) r.apiTrace.push(toolSeg)
              }
            } else {
              if (!msg.apiTrace) msg.apiTrace = []
              if (!dupCheck(msg.apiTrace)) msg.apiTrace.push(toolSeg)
            }
          }
        } else if (event.type === 'done') {
          ensureSessionPrimaryModel(sessionId, event.model)
          if (modelId) {
            const r = getOrCreateReply(modelId, modelName)
            if (r) {
              r.content = event.reply ?? ''
              if (!r.apiTrace) r.apiTrace = []
              r.apiTrace.push({ role: 'assistant', content: event.reply ?? '' })
            }
          } else {
            msg.content = event.reply ?? ''
            if (!msg.apiTrace) msg.apiTrace = []
            msg.apiTrace.push({ role: 'assistant', content: event.reply ?? '' })
          }
        } else if (event.type === 'error') {
          const errContent = event.error
            ? `_${t('assistantChat.error')}: ${event.error}_`
            : t('assistantChat.error') + (event.raw_response ? `\n\n**Debug:**\n\`\`\`\n${event.raw_response}\n\`\`\`` : '')
          if (modelId) {
            const r = getOrCreateReply(modelId)
            if (r) {
              r.content = errContent
              r.apiTrace = undefined
            }
          } else {
            msg.content = errContent
            msg.apiTrace = undefined
          }
        }
        if (event.type === 'done' || event.type === 'error') {
          pendingBumpUpdatedAt.value = true
        }
        if (loaded.value) debouncedPersist()
      } catch (_) {
        // ignore non-JSON or malformed lines
      }
    }
    if (done) break
  }
}

const handleSend = async () => {
  const content = input.value.trim()
  if (!content || streamingSessionId.value !== null) return

  error.value = ''
  pendingBumpUpdatedAt.value = true
  messages.value.push({
    role: 'user',
    content,
  })
  input.value = ''
  const sessionId = activeSessionId.value
  streamingSessionId.value = sessionId
  try {
    await performRequest(sessionId, { appendAssistant: true })
  } catch (e) {
    console.error(e)
    error.value = t('assistantChat.error')
  } finally {
    streamingSessionId.value = null
  }
}

const retryLast = async () => {
  if (streamingSessionId.value !== null || !error.value) return
  error.value = ''
  const sessionId = activeSessionId.value
  streamingSessionId.value = sessionId
  try {
    await performRequest(sessionId, { appendAssistant: false })
  } catch (e) {
    console.error(e)
    error.value = t('assistantChat.error')
  } finally {
    streamingSessionId.value = null
  }
}

</script>

<style scoped>
/*
 * Fill App.vue's full-height main only — do not use 100dvh−header here.
 * Main is already (100svh − app header); an extra viewport-based height can be
 * taller than that slot when 100dvh > 100svh (mobile toolbar), which can make
 * Firefox/Android shift the document scroll and hide the sticky header.
 */
@media (max-width: 767px) {
  .assistant-root {
    box-sizing: border-box;
    height: 100%;
    max-height: 100%;
    min-height: 0;
  }
}

/* aqua-base ellipsizes label spans; keep Send / Sending fully visible on narrow screens */
.assistant-send-btn {
  text-overflow: clip;
}
.assistant-send-btn :deep(span) {
  max-width: none;
  overflow: visible;
  text-overflow: clip;
}

.assistant-markdown :deep(.mathjax-content) {
  display: block;
  min-width: 0;
  max-width: 100%;
}

/*
 * LazyMathJax forces direct children to `inline`, which breaks <pre> and lets long code
 * widen past the bubble. Scroll horizontally inside <pre>; height grows with content (no inner vertical scrollbar).
 */
.assistant-markdown :deep(.mathjax-content > pre) {
  display: block !important;
  max-width: 100%;
  margin: 0.5rem 0;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  box-sizing: border-box;
  @apply rounded-md border border-gray-200 bg-gray-50/80 px-3 py-2 shadow-inner;
}

.assistant-markdown :deep(.mathjax-content pre code) {
  display: block;
  white-space: pre;
  word-break: normal;
  overflow-wrap: normal;
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
