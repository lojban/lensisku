<template>
  <div class="chat-page h-full flex flex-col bg-gray-50">
    <!-- Chat Header -->
    <header class="bg-white border-b border-gray-200 px-4 py-3 sm:px-6">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <!-- Back button on mobile -->
          <button
            class="sm:hidden p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-full"
            @click="$router.back()"
          >
            <ArrowLeft class="h-5 w-5" />
          </button>

          <!-- Thread info -->
          <div class="flex items-center space-x-3">
            <div class="relative">
              <div
                class="h-10 w-10 rounded-full bg-gradient-to-br from-blue-400 to-blue-600 flex items-center justify-center text-white font-medium"
              >
                {{ getThreadInitials() }}
              </div>
              <!-- Online indicator for direct messages -->
              <div
                v-if="thread?.thread_type === 'direct' && isUserOnline(getOtherParticipantId())"
                class="absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full border-2 border-white bg-green-400"
              />
            </div>
            <div>
              <h2 class="text-lg font-semibold text-gray-900">
                {{ getThreadDisplayName() }}
              </h2>
              <p class="text-sm text-gray-500">
                {{ getThreadSubtitle() }}
              </p>
            </div>
          </div>
        </div>

        <!-- Header actions -->
        <div class="flex items-center space-x-2">
          <button
            class="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-full transition-colors"
            title="Start video call"
            @click="startCall"
          >
            <Video class="h-5 w-5" />
          </button>
          <button
            class="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-full transition-colors"
            title="Thread info"
            @click="showThreadInfo = true"
          >
            <Info class="h-5 w-5" />
          </button>
        </div>
      </div>
    </header>

    <!-- Messages Area -->
    <main class="flex-1 overflow-hidden flex flex-col">
      <!-- Loading state -->
      <div v-if="isLoading" class="flex items-center justify-center h-full">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>

      <!-- Messages list -->
      <div
        v-else-if="messages.length > 0"
        ref="messagesContainer"
        class="flex-1 overflow-y-auto px-4 py-4"
      >
        <MessageList
          :messages="messages"
          :current-user-id="currentUserId as number"
          @message-click="handleMessageClick"
        />

        <!-- Typing indicator -->
        <div v-if="typingText" class="flex items-center space-x-2 px-4 py-2 text-sm text-gray-500">
          <div class="flex space-x-1">
            <div
              class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
              style="animation-delay: 0ms"
            ></div>
            <div
              class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
              style="animation-delay: 150ms"
            ></div>
            <div
              class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
              style="animation-delay: 300ms"
            ></div>
          </div>
          <span>{{ typingText }}</span>
        </div>
      </div>

      <!-- Empty state -->
      <div v-else class="flex-1 flex items-center justify-center text-center px-4">
        <div>
          <MessageCircle class="h-12 w-12 text-gray-400 mx-auto mb-4" />
          <h3 class="text-lg font-medium text-gray-900 mb-2">No messages yet</h3>
          <p class="text-gray-500">Start the conversation with a message</p>
        </div>
      </div>

      <!-- Message Input -->
      <div class="bg-white border-t border-gray-200 px-4 py-3">
        <MessageInput
          :thread-id="threadId"
          :disabled="!thread"
          @message-sent="handleMessageSent"
          @typing-start="handleTypingStart"
          @typing-stop="handleTypingStop"
        />
      </div>
    </main>

    <!-- Thread Info Sidebar -->
    <ThreadInfoSidebar
      v-if="showThreadInfo && thread"
      :thread="thread"
      @close="showThreadInfo = false"
      @participant-add="handleParticipantAdd"
      @participant-remove="handleParticipantRemove"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, Video, Info, MessageCircle } from 'lucide-vue-next'
import { useAuth } from '@/composables/useAuth'
import { getThread, getMessages } from '@/services/messaging/messagingApi'
import { webSocketService } from '@/services/messaging/WebSocketService'
import { usePresence } from '@/services/messaging/PresenceService'
import { useTyping } from '@/services/messaging/TypingService'
import MessageList from '@/components/messaging/chat/MessageList.vue'
import MessageInput from '@/components/messaging/chat/MessageInput.vue'
import ThreadInfoSidebar from '@/components/messaging/chat/ThreadInfoSidebar.vue'
import type { Thread, Message } from '@/types/messaging'

const route = useRoute()
const router = useRouter()
const auth = useAuth()
const { isUserOnline } = usePresence()
const { getTypingText } = useTyping()

// Reactive state
const thread = ref<Thread | null>(null)
const messages = ref<Message[]>([])
const isLoading = ref(true)
const showThreadInfo = ref(false)
const messagesContainer = ref<HTMLElement>()
const currentUserId = computed(() => auth.state.username as unknown)

// Computed properties
const threadId = computed(() => parseInt(route.params.threadId as string))
const typingText = computed(() => {
  if (!thread.value) return ''
  return getTypingText(thread.value.thread_id, currentUserId.value)
})

// Methods
const loadThread = async () => {
  try {
    const response = await getThread(threadId.value)
    thread.value = response.thread
  } catch (error) {
    console.error('Failed to load thread:', error)
    router.push('/messages')
  }
}

const loadMessages = async () => {
  try {
    isLoading.value = true
    const response = await getMessages(threadId.value)
    messages.value = response.messages
  } catch (error) {
    console.error('Failed to load messages:', error)
  } finally {
    isLoading.value = false
  }
}

const getThreadDisplayName = (): string => {
  if (!thread.value) return 'Unknown'

  if (thread.value.thread_name) {
    return thread.value.thread_name
  }

  if (thread.value.thread_type === 'direct' && thread.value.participants) {
    const otherParticipant = thread.value.participants.find(
      (p) => p.user_id !== currentUserId.value
    )
    return otherParticipant?.username || 'Unknown User'
  }

  return thread.value.thread_type === 'group' ? 'Group Chat' : 'Unknown'
}

const getThreadInitials = (): string => {
  const name = getThreadDisplayName()

  if (thread.value?.thread_type === 'group') {
    return name
      .split(' ')
      .map((word) => word[0])
      .join('')
      .toUpperCase()
      .slice(0, 2)
  }

  return name[0]?.toUpperCase() || '?'
}

const getOtherParticipantId = (): number => {
  if (!thread.value || thread.value.thread_type !== 'direct') return 0

  const otherParticipant = thread.value.participants?.find((p) => p.user_id !== currentUserId.value)
  return otherParticipant?.user_id || 0
}

const getThreadSubtitle = (): string => {
  if (!thread.value) return ''

  if (thread.value.thread_type === 'direct') {
    const otherParticipant = thread.value.participants?.find(
      (p) => p.user_id !== currentUserId.value
    )
    if (otherParticipant) {
      return isUserOnline(otherParticipant.user_id) ? 'Online' : 'Offline'
    }
  }

  return `${thread.value.participant_count} members`
}

const scrollToBottom = async () => {
  await nextTick()
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

const handleMessageClick = (message: Message) => {
  // Handle message click (e.g., reply, edit, delete)
  console.log('Message clicked:', message)
}

const handleMessageSent = (message: Message) => {
  messages.value.push(message)
  scrollToBottom()
}

const handleTypingStart = () => {
  if (thread.value) {
    useTyping().sendTypingIndicator(thread.value.thread_id, true)
  }
}

const handleTypingStop = () => {
  if (thread.value) {
    useTyping().sendTypingIndicator(thread.value.thread_id, false)
  }
}

const startCall = () => {
  if (thread.value) {
    router.push(`/call/new?threadId=${thread.value.thread_id}`)
  }
}

const handleParticipantAdd = (userId: number) => {
  // Handle participant addition
  console.log('Add participant:', userId)
}

const handleParticipantRemove = (userId: number) => {
  // Handle participant removal
  console.log('Remove participant:', userId)
}

// WebSocket event handlers
const handleNewMessage = (message: Message) => {
  if (message.thread_id === threadId.value) {
    messages.value.push(message)
    scrollToBottom()

    // Mark message as read
    webSocketService.markMessageRead(message.message_id)
  }
}

const handleThreadUpdate = () => {
  loadThread()
}

// Watch for route changes
watch(
  () => route.params.threadId,
  (newThreadId) => {
    if (newThreadId) {
      loadThread()
      loadMessages()
    }
  }
)

// Lifecycle
onMounted(async () => {
  if (!auth.state.isLoggedIn) {
    router.push('/login')
    return
  }

  await loadThread()
  await loadMessages()
  await scrollToBottom()

  // Set up WebSocket listeners
  webSocketService.on('message:new', handleNewMessage)
  webSocketService.on('thread:updated', handleThreadUpdate)

  // Connect to WebSocket for this thread
  try {
    await webSocketService.connect(threadId.value)
  } catch (error) {
    console.warn('Failed to connect WebSocket:', error)
  }
})

onUnmounted(() => {
  // Clean up WebSocket listeners
  webSocketService.off('message:new', handleNewMessage)
  webSocketService.off('thread:updated', handleThreadUpdate)
})
</script>

<style scoped>
.chat-page {
  @apply min-h-screen;
}

@media (max-width: 640px) {
  .chat-page {
    @apply h-screen;
  }
}

/* Typing indicator animation */
@keyframes bounce {
  0%,
  80%,
  100% {
    transform: scale(0);
  }
  40% {
    transform: scale(1);
  }
}

.animate-bounce {
  animation: bounce 1.4s infinite ease-in-out both;
}
</style>
