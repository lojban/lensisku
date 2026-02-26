<template>
  <div class="flex flex-col h-full max-h-[calc(100vh-3rem)]">
    <div class="mb-4">
      <h1 class="text-2xl font-bold text-gray-800">
        {{ $t('assistantChat.title') }}
      </h1>
      <p class="mt-1 text-sm text-gray-600">
        {{ $t('assistantChat.subtitle') }}
      </p>
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
          class="max-w-[80%] rounded-lg px-3 py-2 text-sm whitespace-pre-wrap break-words"
          :class="msg.role === 'user'
            ? 'bg-blue-600 text-white'
            : 'bg-gray-100 text-gray-900'"
        >
          <span v-if="msg.role === 'assistant'" class="block text-[11px] font-semibold text-gray-500 mb-1">
            {{ $t('assistantChat.assistantLabel') }}
          </span>
          <span v-else-if="msg.role === 'user'" class="block text-[11px] font-semibold text-blue-100 mb-1">
            {{ $t('assistantChat.userLabel') }}
          </span>
          <span>{{ msg.content }}</span>
        </div>
      </div>
    </div>

    <form
      class="mt-4 flex flex-col gap-2"
      @submit.prevent="handleSend"
    >
      <textarea
        v-model="input"
        class="input-field min-h-[80px] max-h-40 resize-y"
        :placeholder="$t('assistantChat.placeholder')"
        :disabled="isLoading"
      />

      <div class="flex items-center justify-between gap-3">
        <p
          v-if="error"
          class="text-xs text-red-600"
        >
          {{ error }}
        </p>
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
import { useRoute } from 'vue-router'

import { assistantChat } from '../api'

const { locale, t } = useI18n()
const route = useRoute()

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
  () => messages.value.length,
  () => {
    scrollToBottom()
  }
)

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
    const payload = {
      messages: messages.value.map((m) => ({
        role: m.role,
        content: m.content,
      })),
      locale: locale.value,
    }

    const { data } = await assistantChat(payload)

    messages.value.push({
      role: 'assistant',
      content: data.reply,
    })
  } catch (e) {
    console.error(e)
    error.value = t('assistantChat.error')
  } finally {
    isLoading.value = false
  }
}
</script>
