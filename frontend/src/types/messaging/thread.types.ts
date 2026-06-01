export interface Thread {
  thread_id: number
  thread_name?: string
  thread_type: 'direct' | 'group'
  created_by: number
  created_at: string
  updated_at: string
  last_message_at?: string
  last_message_preview?: string
  message_count: number
  unread_count: number
  participant_count: number
  is_admin: boolean
  participants?: ThreadParticipant[]
  last_message?: Message
}

export interface ThreadParticipant {
  participant_id: number
  thread_id: number
  user_id: number
  username: string
  role: 'admin' | 'member'
  joined_at: string
  left_at?: string
  is_active: boolean
  last_read_at?: string
  unread_count: number
  is_online?: boolean
  is_typing?: boolean
}

export interface CreateThreadRequest {
  thread_name?: string
  thread_type: 'direct' | 'group'
  participant_ids: number[]
}

export interface UpdateThreadRequest {
  thread_name?: string
  thread_type?: 'direct' | 'group'
}

export interface AddParticipantRequest {
  user_id: number
  role?: 'admin' | 'member'
}

export interface UpdateParticipantRoleRequest {
  user_id: number
  role: 'admin' | 'member'
}

export interface GetThreadsQuery {
  page?: number
  per_page?: number
  thread_type?: 'direct' | 'group'
  search?: string
}

export interface ThreadListResponse {
  threads: Thread[]
  total: number
  page: number
  per_page: number
  has_more: boolean
}

export interface ThreadResponse {
  thread: Thread
}

// Re-import Message type for last_message
import type { Message } from './message.types'
