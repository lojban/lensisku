<template>
  <div class="new-chat-page h-full flex flex-col bg-gray-50">
    <!-- Header -->
    <header class="bg-white border-b border-gray-200 px-4 py-3 sm:px-6">
      <div class="flex items-center space-x-3">
        <button
          @click="$router.back()"
          class="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-full"
        >
          <ArrowLeft class="h-5 w-5" />
        </button>
        <h1 class="text-lg font-semibold text-gray-900">New Conversation</h1>
      </div>
    </header>

    <!-- Content -->
    <main class="flex-1 overflow-hidden">
      <NewChatModal
        :is-full-page="true"
        @close="$router.back()"
        @thread-created="handleThreadCreated"
      />
    </main>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import NewChatModal from '@/components/messaging/chat/NewChatModal.vue'
import type { Thread } from '@/types/messaging'

const router = useRouter()

const handleThreadCreated = (thread: Thread) => {
  router.push(`/messages/${thread.thread_id}`)
}
</script>

<style scoped>
.new-chat-page {
  @apply min-h-screen;
}

@media (max-width: 640px) {
  .new-chat-page {
    @apply h-screen;
  }
}
</style>
