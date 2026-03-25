<template>
   <!--
    Full-height route: shell fills App.vue main (already viewport − header).
    Mobile uses height:100% of that slot — not 100dvh−header again — so dvh/svh
    mismatch cannot overflow and confuse mobile Firefox scroll position.

    Main column (assistant-main): column flex — shrink-0 header, flex-1 scroll
    region (assistant-main-stage → assistant-messages), shrink-0 footer (composer).
    One scroll region (messages); composer stays at the bottom of the main column.
  -->
  <div class="assistant-root flex min-h-0 w-full min-w-0 flex-1 gap-0 overflow-hidden md:h-full">
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
           <X class="h-5 w-5" /> </button
        > <button
          type="button"
          class="flex min-w-0 flex-1 items-center justify-center gap-2 rounded-lg border border-dashed border-gray-300 bg-white/80 py-2.5 text-sm font-medium text-gray-700 hover:border-blue-400 hover:bg-blue-50/60 hover:text-blue-800 focus:outline-none focus:ring-2 focus:ring-blue-400/40 transition-colors"
          @click="startNewChat"
        >
           <Plus class="h-4 w-4 shrink-0" /> {{ $t('assistantChat.newChat') }} </button
        >
      </div>
       <!-- Row 2: search -->
      <div class="border-b border-gray-200/60 bg-white/40 px-2 py-2">

        <div class="relative">
           <Search
            class="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400"
            aria-hidden="true"
          /> <input
            v-model="chatSearchQuery"
            type="search"
            autocomplete="off"
            class="w-full rounded-lg border border-gray-200 bg-white py-2 pl-9 pr-3 text-sm text-gray-900 placeholder:text-gray-400 focus:border-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-400/30"
            :placeholder="$t('assistantChat.searchChatsPlaceholder')"
            @blur="onAssistantFormControlBlur"
          />
        </div>

      </div>

      <div class="flex-1 overflow-y-auto overflow-x-hidden min-h-0 p-2 space-y-1" role="list">

        <p v-if="filteredSessions.length === 0" class="px-2 py-6 text-center text-sm text-gray-500">
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
               <Trash2 class="w-3.5 h-3.5" /> </button
            >
          </div>
           </button
        >
      </div>

    </aside>
     <!-- Main column: flex column — header | flex-1 messages | footer composer -->
    <div class="assistant-main flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">

      <header class="assistant-main-header flex shrink-0 items-start gap-3 px-3 pt-3">
         <button
          type="button"
          class="md:hidden shrink-0 p-2 rounded-lg border border-gray-200 bg-white text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-400/50"
          :aria-label="$t('assistantChat.openChatHistory')"
          :aria-expanded="sidebarOpen"
          @click="sidebarOpen = !sidebarOpen"
        >
           <PanelLeft class="w-5 h-5" /> </button
        >
        <div class="min-w-0 flex-1">

          <div class="flex min-w-0 items-start justify-between gap-3">

            <h1 class="min-w-0 text-2xl font-bold text-gray-800">
               {{ $t('assistantChat.title') }}
            </h1>
             <button
              type="button"
              class="hidden shrink-0 sm:inline-flex btn-aqua-emerald items-center gap-1.5 px-2.5 py-1.5 text-sm"
              @click="startNewChat"
            >
               <Plus class="w-4 h-4" /> {{ $t('assistantChat.newChat') }} </button
            >
          </div>

        </div>

      </header>

      <div
        class="assistant-main-stage assistant-main-stage-bg flex min-h-0 flex-1 flex-col px-3 pt-3"
      >

        <div
          ref="scrollContainer"
          class="assistant-messages relative min-h-0 flex-1 overflow-x-hidden rounded-lg border border-gray-200 bg-white [overscroll-behavior-y:contain]"
          :class="isRestoringScroll ? 'overflow-hidden' : 'overflow-y-auto'"
          @scroll.passive="onScrollAreaScroll"
        >
           <!-- Until localStorage is read, avoid wrong empty state / CLS -->
          <div v-if="!loaded" class="p-4 space-y-3 animate-pulse" aria-hidden="true">

            <div class="h-7 w-48 rounded-md bg-gray-200" />

            <div class="h-28 w-full max-w-lg rounded-md bg-gray-100" />

            <div class="h-6 w-40 rounded-md bg-gray-200" />

          </div>
           <template v-else
            > <!-- Non-empty: solid cover + skeleton until scroll restored; real thread stays invisible (layout preserved) -->

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

              <div v-if="showEmptyChatHint" class="text-gray-500 text-sm">
                 {{ $t('assistantChat.emptyState') }}
              </div>

              <div
                v-for="(msg, index) in messages"
                :key="index"
                class="flex flex-col gap-3"
                :class="msg.role === 'user' ? 'items-end' : 'items-start'"
              >
                 <!-- User message: bubble + edit (row, pencil to the right of the bubble) -->
                <div v-if="msg.role === 'user'" class="flex w-full justify-end">

                  <div
                    v-if="editingMessageIndex === index"
                    id="assistant-inline-edit-panel"
                    ref="inlineEditPanelRef"
                    class="flex w-full max-w-[80%] flex-col gap-0 rounded-lg border border-blue-300 bg-white py-0 shadow-sm"
                  >
                     <label class="sr-only" :for="'assistant-edit-' + index">{{
                      $t('assistantChat.editMessage')
                    }}</label
                    > <textarea
                      :id="'assistant-edit-' + index"
                      v-model="editingMessageDraft"
                      class="textarea-field min-h-[4.5rem] w-full resize-y text-sm text-gray-900"
                      rows="4"
                      @keydown.escape.prevent="cancelEditMessage"
                      @blur="onAssistantFormControlBlur"
                    />
                    <div class="mx-2 my-2 flex justify-end gap-2">
                       <button type="button" class="btn-cancel" @click="cancelEditMessage">
                         {{ $t('assistantChat.cancelEdit') }} </button
                      > <button type="button" class="btn-insert" @click="commitEditMessage">
                         {{ $t('assistantChat.saveEdit') }} </button
                      >
                    </div>

                  </div>

                  <div
                    v-else
                    class="flex w-full max-w-[80%] flex-row items-end justify-end gap-1.5"
                  >

                    <div
                      class="min-w-0 max-w-[calc(100%-2.5rem)] rounded-lg px-3 py-2 text-sm break-words bg-blue-600 text-white whitespace-pre-wrap"
                    >
                       <span class="mb-1 block text-[11px] font-semibold text-blue-100"
                        > {{ $t('assistantChat.userLabel') }} </span
                      > <span>{{ msg.content }}</span
                      >
                    </div>
                     <button
                      v-if="canEditMessages"
                      type="button"
                      class="shrink-0 rounded-md p-1.5 text-gray-500 hover:bg-gray-100 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-400/40"
                      :aria-label="$t('assistantChat.editMessage')"
                      :title="$t('assistantChat.editMessage')"
                      @click="startEditMessage(index)"
                    >
                       <Pencil class="h-4 w-4" aria-hidden="true" /> </button
                    >
                  </div>

                </div>
                 <!-- Assistant: one bubble per reply (multi-model) or single top-level bubble -->
                <div
                  v-else-if="msg.role === 'assistant'"
                  class="flex w-full flex-col items-start gap-1"
                >

                  <div
                    v-for="(reply, replyIdx) in assistantReplies(msg)"
                    :key="replyIdx"
                    class="max-w-[80%] min-w-0 rounded-lg px-3 py-2 text-sm break-words bg-gray-100 text-gray-900 assistant-markdown"
                  >
                     <span class="mb-1 block text-[11px] font-semibold text-gray-500"
                      > {{
                        reply.modelName ||
                        (reply.model ? formatModelLabel(reply.model) : '') ||
                        $t('assistantChat.assistantLabel')
                      }} </span
                    > <!-- Thought process: steps with optional folded tool output -->
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
                      v-if="
                        isStreamingThisSession && index === messages.length - 1 && !reply.content
                      "
                      class="thinking-dots mb-1 flex min-h-[1.25rem] items-center gap-1"
                      role="status"
                      :aria-label="$t('assistantChat.thinking')"
                    >
                       <span class="thinking-dot" /> <span class="thinking-dot" /> <span
                        class="thinking-dot"
                      />
                    </div>
                     <LazyMathJax
                      v-if="reply.content"
                      :content="reply.content"
                      :enable-markdown="true"
                      :enable-curly-links="false"
                      :lang-id="locale"
                    />
                  </div>
                   <!-- Single top-level bubble when `replies` is empty -->
                  <div
                    v-if="assistantReplies(msg).length === 0"
                    class="max-w-[80%] min-w-0 rounded-lg px-3 py-2 text-sm break-words bg-gray-100 text-gray-900 assistant-markdown"
                  >
                     <span class="mb-1 block text-[11px] font-semibold text-gray-500"
                      > {{ $t('assistantChat.assistantLabel') }} </span
                    >
                    <div
                      v-if="msg.steps && msg.steps.length > 0"
                      class="thought-process mb-2 space-y-2"
                    >
                       <AssistantThoughtStep
                        v-for="(step, stepIdx) in msg.steps"
                        :key="stepIdx"
                        :step="step"
                        :lang-id="locale"
                        :show-raw-output="isStepOutputVisible(stepKey('single', index, stepIdx))"
                        @toggle-raw="toggleStepOutput(stepKey('single', index, stepIdx))"
                      />
                    </div>

                    <div
                      v-if="isStreamingThisSession && index === messages.length - 1 && !msg.content"
                      class="thinking-dots mb-1 flex min-h-[1.25rem] items-center gap-1"
                      role="status"
                      :aria-label="$t('assistantChat.thinking')"
                    >
                       <span class="thinking-dot" /> <span class="thinking-dot" /> <span
                        class="thinking-dot"
                      />
                    </div>
                     <LazyMathJax
                      v-if="msg.content"
                      :content="msg.content"
                      :enable-markdown="true"
                      :enable-curly-links="false"
                      :lang-id="locale"
                    />
                  </div>

                </div>

              </div>
               <!-- Thinking indicator when no assistant message yet (e.g. before stream starts) -->

              <div
                v-if="
                  !isRestoringScroll &&
                  isStreamingThisSession &&
                  (messages.length === 0 || messages[messages.length - 1]?.role !== 'assistant')
                "
                class="flex justify-start"
                role="status"
                :aria-label="$t('assistantChat.thinking')"
              >

                <div class="max-w-[80%] rounded-lg px-3 py-2.5 bg-gray-100 text-gray-600 text-sm">
                   <span class="block text-[11px] font-semibold text-gray-500 mb-1.5"
                    > {{ $t('assistantChat.assistantLabel') }} </span
                  >
                  <div class="thinking-dots flex items-center gap-1 min-h-[1.25rem]">
                     <span class="thinking-dot" /> <span class="thinking-dot" /> <span
                      class="thinking-dot"
                    />
                  </div>

                </div>

              </div>

            </div>
             </template
          >
        </div>

      </div>

      <footer
        class="assistant-main-footer shrink-0 w-full border-t border-gray-100 bg-white px-3 pt-3 pb-[max(0.75rem,env(safe-area-inset-bottom,0px))]"
      >

        <form class="assistant-composer flex flex-col gap-2" @submit.prevent="handleSend">

          <div class="relative min-w-0">
             <textarea
              v-model="input"
              class="textarea-field min-h-[88px] max-h-40 w-full resize-y pl-3 pr-12 pb-11 pt-2.5"
              :placeholder="$t('assistantChat.placeholder')"
              :disabled="isStreamingThisSession"
              @keydown.enter.exact.prevent="handleSend"
              @blur="onAssistantFormControlBlur"
            /> <button
              :type="isStreamingThisSession ? 'button' : 'submit'"
              class="assistant-composer-action !rounded-full absolute bottom-3 right-3 z-10 flex h-8 w-8 shrink-0 items-center justify-center !p-0 border border-gray-300 bg-white text-black shadow-sm transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-white focus:ring-gray-400/60 enabled:hover:bg-gray-50 enabled:hover:border-gray-400 disabled:cursor-not-allowed disabled:opacity-50"
              :disabled="!isStreamingThisSession && !input.trim()"
              :aria-label="
                isStreamingThisSession
                  ? isRecoveringRemoteStream
                    ? $t('assistantChat.recoveringStream')
                    : $t('assistantChat.stopRecording')
                  : $t('assistantChat.sendMessage')
              "
              @click="isStreamingThisSession ? stopStreaming() : undefined"
            >
               <Square
                v-if="isStreamingThisSession"
                class="h-4 w-4 shrink-0 text-black"
                :stroke-width="2.25"
                fill="currentColor"
                aria-hidden="true"
              /> <ArrowUp v-else class="h-6 w-6 text-black" stroke-width="2.25" aria-hidden="true" /> </button
            >
          </div>

          <div class="flex min-w-0 items-center gap-2 px-0.5 pt-0.5">

            <div v-if="error" class="flex min-w-0 flex-1 items-center gap-2">

              <p class="min-w-0 flex-1 truncate text-xs text-red-600"> {{ error }} </p>
               <button
                type="button"
                class="shrink-0 rounded p-1 text-red-600 hover:bg-red-50 focus:outline-none focus:ring-2 focus:ring-red-300"
                :aria-label="$t('assistantChat.retry')"
                :title="$t('assistantChat.retry')"
                @click="retryLast"
              >
                 <RotateCw class="h-4 w-4" /> </button
              >
            </div>

            <p
              v-else-if="isRecoveringRemoteStream"
              class="min-w-0 flex-1 text-xs leading-snug text-amber-800/90"
            >
               {{ $t('assistantChat.recoveringStream') }}
            </p>
             <span v-else class="min-w-0 flex-1 text-xs leading-snug text-gray-500"
              > {{ $t('assistantChat.hint') }} </span
            >
          </div>

        </form>

      </footer>

    </div>

  </div>

</template>

<script setup lang="ts">
import { ref, watch, nextTick, computed, onMounted, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { useDebounceFn, useMediaQuery, onKeyStroke } from '@vueuse/core'
import {
  RotateCw,
  Trash2,
  Plus,
  PanelLeft,
  X,
  Search,
  MessageSquare,
  ArrowUp,
  Square,
  Pencil,
} from 'lucide-vue-next'

import AssistantThoughtStep from '@/components/AssistantThoughtStep.vue'
import LazyMathJax from '@/components/LazyMathJax.vue'
import { useSeoHead } from '@/composables/useSeoHead'
import { useAuth } from '@/composables/useAuth'
import {
  getAssistantChatsCollectionUrl,
  getAssistantChatStreamPostUrl,
  getAssistantChatUrl,
  getAssistantPublicStreamPostUrl,
  getAuthHeaders,
} from '../api'

const STORAGE_KEY = 'lensisku-assistant-chats-v1'
const MAX_SESSIONS = 100

const { locale, t } = useI18n()
useSeoHead({ title: t('assistantChat.title') })

const auth = useAuth()
/** Logged-in users persist chats to PostgreSQL; anonymous users use localStorage. */
const isRemoteMode = computed(() => auth.state.isLoggedIn === true)

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
/** AbortController for the active stream fetch (stop button). */
const streamAbortController = ref(null)
/**
 * After reload the browser drops the SSE connection; the server may still be writing to Postgres.
 * We poll GET /assistant/chats/:id until the last assistant turn looks complete.
 */
const isRecoveringRemoteStream = ref(false)
let remoteStreamPollTimer = null

const isStreamingThisSession = computed(
  () =>
    (streamingSessionId.value !== null && streamingSessionId.value === activeSessionId.value) ||
    (isRecoveringRemoteStream.value === true && activeSessionId.value != null)
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
/** Inline edit: index in `messages` being edited (`null` = none). */
const editingMessageIndex = ref(null)
const editingMessageDraft = ref('')
/** Set when inline edit panel is mounted (one at a time). */
const inlineEditPanelRef = ref(null)

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

const canEditMessages = computed(() => loaded.value && !isStreamingThisSession.value)

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

/**
 * Whether the assistant turn ended (`streamFinished` from SSE / persisted state).
 * Only explicit `false` means still streaming; missing/`undefined` is treated as complete
 * so legacy DB rows do not trigger endless GET recovery polling instead of normal POST SSE.
 */
function isAssistantStreamComplete(msg) {
  if (!msg || msg.role !== 'assistant') return true
  if (msg.replies?.length) {
    return msg.replies.every((r) => r.streamFinished !== false)
  }
  return msg.streamFinished !== false
}

function stopRemoteStreamRecovery() {
  if (remoteStreamPollTimer != null) {
    clearInterval(remoteStreamPollTimer)
    remoteStreamPollTimer = null
  }
  isRecoveringRemoteStream.value = false
}

function startRemoteStreamRecovery(chatId) {
  if (!isRemoteMode.value || !chatId) return
  stopRemoteStreamRecovery()
  isRecoveringRemoteStream.value = true
  const tick = async () => {
    if (activeSessionId.value !== chatId || !isRecoveringRemoteStream.value) {
      stopRemoteStreamRecovery()
      return
    }
    try {
      const r = await fetch(getAssistantChatUrl(chatId), {
        headers: getAuthHeaders(),
      })
      if (!r.ok) return
      const row = await r.json()
      const msgs = Array.isArray(row.messages) ? row.messages : []
      messages.value = msgs
      const idx = sessions.value.findIndex((s) => s.id === chatId)
      if (idx >= 0) {
        const cur = sessions.value[idx]
        sessions.value.splice(idx, 1, {
          ...cur,
          messages: JSON.parse(JSON.stringify(msgs)),
          primaryModelId: row.primaryModelId ?? cur.primaryModelId ?? null,
        })
      }
      const last = msgs[msgs.length - 1]
      if (!last || isAssistantStreamComplete(last)) {
        stopRemoteStreamRecovery()
        pendingBumpUpdatedAt.value = true
        debouncedPersist()
      }
    } catch (e) {
      console.warn('assistant stream recovery poll failed', e)
    }
  }
  void tick()
  remoteStreamPollTimer = setInterval(tick, 1200)
}

function maybeStartRemoteStreamRecovery() {
  if (!isRemoteMode.value || !activeSessionId.value || !loaded.value) return
  const last = messages.value[messages.value.length - 1]
  if (last && !isAssistantStreamComplete(last)) {
    startRemoteStreamRecovery(activeSessionId.value)
  }
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
  const parts = [msg.content || '']
  if (msg.steps && msg.steps.length > 0) {
    parts.push(
      ...msg.steps.map((s) => `${s.action || ''} ${s.result || ''} ${s.tool_output || ''}`)
    )
  }
  return parts.join(' ')
}

function plainTextForEdit(msg) {
  if (!msg) return ''
  if (msg.role === 'user') return msg.content ?? ''
  return assistantPlainText(msg)
}

/** `ref` on a node inside `v-for` may be an array — unwrap to a single `HTMLElement`. */
function resolveInlineEditPanelEl() {
  const r = inlineEditPanelRef.value
  const v = Array.isArray(r) ? r[0] : r
  if (v instanceof HTMLElement) return v
  if (typeof document !== 'undefined') {
    const byId = document.getElementById('assistant-inline-edit-panel')
    if (byId instanceof HTMLElement) return byId
  }
  return null
}

function scrollInlineEditPanelIntoView() {
  nextTick(() => {
    nextTick(() => {
      requestAnimationFrame(() => {
        const panel = resolveInlineEditPanelEl()
        const sc = scrollContainer.value
        if (!panel || !sc) return
        panel.scrollIntoView({ block: 'end', behavior: 'auto' })
        requestAnimationFrame(() => {
          const pr = panel.getBoundingClientRect()
          const sr = sc.getBoundingClientRect()
          const pad = 8
          if (pr.bottom > sr.bottom - pad) {
            sc.scrollTop += pr.bottom - sr.bottom + pad
          }
          if (pr.top < sr.top + pad) {
            sc.scrollTop += pr.top - sr.top - pad
          }
        })
      })
    })
  })
}

function startEditMessage(index) {
  if (!canEditMessages.value) return
  const msg = messages.value[index]
  if (!msg) return
  editingMessageIndex.value = index
  editingMessageDraft.value = plainTextForEdit(msg)
  scrollInlineEditPanelIntoView()
}

function cancelEditMessage() {
  editingMessageIndex.value = null
  editingMessageDraft.value = ''
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

async function waitAuthReady() {
  for (let i = 0; i < 200; i++) {
    if (!auth.state.isLoading) return
    await new Promise((r) => setTimeout(r, 50))
  }
}

function mapServerChat(row) {
  const msgs = Array.isArray(row.messages) ? row.messages : []
  return {
    id: row.id,
    title: row.title?.trim() ? row.title : t('assistantChat.newChat'),
    updatedAt: new Date(row.updatedAt).getTime(),
    messages: msgs,
    scrollTop: typeof row.scrollTop === 'number' ? row.scrollTop : 0,
    primaryModelId: row.primaryModelId ?? null,
    _stub: false,
  }
}

async function fetchSessionIfNeeded(id) {
  const s = sessions.value.find((x) => x.id === id)
  if (!s?._stub) return
  const r = await fetch(getAssistantChatUrl(id), { headers: getAuthHeaders() })
  if (!r.ok) return
  const row = await r.json()
  const mapped = mapServerChat(row)
  const idx = sessions.value.findIndex((x) => x.id === id)
  if (idx >= 0) sessions.value.splice(idx, 1, mapped)
}

async function loadFromServer() {
  const headers = getAuthHeaders()
  let listRes = await fetch(getAssistantChatsCollectionUrl(), { headers })
  if (!listRes.ok) {
    throw new Error(await listRes.text())
  }
  let list = await listRes.json()
  if (!Array.isArray(list)) list = []
  if (list.length === 0) {
    const cr = await fetch(getAssistantChatsCollectionUrl(), { method: 'POST', headers })
    if (!cr.ok) throw new Error(await cr.text())
    const created = await cr.json()
    list = [
      {
        id: created.id,
        title: created.title,
        updatedAt: created.updatedAt,
      },
    ]
  }
  const sessionsPartial = list.map((item) => ({
    id: item.id,
    title: item.title?.trim() ? item.title : t('assistantChat.newChat'),
    updatedAt: new Date(item.updatedAt).getTime(),
    messages: [],
    scrollTop: 0,
    primaryModelId: null,
    _stub: true,
  }))
  const firstId = sessionsPartial[0].id
  const fullRes = await fetch(getAssistantChatUrl(firstId), { headers })
  if (!fullRes.ok) throw new Error(await fullRes.text())
  const full = await fullRes.json()
  const firstSession = mapServerChat(full)
  sessions.value = sessionsPartial.map((s) => (s.id === firstId ? firstSession : s))
  activeSessionId.value = firstId
  messages.value = JSON.parse(JSON.stringify(firstSession.messages))
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

/** Merges live `messages` into the active session row and sidebar order. */
function mergeActiveSessionIntoState(options: { preserveUpdatedAt?: boolean } = {}) {
  if (!loaded.value || !activeSessionId.value) return null
  const idx = sessions.value.findIndex((s) => s.id === activeSessionId.value)
  if (idx < 0) return null
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
  return session
}

async function persistToServerImmediate(options = {}) {
  const session = mergeActiveSessionIntoState(options)
  if (!session || session._stub) return
  const body = {
    title: session.title,
    messages: session.messages,
    primaryModelId: session.primaryModelId ?? null,
    scrollTop: session.scrollTop,
  }
  const r = await fetch(getAssistantChatUrl(session.id), {
    method: 'PUT',
    headers: {
      ...getAuthHeaders(),
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(body),
  })
  if (!r.ok) {
    console.warn('assistant chat PUT failed', await r.text())
  }
}

function persistToStorageLocal(options = {}) {
  mergeActiveSessionIntoState(options)
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

function persistToStorage(options = {}) {
  if (!loaded.value || !activeSessionId.value) return
  if (isRemoteMode.value) {
    void persistToServerImmediate(options)
  } else {
    persistToStorageLocal(options)
  }
}

const debouncedPersist = useDebounceFn(() => {
  if (isRemoteMode.value) {
    void persistToServerImmediate({})
  } else {
    persistToStorageLocal({})
  }
}, 400)
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

const ASSISTANT_ROOT_SELECTOR = '.assistant-root'

/**
 * Mobile keyboards often scroll the layout/visual viewport; after blur the page can stay offset with no way to scroll back.
 * Reset document scroll and reclamp the messages pane. Skip when focus moved to another input inside this page.
 */
function onAssistantFormControlBlur() {
  if (typeof window === 'undefined') return
  requestAnimationFrame(() => {
    const ae = document.activeElement
    if (
      ae instanceof HTMLElement &&
      ae.closest(ASSISTANT_ROOT_SELECTOR) &&
      (ae.tagName === 'INPUT' || ae.tagName === 'TEXTAREA' || ae.isContentEditable)
    ) {
      return
    }
    window.scrollTo({ top: 0, left: 0, behavior: 'auto' })
    document.documentElement.scrollTop = 0
    document.body.scrollTop = 0
    clampMessageScroll()
  })
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

onMounted(async () => {
  suppressAutoScroll.value = true
  await waitAuthReady()
  try {
    if (isRemoteMode.value) {
      await loadFromServer()
    } else {
      loadFromStorage()
    }
  } catch (e) {
    console.error(e)
    if (!isRemoteMode.value) loadFromStorage()
    else {
      error.value = t('assistantChat.error')
      loadFromStorage()
    }
  }

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
        maybeStartRemoteStreamRecovery()
      })
    })
  })
})

onBeforeUnmount(() => {
  stopRemoteStreamRecovery()
  if (loaded.value) persistToStorage()
  window.removeEventListener('resize', clampMessageScroll)
  const vv = typeof window !== 'undefined' ? window.visualViewport : null
  if (vv) {
    vv.removeEventListener('resize', clampMessageScroll)
    vv.removeEventListener('scroll', clampMessageScroll)
  }
})

onKeyStroke('Escape', (e) => {
  if (editingMessageIndex.value !== null) {
    cancelEditMessage()
    e.preventDefault?.()
    return
  }
  if (!isDesktop.value) sidebarOpen.value = false
})

/** Returns array of reply objects for an assistant message (multi-model or single top-level bubble). */
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

/** Pixels from the bottom of the messages pane to still count as “at bottom” (sticky follow). */
const SCROLL_BOTTOM_THRESHOLD_PX = 48

function isMessagePaneNearBottom() {
  const el = scrollContainer.value
  if (!el) return true
  const { scrollTop, scrollHeight, clientHeight } = el
  const maxScroll = scrollHeight - clientHeight
  if (maxScroll <= 0) return true
  return maxScroll - scrollTop <= SCROLL_BOTTOM_THRESHOLD_PX
}

/** User explicitly followed the thread (send, edit submit) — always jump to latest. */
function scrollToBottomForce() {
  if (suppressAutoScroll.value) return
  nextTick(() => {
    if (scrollContainer.value) {
      scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight
    }
  })
}

/** Live updates (streaming): only scroll if the user was already pinned to the bottom. */
function scrollToBottomIfPinned() {
  if (suppressAutoScroll.value) return
  if (!isMessagePaneNearBottom()) return
  nextTick(() => {
    if (scrollContainer.value) {
      scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight
    }
  })
}

/** Replace message at `index` with edited text; drop all messages after it (local + RAM). */
function commitEditMessage() {
  const index = editingMessageIndex.value
  if (index == null || index < 0) return
  if (streamingSessionId.value !== null) return
  const text = editingMessageDraft.value.trim()
  if (!text) return
  const original = messages.value[index]
  if (!original) return
  const prefix = messages.value.slice(0, index)
  let newMsg
  if (original.role === 'user') {
    newMsg = { role: 'user', content: text }
  } else {
    newMsg = {
      role: 'assistant',
      content: text,
      steps: [],
      replies: [],
      streamFinished: true,
    }
  }
  messages.value = [...prefix, newMsg]
  pendingBumpUpdatedAt.value = true
  cancelEditMessage()
  const sessionId = activeSessionId.value
  const appendAssistant = newMsg.role === 'user'
  nextTick(() => {
    scrollToBottomForce()
    void runAssistantStream(sessionId, appendAssistant)
  })
}

watch(activeSessionId, () => {
  cancelEditMessage()
})

watch(
  () => [messages.value.length, isStreamingThisSession.value],
  () => {
    if (!loaded.value) return
    scrollToBottomIfPinned()
  }
)

watch(
  messages,
  () => {
    if (!loaded.value) return
    if (!isStreamingThisSession.value) return
    scrollToBottomIfPinned()
  },
  { deep: true }
)

/** Messages array to mutate for a session (live view when active, stored copy when not). */
function getSessionMessagesForMutation(sessionId) {
  if (activeSessionId.value === sessionId) {
    return messages.value
  }
  const s = sessions.value.find((x) => x.id === sessionId)
  return s?.messages ?? null
}

async function selectSession(id) {
  if (id === activeSessionId.value) return
  stopRemoteStreamRecovery()
  persistToStorage({ preserveUpdatedAt: true })
  suppressAutoScroll.value = true
  activeSessionId.value = id
  if (isRemoteMode.value) {
    try {
      await fetchSessionIfNeeded(id)
    } catch (e) {
      console.error(e)
    }
  }
  const s = sessions.value.find((x) => x.id === id)
  const nextMsgs = s ? JSON.parse(JSON.stringify(s.messages || [])) : []
  chatMessagesReady.value = nextMsgs.length === 0
  messages.value = nextMsgs
  error.value = ''
  sidebarOpen.value = false
  nextTick(() => {
    applyStoredScrollForActiveSession()
    nextTick(() => {
      suppressAutoScroll.value = false
      maybeStartRemoteStreamRecovery()
    })
  })
}

async function startNewChat() {
  error.value = ''
  stopRemoteStreamRecovery()
  persistToStorage({ preserveUpdatedAt: true })
  suppressAutoScroll.value = true
  chatMessagesReady.value = true
  if (isRemoteMode.value) {
    try {
      const r = await fetch(getAssistantChatsCollectionUrl(), {
        method: 'POST',
        headers: getAuthHeaders(),
      })
      if (!r.ok) throw new Error(await r.text())
      const created = await r.json()
      const newSession = {
        id: created.id,
        title: created.title?.trim() ? created.title : t('assistantChat.newChat'),
        updatedAt: new Date(created.updatedAt).getTime(),
        messages: [],
        scrollTop: 0,
        primaryModelId: null,
        _stub: false,
      }
      sessions.value = [newSession, ...sessions.value.filter((s) => s.id !== newSession.id)].slice(
        0,
        MAX_SESSIONS
      )
      activeSessionId.value = newSession.id
      messages.value = []
    } catch (e) {
      console.error(e)
      error.value = t('assistantChat.error')
    }
  } else {
    const id = createId()
    const newSession = {
      id,
      title: t('assistantChat.newChat'),
      updatedAt: Date.now(),
      messages: [],
      scrollTop: 0,
      primaryModelId: null,
    }
    sessions.value = [newSession, ...sessions.value.filter((s) => s.id !== id)].slice(
      0,
      MAX_SESSIONS
    )
    activeSessionId.value = id
    messages.value = []
  }
  sidebarOpen.value = false
  nextTick(() => {
    if (scrollContainer.value) scrollContainer.value.scrollTop = 0
    suppressAutoScroll.value = false
    persistToStorage()
  })
}

/** OpenAI-shaped segments replayed to the backend (assistant + tool + assistant …). */
function pruneApiSegment(seg: Record<string, unknown>) {
  const o: Record<string, unknown> = { role: seg.role, content: seg.content ?? '' }
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

async function deleteSession(id, e) {
  e?.stopPropagation?.()
  if (isRemoteMode.value) {
    try {
      const r = await fetch(getAssistantChatUrl(id), {
        method: 'DELETE',
        headers: getAuthHeaders(),
      })
      if (!r.ok && r.status !== 404) {
        console.warn('delete chat failed', await r.text())
      }
    } catch (err) {
      console.error(err)
    }
  }
  if (sessions.value.length <= 1) {
    if (isRemoteMode.value) {
      try {
        const r = await fetch(getAssistantChatsCollectionUrl(), {
          method: 'POST',
          headers: getAuthHeaders(),
        })
        if (r.ok) {
          const created = await r.json()
          const empty = {
            id: created.id,
            title: t('assistantChat.newChat'),
            updatedAt: new Date(created.updatedAt).getTime(),
            messages: [],
            scrollTop: 0,
            primaryModelId: null,
            _stub: false,
          }
          sessions.value = [empty]
          activeSessionId.value = empty.id
          messages.value = []
          chatMessagesReady.value = true
          error.value = ''
          return
        }
      } catch (_) {
        /* fall through */
      }
    }
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
    if (isRemoteMode.value) {
      try {
        await fetchSessionIfNeeded(next[0].id)
      } catch (_) {
        /* ignore */
      }
    }
    const switchedMsgs = JSON.parse(JSON.stringify(next[0].messages || []))
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

function stopStreaming() {
  streamAbortController.value?.abort()
  if (isRecoveringRemoteStream.value) {
    const sid = activeSessionId.value
    stopRemoteStreamRecovery()
    if (sid) {
      abortAssistantMessage(sid)
      pendingBumpUpdatedAt.value = true
      void persistToServerImmediate({})
    }
  }
}

/** After user aborts a stream: drop empty assistant stub or append a short stopped note. */
function abortAssistantMessage(sessionId) {
  const list = getSessionMessagesForMutation(sessionId)
  if (!list?.length) return
  const last = list[list.length - 1]
  if (last.role !== 'assistant') return

  const hasVisible =
    (last.content && last.content.trim()) ||
    (last.replies &&
      last.replies.some(
        (r) =>
          (r.content && r.content.trim()) ||
          (r.steps && r.steps.some((s) => s.result && s.result !== '…'))
      )) ||
    (last.steps && last.steps.some((s) => s.result && s.result !== '…'))

  const note = `_${t('assistantChat.stopped')}_`
  if (!hasVisible) {
    list.pop()
    return
  }

  last.streamFinished = true
  if (last.replies?.length) {
    for (const r of last.replies) {
      r.streamFinished = true
    }
    const r = last.replies.find((x) => x.content?.trim()) ?? last.replies[last.replies.length - 1]
    r.content = (r.content || '') + (r.content?.trim() ? '\n\n' : '') + note
  } else {
    last.content = (last.content || '') + (last.content?.trim() ? '\n\n' : '') + note
  }
}

async function performRequest(
  sessionId: string,
  options: { appendAssistant?: boolean; signal?: AbortSignal } = {}
) {
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
      streamFinished: false,
    })
  } else {
    const last = msgList[msgList.length - 1]
    if (last?.role === 'assistant') {
      last.streamFinished = false
      if (last.replies?.length) {
        for (const r of last.replies) {
          r.streamFinished = false
        }
      }
    }
  }

  const payload = {
    messages: payloadMessages,
    locale: locale.value,
  }

  const assistantIndex = msgList.length - 1

  if (isRemoteMode.value) {
    await persistToServerImmediate({})
  }

  function getOrCreateReply(modelId, modelName = null) {
    const list = getSessionMessagesForMutation(sessionId)
    if (!list || list.length <= assistantIndex) return null
    const msg = list[assistantIndex]
    if (!msg.replies) msg.replies = []
    let r = msg.replies.find((x) => x.model === modelId)
    if (!r) {
      r = {
        model: modelId,
        modelName: modelName || null,
        steps: [],
        content: '',
        streamFinished: false,
      }
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

  const streamUrl = isRemoteMode.value
    ? getAssistantChatStreamPostUrl(sessionId)
    : getAssistantPublicStreamPostUrl()
  const streamBody = isRemoteMode.value
    ? JSON.stringify({ locale: locale.value })
    : JSON.stringify(payload)

  const response = await fetch(streamUrl, {
    method: 'POST',
    headers,
    body: streamBody,
    signal: options.signal,
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
        // Debug-only SSE payloads from the backend (model plan, per-model errors). Not shown in UI.
        if (event.type === 'stream_debug') {
          if (import.meta.env.DEV) {
            console.debug('[assistant stream_debug]', event.debug)
          }
          continue
        }
        if (event.type === 'parallel_branches') {
          const listPb = getSessionMessagesForMutation(sessionId)
          const msgPb = listPb?.[assistantIndex]
          if (msgPb) {
            const models = event.models
            if (Array.isArray(models)) {
              for (const p of models) {
                if (p?.id) getOrCreateReply(p.id, p.name ?? null)
              }
            }
            if (loaded.value) debouncedPersist()
          }
          continue
        }
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
          const idx = typeof event.index === 'number' ? event.index : steps.length
          while (steps.length < idx) {
            steps.push({
              action: '',
              result: '…',
              tool_output: undefined,
              assistant_reasoning: undefined,
            })
          }
          if (steps.length === idx) {
            steps.push({
              action: event.action ?? '',
              result: '…',
              tool_output: undefined,
              assistant_reasoning: event.assistant_reasoning,
            })
          } else {
            const existing = steps[idx]
            steps[idx] = {
              action: event.action ?? existing?.action ?? '',
              result: '…',
              tool_output: existing?.tool_output,
              assistant_reasoning: event.assistant_reasoning ?? existing?.assistant_reasoning,
            }
          }
        } else if (event.type === 'step') {
          if (modelId) getOrCreateReply(modelId, modelName)
          const idx = typeof event.index === 'number' ? event.index : Math.max(0, steps.length - 1)
          const stepPayload = {
            action: event.action ?? steps[idx]?.action ?? '',
            result: event.result ?? '',
            tool_output: event.tool_output ?? steps[idx]?.tool_output,
            tool_call_id: event.tool_call_id ?? steps[idx]?.tool_call_id,
            tool_content_plain: event.tool_content_plain ?? steps[idx]?.tool_content_plain,
            assistant_reasoning: event.assistant_reasoning ?? steps[idx]?.assistant_reasoning,
          }
          while (steps.length < idx) {
            steps.push({
              action: stepPayload.action,
              result: '…',
              tool_output: undefined,
              assistant_reasoning: undefined,
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
              trace?.some((s) => s.role === 'tool' && tcId != null && s.tool_call_id === tcId)
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
              r.streamFinished = true
              if (!r.apiTrace) r.apiTrace = []
              r.apiTrace.push({ role: 'assistant', content: event.reply ?? '' })
            }
          } else {
            msg.content = event.reply ?? ''
            msg.streamFinished = true
            if (!msg.apiTrace) msg.apiTrace = []
            msg.apiTrace.push({ role: 'assistant', content: event.reply ?? '' })
          }
        } else if (event.type === 'error') {
          const errContent = event.error
            ? `_${t('assistantChat.error')}: ${event.error}_`
            : t('assistantChat.error') +
              (event.raw_response ? `\n\n**Debug:**\n\`\`\`\n${event.raw_response}\n\`\`\`` : '')
          if (modelId) {
            const r = getOrCreateReply(modelId)
            if (r) {
              r.content = errContent
              r.streamFinished = true
              r.apiTrace = undefined
            }
          } else {
            msg.content = errContent
            msg.streamFinished = true
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
  if (!content || isStreamingThisSession.value) return

  error.value = ''
  pendingBumpUpdatedAt.value = true
  messages.value.push({
    role: 'user',
    content,
  })
  input.value = ''
  nextTick(() => {
    scrollToBottomForce()
  })
  const sessionId = activeSessionId.value
  await runAssistantStream(sessionId, true)
}

const retryLast = async () => {
  if (isStreamingThisSession.value || !error.value) return
  error.value = ''
  nextTick(() => {
    scrollToBottomForce()
  })
  const sessionId = activeSessionId.value
  await runAssistantStream(sessionId, false)
}

/** Stream assistant reply; `appendAssistant` true when the last message is user (append empty assistant first). */
async function runAssistantStream(sessionId, appendAssistant) {
  stopRemoteStreamRecovery()
  error.value = ''
  const ac = new AbortController()
  streamAbortController.value = ac
  streamingSessionId.value = sessionId
  try {
    await performRequest(sessionId, { appendAssistant, signal: ac.signal })
  } catch (e) {
    if (e?.name === 'AbortError' || e?.cause?.name === 'AbortError') {
      abortAssistantMessage(sessionId)
      error.value = ''
      pendingBumpUpdatedAt.value = true
      if (loaded.value) debouncedPersist()
      return
    }
    console.error(e)
    error.value = t('assistantChat.error')
  } finally {
    streamingSessionId.value = null
    streamAbortController.value = null
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

/* Same horizontal stripes as global `header, footer` in App.vue */
.assistant-main-stage-bg {
  background: repeating-linear-gradient(#f8f8f8, #f8f8f8 4px, #ffffff 4px, #ffffff 8px);
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

.assistant-markdown :deep(.mathjax-content .mathjax-table-wrap) {
  @apply max-w-full;
}

.assistant-markdown :deep(.mathjax-content table) {
  @apply my-2 text-sm;
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

