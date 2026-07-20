<template>
  <div class="call-page h-full flex flex-col bg-gray-900">
    <!-- Call Interface -->
    <CallInterface :call-id="callId" @call-ended="handleCallEnded" />
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import CallInterface from '@/components/messaging/calling/CallInterface.vue'

interface Props {
  callId: string
}

defineProps<Props>()

const router = useRouter()

const handleCallEnded = () => {
  router.push('/messages')
}
</script>

<style scoped>
.call-page {
  @apply min-h-screen;
}

@media (max-width: 640px) {
  .call-page {
    @apply h-screen;
  }
}
</style>
