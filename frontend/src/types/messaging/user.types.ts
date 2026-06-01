export interface UserStatus {
  user_id: number
  username: string
  is_online: boolean
  last_seen?: string
  is_typing?: boolean
  typing_in_thread?: number
}

export interface BlockedUser {
  block_id: number
  blocker_id: number
  blocked_id: number
  blocked_username: string
  blocked_at: string
  reason?: string
  is_active: boolean
}

export interface BlockUserRequest {
  user_id: number
  reason?: string
}

export interface BlockedUserResponse {
  blocked_users: BlockedUser[]
  total: number
}

export interface UserPresence {
  user_id: number
  status: 'online' | 'offline' | 'away'
  last_seen?: string
  current_thread?: number
}

export interface TypingIndicator {
  user_id: number
  username: string
  thread_id: number
  is_typing: boolean
  timestamp: string
}
