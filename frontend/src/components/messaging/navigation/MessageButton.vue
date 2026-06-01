<template>
  <div class="message-button-container">
    <button
      @click="handleMessageClick"
      :disabled="isLoading || !canMessage"
      class="inline-flex items-center justify-center px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200"
      :class="buttonClass"
    >
      <MessageSquare class="h-4 w-4 mr-2" />
      <span v-if="!isLoading">{{ buttonText }}</span>
      <div v-else class="flex items-center">
        <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
        Loading...
      </div>
    </button>

    <!-- Error message -->
    <div
      v-if="error"
      class="mt-2 text-sm text-red-600"
    >
      {{ error }}
    </div>

    <!-- Success message -->
    <div
      v-if="success"
      class="mt-2 text-sm text-green-600"
    >
      {{ success }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { MessageSquare } from 'lucide-vue-next'
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
  disabled: false
})

const router = useRouter()
const auth = useAuth()

// Reactive state
const isLoading = ref(false)
const error = ref<string | null>(null)
const success = ref<string | null>(null)

// Computed properties
const canMessage = computed(() => {
  return auth.state.isLoggedIn && 
         props.userId !== (auth.state.username as any) && 
         !props.disabled
})

const buttonText = computed(() => {
  if (!auth.state.isLoggedIn) {
    return 'Login to Message'
  }
  return 'Message'
})

const buttonClass = computed(() => {
  const classes = []

  // Size classes
  switch (props.size) {
    case 'sm':
      classes.push('px-3 py-1.5 text-xs')
      break
    case 'lg':
      classes.push('px-6 py-3 text-base')
      break
    default: // md
      classes.push('px-4 py-2 text-sm')
  }

  // Width
  if (props.fullWidth) {
    classes.push('w-full')
  }

  // Variant classes
  switch (props.variant) {
    case 'secondary':
      classes.push('bg-gray-600 hover:bg-gray-700 focus:ring-gray-500')
      break
    case 'outline':
      classes.push('bg-transparent border-blue-600 text-blue-600 hover:bg-blue-50 focus:ring-blue-500')
      break
    default: // primary
      classes.push('bg-blue-600 hover:bg-blue-700 focus:ring-blue-500 text-white')
  }

  return classes.join(' ')
})

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
      participant_ids: [props.userId]
    }

    const response = await createThread(request)
    const thread = response.thread

    // Navigate to the thread
    router.push(`/messages/${thread.thread_id}`)
    
    success.value = 'Thread created successfully!'
    
    // Clear success message after 3 seconds
    setTimeout(() => {
      success.value = null
    }, 3000)

  } catch (err: any) {
    console.error('Failed to create thread:', err)
    error.value = err.response?.data?.message || 'Failed to start conversation'
    
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
  clearMessages
})
</script>

<style scoped>
.message-button-container {
  @apply inline-block;
}

.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
