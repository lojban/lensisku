<template>
  <article v-if="messageStack.length" class="live-chat-card" :aria-label="t('footer.liveChat')">
    <div class="live-chat-card__social">
      <SocialLinks class="live-chat-card__social-scroll" :buttons="true" />
      <span class="live-chat-card__label">{{ t('footer.liveChat') }}</span>
    </div>
    <div ref="listEl" class="live-chat-card__messages">
      <div
        v-for="(msg, index) in messageStack"
        :key="`${msg.w}-${index}-${msg.d.slice(0, 24)}`"
        class="live-chat-card__row"
      >
        <span class="live-chat-card__author">{{ msg.w }}:</span>
        <span class="live-chat-card__text">{{ msg.d }}</span>
      </div>
    </div>
  </article>
</template>

<script setup lang="ts">
import { io } from 'socket.io-client'
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import SocialLinks from '@/components/SocialLinks.vue'

type ChatMessage = { d: string; s?: string; w: string }

const { t } = useI18n()

const messageStack = ref<ChatMessage[]>([])
const listEl = ref<HTMLElement | null>(null)
const socketConnected = ref(false)

let socket: ReturnType<typeof io> | null = null

const sanitize = (text: string) =>
  text
    .replace(/<[^>]*>?/gm, '')
    .replace(/\s+/g, ' ')
    .trim()

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

onMounted(() => {
  socket = io('wss://jbotcan.org:9091', {
    transports: ['polling', 'websocket'],
  })

  socket.on('connect', () => {
    socketConnected.value = true
  })

  socket.on('connect_error', () => {
    console.error('1chat connection error')
  })

  socket.on(
    'sentFrom',
    (data: { data?: { chunk?: string; channelId?: string; author?: string } }) => {
      if (!socketConnected.value || !data?.data) return
      const i = data.data
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
    if (!Array.isArray(data)) return
    if (socketConnected.value) {
      pushMessages(
        data.slice(-20).map((m) => ({
          d: m.chunk ?? '',
          s: m.channelId,
          w: m.author ?? '',
        }))
      )
    }
  })
})

onUnmounted(() => {
  socket?.disconnect()
  socket = null
})
</script>
