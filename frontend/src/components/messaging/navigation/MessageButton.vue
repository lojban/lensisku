<template>
  <div class="inline-block">
    <Button
      :disabled="isLoading || !canMessage"
      :loading="isLoading"
      :variant="buttonVariant"
      :size="buttonSize"
      :class="[{ 'w-full': fullWidth }, size === 'sm' ? '!text-xs !px-3 !py-1.5' : '']"
      @click="handleMessageClick"
    >
      <template #icon>
        <MessageSquare class="h-4 w-4" />
      </template>
      <span>{{ buttonText }}</span>
    </Button>

    <!-- Error message -->
    <div v-if="error" class="mt-2 text-sm text-red-600">
      {{ error }}
    </div>

    <!-- Success message -->
    <div v-if="success" class="mt-2 text-sm text-green-600">
      {{ success }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { MessageSquare } from 'lucide-vue-next'
import { Button } from '@packages/ui'
import { useAuth } from '@/composables/useAuth'
import { createThread } from '@/services/messaging/messagingApi'
import type { CreateThreadRequest } from '@/types/messaging'

interface Props {
  userId: number
  username: string
  variant?: 'primary' | 'secondary' | 'outline'
  size?: 'sm' | 'md' | 'lg'
  fullWidth?: boolean
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  fullWidth: false,
  disabled: false,
})

const router = useRouter()
const auth = useAuth()

// Reactive state
const isLoading = ref(false)
const error = ref<string | null>(null)
const success = ref<string | null>(null)

// Computed properties
const canMessage = computed(() => {
  return (
    auth.state.isLoggedIn && props.userId !== (auth.state.username as unknown) && !props.disabled
  )
})

const buttonText = computed(() => {
  if (!auth.state.isLoggedIn) {
    return 'Login to Message'
  }
  return 'Message'
})

const buttonVariant = computed(() => {
  switch (props.variant) {
    case 'secondary':
      return 'neutral-muted'
    case 'outline':
      return 'link'
    default:
      return 'primary'
  }
})

const buttonSize = computed(() => (props.size === 'lg' ? 'lg' : 'md'))

// Methods
const handleMessageClick = async () => {
  if (!canMessage.value) {
    if (!auth.state.isLoggedIn) {
      // Redirect to login page with return URL
      const returnUrl = router.currentRoute.value.fullPath
      router.push(`/login?return=${encodeURIComponent(returnUrl)}`)
      return
    }
    return
  }

  isLoading.value = true
  error.value = null
  success.value = null

  try {
    // Check if a thread already exists with this user
    // For now, we'll create a new thread, but in a real implementation
    // you might want to check for existing threads first

    const request: CreateThreadRequest = {
      thread_type: 'direct',
      participant_ids: [props.userId],
    }

    const response = await createThread(request)
    const thread = (response as { thread: unknown }).thread

    // Navigate to the thread
    router.push(`/messages/${(thread as { thread_id: number }).thread_id}`)

    success.value = 'Thread created successfully!'

    // Clear success message after 3 seconds
    setTimeout(() => {
      success.value = null
    }, 3000)
  } catch (err: unknown) {
    console.error('Failed to create thread:', err)
    error.value =
      (err as { response?: { data?: { message?: string } } })?.response?.data?.message ||
      'Failed to start conversation'

    // Clear error message after 5 seconds
    setTimeout(() => {
      error.value = null
    }, 5000)
  } finally {
    isLoading.value = false
  }
}

// Clear messages when component is unmounted
const clearMessages = () => {
  error.value = null
  success.value = null
}

// Expose method to clear messages (for parent components)
defineExpose({
  clearMessages,
})
</script>
