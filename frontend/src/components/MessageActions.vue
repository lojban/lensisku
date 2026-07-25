<template>
  <div class="flex gap-3">
    <Button variant="back" @click="goBack"><ArrowLeft class="h-5 w-5" /></Button>
    <Button variant="market" @click="viewThread">
      {{ t('components.messageActions.viewThread') }}
    </Button>
    <Button
      v-if="showSpamButton"
      variant="warning"
      :class="currentUserVotedSpam ? 'ui-btn--warning' : 'ui-btn--empty'"
      @click="$emit('toggle-spam-vote')"
    >
      {{
        currentUserVotedSpam
          ? t('components.messageDetail.unlabelAsSpam', { count: spamVoteCount })
          : t('components.messageDetail.labelAsSpam', { count: spamVoteCount })
      }}
    </Button>
  </div>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { ArrowLeft } from 'lucide-vue-next'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const goBack = () => {
  router.back()
}

const props = defineProps({
  cleanedSubject: {
    type: String,
    default: '',
  },
  messageId: {
    type: Number,
    default: null,
  },
  spamVoteCount: {
    type: Number,
    default: 0,
  },
  showSpamButton: {
    type: Boolean,
    default: false,
  },
  currentUserVotedSpam: {
    type: Boolean,
    default: false,
  },
})
defineEmits(['toggle-spam-vote'])

const viewThread = () => {
  if (props.cleanedSubject) {
    const currentLocale = route.path.split('/')[1] || 'en' // Default to 'en' if locale is missing
    const routeName = `ThreadView-${currentLocale}`
    router.push({
      name: routeName,
      params: {
        subject: props.cleanedSubject,
      },
    })
  }
}
</script>
