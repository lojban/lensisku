import { ref, reactive } from 'vue'
import { sendSignal, getPendingSignals, markSignalProcessed, getWebRTCConfig } from '@/services/messaging/messagingApi'
import { webSocketService } from './WebSocketService'
import type { WebRTCConfig, CallState, CallUser, WebRTCSignal, ActiveCall } from '@/types/messaging'

class WebRTCService {
  private peerConnection: RTCPeerConnection | null = null
  private localStreamRef: MediaStream | null = null
  private remoteStreams = reactive(new Map<number, MediaStream>())
  private currentCallId = ref<string | null>(null)
  private isInitiator = ref(false)
  private callStatus = ref<'idle' | 'ringing' | 'connected' | 'ended'>('idle')
  private callType = ref<'audio' | 'video'>('video')
  private participants = reactive(new Map<number, CallUser>())
  private callDuration = ref(0)
  private durationInterval: number | null = null

  // Reactive state
  public callState = reactive<CallState>({
    call_id: '',
    thread_id: 0,
    is_initiator: false,
    local_stream: undefined,
    remote_streams: new Map(),
    participants: [],
    call_status: 'ended',
    call_type: 'video',
    duration: 0
  })

  constructor() {
    this.setupWebSocketListeners()
  }

  private setupWebSocketListeners() {
    webSocketService.on('webrtc:signal', (signal: WebRTCSignal) => {
      this.handleIncomingSignal(signal)
    })

    webSocketService.on('webrtc:call', (callData: any) => {
      this.handleIncomingCall(callData)
    })
  }

  private async handleIncomingSignal(signal: WebRTCSignal) {
    if (!this.peerConnection) return

    try {
      const signalData = JSON.parse(signal.signal_data)
      
      if (signal.signal_type === 'offer') {
        await this.peerConnection.setRemoteDescription(new RTCSessionDescription(signalData))
        const answer = await this.peerConnection.createAnswer()
        await this.peerConnection.setLocalDescription(answer)
        
        // Send answer back
        await sendSignal({
          signal_type: 'answer',
          signal_data: JSON.stringify(answer),
          to_user_id: signal.from_user_id
        })
      } else if (signal.signal_type === 'answer') {
        await this.peerConnection.setRemoteDescription(new RTCSessionDescription(signalData))
      } else if (signal.signal_type === 'ice-candidate') {
        await this.peerConnection.addIceCandidate(new RTCIceCandidate(signalData))
      }
      
      // Mark signal as processed
      await markSignalProcessed(signal.signal_id)
    } catch (error) {
      console.error('Error handling WebRTC signal:', error)
    }
  }

  private handleIncomingCall(callData: any) {
    // Handle incoming call logic
    console.log('Incoming call:', callData)
    // This would trigger a call UI or notification
  }

  public async initializeCall(threadId: number, isVideo: boolean = true): Promise<void> {
    try {
      // Get WebRTC configuration
      const config = await getWebRTCConfig()
      
      // Create peer connection
      this.peerConnection = new RTCPeerConnection({
        iceServers: config.ice_servers
      })

      // Setup peer connection event handlers
      this.setupPeerConnectionHandlers()

      // Get local media stream
      this.localStreamRef = await navigator.mediaDevices.getUserMedia({
        video: isVideo,
        audio: true
      })

      // Add local stream to peer connection
      this.localStreamRef.getTracks().forEach(track => {
        this.peerConnection?.addTrack(track, this.localStreamRef!)
      })

      // Update state
      this.callType.value = isVideo ? 'video' : 'audio'
      this.isInitiator.value = true
      this.callStatus.value = 'ringing'
      
      this.updateCallState()

    } catch (error) {
      console.error('Failed to initialize call:', error)
      throw error
    }
  }

  private setupPeerConnectionHandlers() {
    if (!this.peerConnection) return

    this.peerConnection.onicecandidate = (event) => {
      if (event.candidate) {
        // Send ICE candidate to other party
        this.sendSignalToPeer('ice-candidate', event.candidate)
      }
    }

    this.peerConnection.ontrack = (event) => {
      if (event.streams && event.streams[0]) {
        const stream = event.streams[0]
        // In a real implementation, you'd identify which user the stream belongs to
        // For now, we'll use a placeholder user ID
        const userId = 1 // This would come from the signal or call data
        this.remoteStreams.set(userId, stream)
        this.updateCallState()
      }
    }

    this.peerConnection.onconnectionstatechange = () => {
      if (this.peerConnection) {
        const state = this.peerConnection.connectionState
        if (state === 'connected') {
          this.callStatus.value = 'connected'
          this.startCallTimer()
        } else if (state === 'disconnected' || state === 'failed') {
          this.endCall()
        }
        this.updateCallState()
      }
    }
  }

  private async sendSignalToPeer(type: 'offer' | 'answer' | 'ice-candidate', data: any) {
    try {
      const signalData = type === 'ice-candidate' ? data : JSON.stringify(data)
      // In a real implementation, you'd send this to the specific peer
      await sendSignal({
        signal_type: type,
        signal_data: signalData,
        to_user_id: 1 // This would be the actual peer user ID
      })
    } catch (error) {
      console.error('Failed to send signal:', error)
    }
  }

  public async createOffer(): Promise<void> {
    if (!this.peerConnection) return

    try {
      const offer = await this.peerConnection.createOffer()
      await this.peerConnection.setLocalDescription(offer)
      
      // Send offer to peer
      await this.sendSignalToPeer('offer', offer)
    } catch (error) {
      console.error('Failed to create offer:', error)
    }
  }

  public async acceptCall(threadId: number, callId: string): Promise<void> {
    try {
      await this.initializeCall(threadId, true)
      this.currentCallId.value = callId
      this.isInitiator.value = false
      this.callStatus.value = 'connected'
      this.startCallTimer()
      this.updateCallState()
    } catch (error) {
      console.error('Failed to accept call:', error)
    }
  }

  public rejectCall(callId: string): void {
    // Send reject signal via WebSocket
    webSocketService.sendMessage({
      type: 'call_reject',
      call_id: callId
    })
  }

  public endCall(): void {
    // Stop local stream
    if (this.localStreamRef) {
      this.localStreamRef.getTracks().forEach(track => track.stop())
      this.localStreamRef = null
    }

    // Close peer connection
    if (this.peerConnection) {
      this.peerConnection.close()
      this.peerConnection = null
    }

    // Clear remote streams
    this.remoteStreams.clear()

    // Stop call timer
    if (this.durationInterval) {
      clearInterval(this.durationInterval)
      this.durationInterval = null
    }

    // Reset state
    this.callStatus.value = 'ended'
    this.currentCallId.value = null
    this.callDuration.value = 0
    this.updateCallState()

    // Send call ended signal
    if (this.currentCallId.value) {
      webSocketService.sendMessage({
        type: 'call_ended',
        call_id: this.currentCallId.value
      })
    }
  }

  public toggleAudio(enabled: boolean): void {
    if (this.localStreamRef) {
      this.localStreamRef.getAudioTracks().forEach(track => {
        track.enabled = enabled
      })
    }
  }

  public toggleVideo(enabled: boolean): void {
    if (this.localStreamRef) {
      this.localStreamRef.getVideoTracks().forEach(track => {
        track.enabled = enabled
      })
    }
  }

  private startCallTimer(): void {
    this.durationInterval = setInterval(() => {
      this.callDuration.value++
      this.updateCallState()
    }, 1000)
  }

  private updateCallState(): void {
    this.callState = {
      call_id: this.currentCallId.value || '',
      thread_id: 0, // This would be set from actual call data
      is_initiator: this.isInitiator.value,
      local_stream: this.localStreamRef || undefined,
      remote_streams: new Map(this.remoteStreams),
      participants: Array.from(this.participants.values()),
      call_status: this.callStatus.value,
      call_type: this.callType.value,
      duration: this.callDuration.value
    }
  }

  // Getters for reactive state
  public get isConnected(): boolean {
    return this.callStatus.value === 'connected'
  }

  public get isRinging(): boolean {
    return this.callStatus.value === 'ringing'
  }

  public get isCallActive(): boolean {
    return ['ringing', 'connected'].includes(this.callStatus.value)
  }

  public get duration(): number {
    return this.callDuration.value
  }

  public get localStream(): MediaStream | null {
    return this.localStreamRef
  }

  public get remoteStreamMap(): Map<number, MediaStream> {
    return new Map(this.remoteStreams)
  }
}

// Create singleton instance
export const webRTCService = new WebRTCService()

// Export composable for Vue components
export function useWebRTC() {
  return {
    callState: webRTCService.callState,
    isConnected: () => webRTCService.isConnected,
    isRinging: () => webRTCService.isRinging,
    isCallActive: () => webRTCService.isCallActive,
    duration: () => webRTCService.duration,
    localStream: () => webRTCService.localStream,
    remoteStreamMap: () => webRTCService.remoteStreamMap,
    initializeCall: webRTCService.initializeCall.bind(webRTCService),
    createOffer: webRTCService.createOffer.bind(webRTCService),
    acceptCall: webRTCService.acceptCall.bind(webRTCService),
    rejectCall: webRTCService.rejectCall.bind(webRTCService),
    endCall: webRTCService.endCall.bind(webRTCService),
    toggleAudio: webRTCService.toggleAudio.bind(webRTCService),
    toggleVideo: webRTCService.toggleVideo.bind(webRTCService),
  }
}
