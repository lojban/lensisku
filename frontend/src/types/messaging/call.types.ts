export interface WebRTCSignal {
  signal_id: number
  from_user_id: number
  to_user_id: number
  signal_type: 'offer' | 'answer' | 'ice-candidate'
  signal_data: string
  created_at: string
  expires_at: string
  is_processed: boolean
}

export interface WebRTCSignalRequest {
  signal_type: 'offer' | 'answer' | 'ice-candidate'
  signal_data: string
  to_user_id: number
}

export interface WebRTCSignalResponse {
  signal: WebRTCSignal
  from_username: string
}

export interface WebRTCConfig {
  ice_servers: Array<{
    urls: string[]
    username?: string
    credential?: string
  }>
}

export interface ActiveCall {
  call_id: string
  thread_id: number
  initiator_id: number
  participant_ids: number[]
  call_type: 'audio' | 'video'
  status: 'ringing' | 'connected' | 'ended'
  started_at: string
  ended_at?: string
}

export interface CallUser {
  user_id: number
  username: string
  is_in_call: boolean
  is_audio_enabled: boolean
  is_video_enabled: boolean
  is_speaking: boolean
}

export interface CallState {
  call_id: string
  thread_id: number
  is_initiator: boolean
  local_stream?: MediaStream
  remote_streams: Map<number, MediaStream>
  participants: CallUser[]
  call_status: 'idle' | 'ringing' | 'connected' | 'ended'
  call_type: 'audio' | 'video'
  duration: number
}
