export interface Message {
  message_id: number
  thread_id: number
  sender_id: number
  username: string
  message_type: 'text' | 'image' | 'file' | 'system'
  encrypted_content: string
  content_nonce: string
  sender_key_signature?: string
  reply_to_message_id?: number
  created_at: string
  updated_at: string
  is_deleted: boolean
  edit_count: number
  last_edited_at?: string
  is_from_sender: boolean
  is_read?: boolean
  delivery_status?: 'sending' | 'sent' | 'delivered' | 'read'
}

export interface SendMessageRequest {
  thread_id: number
  message_type: 'text' | 'image' | 'file' | 'system'
  encrypted_content: string
  content_nonce: string
  sender_key_signature?: string
  reply_to_message_id?: number
}

export interface UpdateMessageRequest {
  message_id: number
  encrypted_content: string
  content_nonce: string
  sender_key_signature?: string
}

export interface GetMessagesQuery {
  page?: number
  per_page?: number
  message_type?: string
  search?: string
}

export interface MessageListResponse {
  messages: Message[]
  total: number
  page: number
  per_page: number
  has_more: boolean
}

export interface MessageResponse {
  message: Message
}
