<template>
  <PageBanner variant="error" :show="showTestDataWarning">
    {{ t('testDataWarning') }}
    <a
      :href="discordChatUrl"
      target="_blank"
      rel="noopener noreferrer"
      class="text-blue-500 hover:text-red-800 underline"
    >
      {{ t('discord') }}
    </a>
  </PageBanner>
  <PageBanner variant="warning" :show="showUnconfirmedWarning">
    <div class="max-w-4xl mx-auto flex items-center justify-center gap-2 flex-wrap">
      <span>{{ t('unconfirmedWarning') }}</span>
      <button
        type="button"
        :disabled="isResendingConfirmation"
        class="text-blue-600 hover:text-blue-800 underline font-medium disabled:opacity-50 disabled:cursor-not-allowed"
        @click="$emit('resend-confirmation')"
      >
        {{ isResendingConfirmation ? t('emailConfirmation.sending') : t('unconfirmedWarningLink') }}
      </button>
      <span v-if="resendConfirmationSuccess" class="text-green-600 text-xs ml-2">
        {{ t('unconfirmedWarningMessageSent') }}
      </span>
    </div>
  </PageBanner>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { PageBanner } from '@packages/ui'

defineProps({
  showTestDataWarning: { type: Boolean, default: false },
  showUnconfirmedWarning: { type: Boolean, default: false },
  discordChatUrl: { type: String, required: true },
  isResendingConfirmation: { type: Boolean, default: false },
  resendConfirmationSuccess: { type: Boolean, default: false },
})

defineEmits<{ 'resend-confirmation': [] }>()

const { t } = useI18n()
</script>
