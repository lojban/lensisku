<template>
  <div class="call-controls">
    <div class="flex items-center justify-center space-x-4">
      <!-- Microphone Toggle -->
      <button
        @click="$emit('toggle-audio')"
        :class="audioButtonClass"
        class="p-4 rounded-full transition-all duration-200 focus:outline-none focus:ring-4"
        :title="isAudioEnabled ? 'Mute microphone' : 'Unmute microphone'"
      >
        <Mic v-if="isAudioEnabled" class="h-6 w-6" />
        <MicOff v-else class="h-6 w-6" />
      </button>

      <!-- Video Toggle (only for video calls) -->
      <button
        v-if="callType === 'video'"
        @click="$emit('toggle-video')"
        :class="videoButtonClass"
        class="p-4 rounded-full transition-all duration-200 focus:outline-none focus:ring-4"
        :title="isVideoEnabled ? 'Turn off camera' : 'Turn on camera'"
      >
        <Video v-if="isVideoEnabled" class="h-6 w-6" />
        <VideoOff v-else class="h-6 w-6" />
      </button>

      <!-- End Call -->
      <button
        @click="$emit('end-call')"
        class="p-4 bg-red-600 hover:bg-red-700 rounded-full transition-all duration-200 focus:outline-none focus:ring-4 focus:ring-red-300"
        title="End call"
      >
        <PhoneOff class="h-6 w-6" />
      </button>

      <!-- Additional Controls (when connected) -->
      <template v-if="isConnected">
        <!-- Screen Share (desktop only) -->
        <button
          v-if="callType === 'video' && !isMobile"
          @click="toggleScreenShare"
          :class="screenShareButtonClass"
          class="p-4 rounded-full transition-all duration-200 focus:outline-none focus:ring-4"
          title="Share screen"
        >
          <Monitor v-if="!isScreenSharing" class="h-6 w-6" />
          <MonitorOff v-else class="h-6 w-6" />
        </button>

        <!-- More Options -->
        <button
          @click="showMoreOptions = !showMoreOptions"
          class="p-4 bg-gray-600 hover:bg-gray-700 rounded-full transition-all duration-200 focus:outline-none focus:ring-4 focus:ring-gray-300"
          title="More options"
        >
          <MoreVertical class="h-6 w-6" />
        </button>
      </template>
    </div>

    <!-- More Options Dropdown -->
    <div
      v-if="showMoreOptions"
      class="absolute bottom-20 left-1/2 transform -translate-x-1/2 bg-gray-800 rounded-lg shadow-lg p-2 min-w-48"
    >
      <button
        v-if="callType === 'video'"
        @click="toggleCamera"
        class="w-full text-left px-4 py-2 text-sm hover:bg-gray-700 rounded flex items-center space-x-2"
      >
        <Camera class="h-4 w-4" />
        <span>Switch Camera</span>
      </button>
      <button
        @click="toggleSpeaker"
        class="w-full text-left px-4 py-2 text-sm hover:bg-gray-700 rounded flex items-center space-x-2"
      >
        <Speaker class="h-4 w-4" />
        <span>{{ isSpeakerOn ? 'Mute Speaker' : 'Unmute Speaker' }}</span>
      </button>
      <button
        @click="toggleFullscreen"
        class="w-full text-left px-4 py-2 text-sm hover:bg-gray-700 rounded flex items-center space-x-2"
      >
        <Maximize class="h-4 w-4" />
        <span>Toggle Fullscreen</span>
      </button>
      <hr class="my-2 border-gray-700">
      <button
        @click="reportIssue"
        class="w-full text-left px-4 py-2 text-sm hover:bg-gray-700 rounded flex items-center space-x-2 text-red-400"
      >
        <Flag class="h-4 w-4" />
        <span>Report Issue</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { 
  Mic, 
  MicOff, 
  Video, 
  VideoOff, 
  PhoneOff, 
  Monitor, 
  MonitorOff, 
  MoreVertical,
  Camera,
  Speaker,
  Maximize,
  Flag
} from 'lucide-vue-next'

interface Props {
  isAudioEnabled: boolean
  isVideoEnabled: boolean
  isConnected: boolean
  callType: 'audio' | 'video'
}

const props = defineProps<Props>()

defineEmits<{
  'toggle-audio': []
  'toggle-video': []
  'end-call': []
}>()

// Reactive state
const showMoreOptions = ref(false)
const isScreenSharing = ref(false)
const isSpeakerOn = ref(true)
const isMobile = ref(false)

// Computed properties
const audioButtonClass = computed(() => {
  return props.isAudioEnabled 
    ? 'bg-gray-600 hover:bg-gray-700 focus:ring-gray-300' 
    : 'bg-red-600 hover:bg-red-700 focus:ring-red-300'
})

const videoButtonClass = computed(() => {
  return props.isVideoEnabled 
    ? 'bg-gray-600 hover:bg-gray-700 focus:ring-gray-300' 
    : 'bg-red-600 hover:bg-red-700 focus:ring-red-300'
})

const screenShareButtonClass = computed(() => {
  return isScreenSharing.value 
    ? 'bg-blue-600 hover:bg-blue-700 focus:ring-blue-300' 
    : 'bg-gray-600 hover:bg-gray-700 focus:ring-gray-300'
})

// Methods
const toggleScreenShare = async () => {
  try {
    if (!isScreenSharing.value) {
      // Start screen share
      const screenStream = await navigator.mediaDevices.getDisplayMedia({
        video: true,
        audio: true
      })
      
      // Add screen share tracks to peer connection
      // This would integrate with the WebRTC service
      
      screenStream.getVideoTracks()[0].addEventListener('ended', () => {
        isScreenSharing.value = false
      })
      
      isScreenSharing.value = true
    } else {
      // Stop screen share
      isScreenSharing.value = false
    }
  } catch (error) {
    console.error('Failed to toggle screen share:', error)
  }
}

const toggleCamera = () => {
  // Switch between front/back camera on mobile
  // This would integrate with the WebRTC service
  console.log('Toggle camera')
}

const toggleSpeaker = () => {
  isSpeakerOn.value = !isSpeakerOn.value
  // This would control audio output
}

const toggleFullscreen = () => {
  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen()
  } else {
    document.exitFullscreen()
  }
}

const reportIssue = () => {
  // Open issue reporting modal or navigate to help
  console.log('Report call issue')
  showMoreOptions.value = false
}

// Close dropdown when clicking outside
const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as Element
  if (!target.closest('.call-controls')) {
    showMoreOptions.value = false
  }
}

// Check if mobile
const checkMobile = () => {
  isMobile.value = /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)
}

// Lifecycle
onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  checkMobile()
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
.call-controls {
  @apply relative;
}

/* Button hover effects */
button {
  @apply transform transition-transform duration-200;
}

button:hover {
  @apply scale-105;
}

button:active {
  @apply scale-95;
}

/* Dropdown animation */
.absolute {
  @apply transition-all duration-200 ease-out;
}

/* Focus ring styles */
.focus\:ring-4:focus {
  @apply ring-4 ring-opacity-50;
}

/* Mobile optimizations */
@media (max-width: 640px) {
  .call-controls {
    @apply px-2;
  }
  
  button {
    @apply p-3;
  }
  
  .min-w-48 {
    @apply min-w-40;
  }
}

/* Dark mode specific styles */
.bg-gray-800 {
  @apply shadow-2xl;
}

/* Animation for screen share indicator */
@keyframes pulse-blue {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(59, 130, 246, 0.7);
  }
  70% {
    box-shadow: 0 0 0 10px rgba(59, 130, 246, 0);
  }
}

.bg-blue-600 {
  animation: pulse-blue 2s infinite;
}
</style>
