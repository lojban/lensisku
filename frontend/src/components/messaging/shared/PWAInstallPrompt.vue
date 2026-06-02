<template>
  <div
    v-if="showPrompt && canInstallPWA"
    class="fixed bottom-4 left-4 right-4 z-50 sm:left-auto sm:right-4 sm:w-96 bg-white rounded-lg shadow-lg border border-gray-200 p-4 transform transition-all duration-300"
    :class="{ 'translate-y-full opacity-0': !isVisible, 'translate-y-0 opacity-100': isVisible }"
  >
    <div class="flex items-start space-x-3">
      <div class="flex-shrink-0">
        <div class="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
          <Download class="h-6 w-6 text-blue-600" />
        </div>
      </div>

      <div class="flex-1 min-w-0">
        <h3 class="text-lg font-medium text-gray-900 mb-1">Install Our App</h3>
        <p class="text-sm text-gray-600 mb-3">
          Get the best experience with our app. Install it on your device for quick access and
          offline features.
        </p>

        <div class="flex items-center space-x-2">
          <button
            :disabled="isInstalling"
            class="flex-1 bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            @click="installApp"
          >
            <div v-if="isInstalling" class="flex items-center justify-center">
              <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
              Installing...
            </div>
            <span v-else>Install App</span>
          </button>

          <button
            class="px-3 py-2 text-sm text-gray-600 hover:text-gray-800 transition-colors"
            @click="dismissPrompt"
          >
            Not now
          </button>
        </div>
      </div>

      <button
        class="flex-shrink-0 p-1 text-gray-400 hover:text-gray-600 transition-colors"
        @click="dismissPrompt"
      >
        <X class="h-4 w-4" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Download, X } from 'lucide-vue-next'
import { useNotifications } from '@/services/messaging/NotificationService'

const { canInstallPWA, showInstallPrompt } = useNotifications()

// Reactive state
const showPrompt = ref(false)
const isVisible = ref(false)
const isInstalling = ref(false)

// Methods
const installApp = async () => {
  isInstalling.value = true

  try {
    const installed = await showInstallPrompt()
    if (installed) {
      showPrompt.value = false
      // Show success message
      console.log('App installed successfully!')
    }
  } catch (error) {
    console.error('Failed to install app:', error)
  } finally {
    isInstalling.value = false
  }
}

const dismissPrompt = () => {
  isVisible.value = false
  setTimeout(() => {
    showPrompt.value = false
  }, 300)
}

// Show prompt after a delay
const showInstallPromptAfterDelay = () => {
  setTimeout(() => {
    if (canInstallPWA) {
      showPrompt.value = true
      // Trigger animation
      setTimeout(() => {
        isVisible.value = true
      }, 100)
    }
  }, 5000) // Show after 5 seconds
}

// Lifecycle
onMounted(() => {
  showInstallPromptAfterDelay()
})
</script>

<style scoped>
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

/* Mobile optimizations */
@media (max-width: 640px) {
  .fixed {
    @apply mx-4;
  }
}
</style>
