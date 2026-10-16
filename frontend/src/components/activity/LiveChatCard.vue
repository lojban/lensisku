<template>
  <article
    class="bg-white border rounded-lg p-3 hover:border-blue-300 transition-colors min-w-48"
    :aria-label="t('footer.liveChat')"
    :aria-busy="!historyLoaded"
  >
    <div class="flex flex-wrap items-center justify-between gap-2 min-h-8">
      <span
        class="inline-block px-2 py-1 text-xs font-medium rounded-full bg-indigo-100 text-indigo-800"
      >
        {{ t('footer.liveChat') }}
      </span>
      <SocialLinks class="max-w-full" :buttons="true" />
    </div>
    <!-- Height capped only while loading; after history loads, grow with content (CLS OK). -->
    <div
      ref="listEl"
      class="mt-2 w-full space-y-1 rounded bg-gray-50 p-2"
      :class="historyLoaded ? '' : 'min-h-[7.5rem] max-h-[7.5rem] overflow-y-auto'"
    >
      <div v-if="!historyLoaded" class="space-y-2 skeleton-pulse" aria-hidden="true">
        <div class="skeleton-bone h-3.5 w-4/5" />
        <div class="skeleton-bone h-3.5 w-3/5" />
        <div class="skeleton-bone--soft h-3.5 w-2/3" />
        <div class="skeleton-bone h-3.5 w-1/2" />
      </div>
      <template v-else>
        <div
          v-for="(msg, index) in messageStack"
          :key="`${msg.w}-${index}-${msg.d.slice(0, 24)}`"
          class="break-words text-sm leading-snug"
        >
          <span class="font-medium text-blue-600">{{ msg.w }}:</span>
          <span class="ml-1 text-gray-700">
            <template v-for="(part, partIndex) in linkify(msg.d)" :key="partIndex">
              <a
                v-if="part.href"
                :href="part.href"
                target="_blank"
                rel="noopener noreferrer"
                class="card-meta-link break-all"
              >
                {{ part.text }}
              </a>
              <template v-else>{{ part.text }}</template>
            </template>
          </span>
        </div>
      </template>
    </div>
  </article>
</template>

<script setup lang="ts">
import type { Socket } from 'socket.io-client'
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import SocialLinks from '@/components/SocialLinks.vue'

type ChatMessage = { d: string; s?: string; w: string }
type TextPart = { text: string; href?: string }

const { t } = useI18n()

const messageStack = ref<ChatMessage[]>([])
const listEl = ref<HTMLElement | null>(null)
const socketConnected = ref(false)
/** True after history arrives, connect fails, or a short timeout — swaps skeleton for content. */
const historyLoaded = ref(false)

/** Runtime client is still loaded via dynamic import (type-only import above). */
let socket: Socket | null = null
let historyTimeout: ReturnType<typeof setTimeout> | null = null
let idleHandle: number | null = null
let connectTimer: ReturnType<typeof setTimeout> | null = null
let cancelled = false

const URL_RE = /https?:\/\/[^\s<>"']+|www\.[^\s<>"']+/gi
const TRAILING_PUNCT_RE = /[.,;:!?)]}'"»]+$/

const sanitize = (text: string) =>
  text
    .replace(/<[^>]*>?/gm, '')
    .replace(/\s+/g, ' ')
    .trim()

/** Split plain text into safe text/link segments for http(s) and www. URLs. */
const linkify = (text: string): TextPart[] => {
  const parts: TextPart[] = []
  let lastIndex = 0
  const re = new RegExp(URL_RE.source, 'gi')
  let match: RegExpExecArray | null
  while ((match = re.exec(text)) !== null) {
    let raw = match[0]
    let trailing = ''
    const punct = raw.match(TRAILING_PUNCT_RE)
    if (punct) {
      trailing = punct[0]
      raw = raw.slice(0, -trailing.length)
    }
    if (match.index > lastIndex) {
      parts.push({ text: text.slice(lastIndex, match.index) })
    }
    if (raw) {
      const href = /^www\./i.test(raw) ? `https://${raw}` : raw
      if (/^https?:\/\//i.test(href)) {
        parts.push({ text: raw, href })
      } else {
        parts.push({ text: raw })
      }
    }
    if (trailing) parts.push({ text: trailing })
    lastIndex = match.index + match[0].length
  }
  if (lastIndex < text.length) parts.push({ text: text.slice(lastIndex) })
  return parts.length ? parts : [{ text }]
}

const markHistoryLoaded = () => {
  historyLoaded.value = true
  if (historyTimeout) {
    clearTimeout(historyTimeout)
    historyTimeout = null
  }
}

const scrollToBottom = async () => {
  await nextTick()
  const el = listEl.value
  if (el) el.scrollTop = el.scrollHeight
}

const pushMessages = (messages: ChatMessage[]) => {
  const sanitized = messages.map((m) => ({ ...m, d: sanitize(m.d) })).filter((m) => m.d)
  messageStack.value = [...messageStack.value, ...sanitized].slice(-40)
  void scrollToBottom()
}

/**
 * Connect after the browser is idle so socket.io never blocks home search/filters
 * (those only wait on getLanguages via HomePage isInitialLoading).
 */
const connectWhenIdle = () => {
  const run = () => {
    idleHandle = null
    connectTimer = null
    void connectSocket()
  }
  const ric = window.requestIdleCallback as
    ((cb: () => void, opts?: { timeout: number }) => number) | undefined
  if (typeof ric === 'function') {
    idleHandle = ric(run, { timeout: 2500 })
  } else {
    connectTimer = setTimeout(run, 1)
  }
}

const connectSocket = async () => {
  if (cancelled) return

  // Avoid an infinite skeleton if the socket never delivers history.
  historyTimeout = setTimeout(markHistoryLoaded, 4000)

  try {
    const { io } = await import('socket.io-client')
    if (cancelled) return

    socket = io('wss://jbotcan.org:9091', {
      transports: ['polling', 'websocket'],
    })

    socket.on('connect', () => {
      socketConnected.value = true
    })

    socket.on('connect_error', () => {
      console.error('1chat connection error')
      markHistoryLoaded()
    })

    socket.on(
      'sentFrom',
      (data: { data?: { chunk?: string; channelId?: string; author?: string } }) => {
        if (!socketConnected.value || !data?.data) return
        const i = data.data
        markHistoryLoaded()
        pushMessages([
          {
            d: i.chunk ?? '',
            s: i.channelId,
            w: i.author ?? '',
          },
        ])
      }
    )

    socket.on('history', (data: Array<{ chunk?: string; channelId?: string; author?: string }>) => {
      if (!Array.isArray(data)) {
        markHistoryLoaded()
        return
      }
      if (socketConnected.value) {
        pushMessages(
          data.slice(-20).map((m) => ({
            d: m.chunk ?? '',
            s: m.channelId,
            w: m.author ?? '',
          }))
        )
      }
      markHistoryLoaded()
    })
  } catch (e) {
    console.error('1chat failed to load', e)
    markHistoryLoaded()
  }
}

onMounted(() => {
  connectWhenIdle()
})

onUnmounted(() => {
  cancelled = true
  if (idleHandle != null && typeof window.cancelIdleCallback === 'function') {
    window.cancelIdleCallback(idleHandle)
    idleHandle = null
  }
  if (connectTimer) {
    clearTimeout(connectTimer)
    connectTimer = null
  }
  if (historyTimeout) {
    clearTimeout(historyTimeout)
    historyTimeout = null
  }
  socket?.disconnect()
  socket = null
})
</script>
