<template>
  <div class="call-controls relative px-2 md:px-0">
    <div class="flex items-center justify-center gap-4">
      <!-- Microphone Toggle -->
      <button
        type="button"
        :class="[audioButtonClass, '!h-12 !w-12 !p-0 md:!h-14 md:!w-14']"
        :title="isAudioEnabled ? 'Mute microphone' : 'Unmute microphone'"
        @click="$emit('toggle-audio')"
      >
        <Mic v-if="isAudioEnabled" class="h-6 w-6" />
        <MicOff v-else class="h-6 w-6" />
      </button>

      <!-- Video Toggle (only for video calls) -->
      <button
        v-if="callType === 'video'"
        type="button"
        :class="[videoButtonClass, '!h-12 !w-12 !p-0 md:!h-14 md:!w-14']"
        :title="isVideoEnabled ? 'Turn off camera' : 'Turn on camera'"
        @click="$emit('toggle-video')"
      >
        <Video v-if="isVideoEnabled" class="h-6 w-6" />
        <VideoOff v-else class="h-6 w-6" />
      </button>

      <!-- End Call -->
      <button
        type="button"
        class="ui-btn--delete !h-12 !w-12 !p-0 md:!h-14 md:!w-14"
        title="End call"
        @click="$emit('end-call')"
      >
        <PhoneOff class="h-6 w-6" />
      </button>

      <!-- Additional Controls (when connected) -->
      <template v-if="isConnected">
        <!-- Screen Share (desktop only) -->
        <button
          v-if="callType === 'video' && !isMobile"
          type="button"
          :class="[screenShareButtonClass, '!h-12 !w-12 !p-0 md:!h-14 md:!w-14']"
          title="Share screen"
          @click="toggleScreenShare"
        >
          <Monitor v-if="!isScreenSharing" class="h-6 w-6" />
          <MonitorOff v-else class="h-6 w-6" />
        </button>

        <!-- More Options -->
        <button
          type="button"
          class="ui-btn--neutral-muted !h-12 !w-12 !p-0 md:!h-14 md:!w-14"
          title="More options"
          @click="showMoreOptions = !showMoreOptions"
        >
          <MoreVertical class="h-6 w-6" />
        </button>
      </template>
    </div>

    <!-- More Options Dropdown -->
    <div
      v-if="showMoreOptions"
      class="absolute bottom-20 left-1/2 -translate-x-1/2 bg-gray-800 rounded-lg shadow-lg p-2 min-w-40 sm:min-w-48 z-10"
    >
      <button
        v-if="callType === 'video'"
        type="button"
        class="w-full text-left px-4 py-2 text-sm text-white hover:bg-gray-700 rounded flex items-center gap-2"
        @click="toggleCamera"
      >
        <Camera class="h-4 w-4" />
        <span>Switch Camera</span>
      </button>
      <button
        type="button"
        class="w-full text-left px-4 py-2 text-sm text-white hover:bg-gray-700 rounded flex items-center gap-2"
        @click="toggleSpeaker"
      >
        <Speaker class="h-4 w-4" />
        <span>{{ isSpeakerOn ? 'Mute Speaker' : 'Unmute Speaker' }}</span>
      </button>
      <button
        type="button"
        class="w-full text-left px-4 py-2 text-sm text-white hover:bg-gray-700 rounded flex items-center gap-2"
        @click="toggleFullscreen"
      >
        <Maximize class="h-4 w-4" />
        <span>Toggle Fullscreen</span>
      </button>
      <hr class="my-2 border-gray-700" />
      <button
        type="button"
        class="w-full text-left px-4 py-2 text-sm text-red-400 hover:bg-gray-700 rounded flex items-center gap-2"
        @click="reportIssue"
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
  Flag,
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
const audioButtonClass = computed(() =>
  props.isAudioEnabled ? 'ui-btn--neutral-muted' : 'ui-btn--delete'
)

const videoButtonClass = computed(() =>
  props.isVideoEnabled ? 'ui-btn--neutral-muted' : 'ui-btn--delete'
)

const screenShareButtonClass = computed(() =>
  isScreenSharing.value ? 'ui-btn--create' : 'ui-btn--neutral-muted'
)

// Methods
const toggleScreenShare = async () => {
  try {
    if (!isScreenSharing.value) {
      // Start screen share
      const screenStream = await navigator.mediaDevices.getDisplayMedia({
        video: true,
        audio: true,
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
  isMobile.value = /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
    navigator.userAgent
  )
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
.call-controls button {
  transition: transform 0.2s ease-in-out;
}

.call-controls button:hover:not(:disabled) {
  transform: scale(1.05);
}

.call-controls button:active:not(:disabled) {
  transform: scale(0.95);
}
</style>
