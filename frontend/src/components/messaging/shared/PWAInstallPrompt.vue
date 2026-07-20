<template>
  <div
    v-if="showPrompt && canInstallPWA"
    class="card-base card-elevated fixed bottom-4 left-4 right-4 z-50 sm:left-auto sm:right-4 sm:w-96 transform transition-all duration-300"
    :class="{ 'translate-y-full opacity-0': !isVisible, 'translate-y-0 opacity-100': isVisible }"
  >
    <div class="flex items-start gap-3 p-4">
      <div class="shrink-0">
        <div class="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
          <Download class="h-6 w-6 text-blue-600" />
        </div>
      </div>

      <div class="flex-1 min-w-0">
        <h3 class="text-lg font-medium text-gray-900 mb-1">Install Our App</h3>
        <p class="card-description mb-3">
          Get the best experience with our app. Install it on your device for quick access and
          offline features.
        </p>

        <div class="flex items-center gap-2">
          <button :disabled="isInstalling" class="ui-btn--create flex-1" @click="installApp">
            <span
              v-if="isInstalling"
              class="inline-block h-4 w-4 animate-spin rounded-full border-b-2 border-current"
              aria-hidden="true"
            />
            {{ isInstalling ? 'Installing...' : 'Install App' }}
          </button>

          <button type="button" class="ui-btn--neutral-muted" @click="dismissPrompt">
            Not now
          </button>
        </div>
      </div>

      <button type="button" class="icon-btn-ghost shrink-0" @click="dismissPrompt">
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
