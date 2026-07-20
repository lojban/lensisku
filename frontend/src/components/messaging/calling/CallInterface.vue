<template>
  <div class="call-interface h-full min-h-screen flex flex-col bg-gray-900 text-white">
    <!-- Call Header -->
    <header class="flex items-center justify-between p-4 bg-gray-800">
      <div class="flex items-center gap-3">
        <button
          type="button"
          class="icon-btn-ghost !text-white hover:!bg-gray-700"
          @click="$emit('call-ended')"
        >
          <ArrowLeft class="h-5 w-5" />
        </button>
        <div>
          <h2 class="text-lg font-semibold">{{ callTitle }}</h2>
          <p class="text-sm text-gray-400">{{ callStatusText }}</p>
        </div>
      </div>
      <div class="text-sm text-gray-400">
        {{ formatDuration(duration()) }}
      </div>
    </header>

    <!-- Main Call Area -->
    <main class="flex-1 relative overflow-hidden">
      <!-- Local Video -->
      <div
        v-if="localStream && callType === 'video'"
        class="absolute top-4 right-4 w-32 h-24 bg-gray-800 rounded-lg shadow-lg overflow-hidden z-10"
      >
        <video
          ref="localVideoRef"
          :srcObject="localStream"
          autoplay
          muted
          class="w-full h-full object-cover bg-gray-800"
        />
        <div class="absolute bottom-2 left-2 text-xs bg-black/50 px-1 py-0.5 rounded">You</div>
      </div>

      <!-- Remote Video(s) -->
      <div class="h-full flex items-center justify-center">
        <div v-if="remoteStreams.size === 0 && isConnected" class="text-center">
          <div class="animate-pulse">
            <User class="h-16 w-16 mx-auto mb-4 text-gray-600" />
            <p class="text-gray-400">Waiting for participant...</p>
          </div>
        </div>

        <div
          v-else-if="remoteStreams.size > 0"
          class="w-full h-full flex items-center justify-center"
        >
          <div
            v-for="[userId, stream] in remoteStreams"
            :key="userId"
            class="relative w-full h-full max-w-4xl"
          >
            <video
              :ref="
                (el) => {
                  if (el) remoteVideoRefs.set(userId, el as HTMLVideoElement)
                }
              "
              :srcObject="stream"
              autoplay
              class="w-full h-full object-contain bg-gray-800"
            />
            <div class="absolute bottom-4 left-4 text-lg font-medium">User {{ userId }}</div>
          </div>
        </div>

        <!-- Audio-only call interface -->
        <div v-else-if="callType === 'audio' && isConnected" class="text-center">
          <div
            class="h-32 w-32 rounded-full bg-gradient-to-br from-blue-400 to-blue-600 flex items-center justify-center mx-auto mb-4"
          >
            <User class="h-16 w-16 text-white" />
          </div>
          <h3 class="text-xl font-semibold mb-2">Audio Call</h3>
          <p class="text-gray-400">{{ callStatusText }}</p>
        </div>

        <!-- Ringing state -->
        <div v-else-if="isRinging" class="text-center">
          <div class="animate-pulse">
            <Phone class="h-16 w-16 mx-auto mb-4 text-green-400" />
            <h3 class="text-xl font-semibold mb-2">Ringing...</h3>
            <p class="text-gray-400">Waiting for answer</p>
          </div>
        </div>
      </div>
    </main>

    <!-- Call Controls -->
    <footer class="bg-gray-800 p-4">
      <CallControls
        :is-audio-enabled="isAudioEnabled"
        :is-video-enabled="isVideoEnabled"
        :is-connected="isConnected()"
        :call-type="callType"
        @toggle-audio="toggleAudio"
        @toggle-video="toggleVideo"
        @end-call="$emit('call-ended')"
      />
    </footer>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { ArrowLeft, User, Phone } from 'lucide-vue-next'
import { useWebRTC } from '@/services/messaging/WebRTCService'
import CallControls from './CallControls.vue'

interface Props {
  callId: string
}

const _props = defineProps<Props>()

defineEmits<{
  'call-ended': []
}>()

const {
  callState,
  isConnected,
  isRinging,
  duration,
  localStream,
  remoteStreamMap: _remoteStreamMap,
  toggleAudio,
  toggleVideo,
  endCall: _endCall,
} = useWebRTC()

// Reactive state
const localVideoRef = ref<HTMLVideoElement>()
const remoteVideoRefs = ref<Map<number, HTMLVideoElement>>(new Map())
const isAudioEnabled = ref(true)
const isVideoEnabled = ref(true)

// Computed properties
const callType = computed(() => callState.call_type)
const callStatus = computed(() => callState.call_status)
const remoteStreams = computed(() => callState.remote_streams)

const callTitle = computed(() => {
  if (remoteStreams.value.size > 0) {
    return `Call with ${remoteStreams.value.size} participant${remoteStreams.value.size > 1 ? 's' : ''}`
  }
  return 'Starting Call...'
})

const callStatusText = computed(() => {
  switch (callStatus.value) {
    case 'ringing':
      return 'Ringing...'
    case 'connected':
      return 'Connected'
    case 'ended':
      return 'Call ended'
    default:
      return 'Connecting...'
  }
})

// Methods
const formatDuration = (seconds: number): string => {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60

  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
  }
  return `${minutes}:${secs.toString().padStart(2, '0')}`
}

const _handleToggleAudio = () => {
  isAudioEnabled.value = !isAudioEnabled.value
  toggleAudio(isAudioEnabled.value)
}

const _handleToggleVideo = () => {
  isVideoEnabled.value = !isVideoEnabled.value
  toggleVideo(isVideoEnabled.value)
}

// Watch for remote streams changes and set up video elements
watch(
  remoteStreams,
  (newStreams) => {
    newStreams.forEach((stream, userId) => {
      const videoElement = remoteVideoRefs.value.get(userId)
      if (videoElement) {
        videoElement.srcObject = stream
      }
    })
  },
  { deep: true }
)

// Watch for local stream changes
watch(localStream, (newStream) => {
  if (newStream && localVideoRef.value) {
    localVideoRef.value.srcObject = newStream
  }
})

// Lifecycle
onMounted(() => {
  // Initialize call if needed
  // This would be called based on props.callId
})

onUnmounted(() => {
  // Clean up video streams
  if (localVideoRef.value) {
    localVideoRef.value.srcObject = null
  }

  remoteVideoRefs.value.forEach((video) => {
    video.srcObject = null
  })
})
</script>
