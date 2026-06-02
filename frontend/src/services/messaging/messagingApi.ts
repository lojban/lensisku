import { api } from '@/api'
import type {
  SendMessageRequest,
  UpdateMessageRequest,
  GetMessagesQuery,
  MessageListResponse,
  MessageResponse,
  CreateThreadRequest,
  UpdateThreadRequest,
  AddParticipantRequest,
  UpdateParticipantRoleRequest,
  GetThreadsQuery,
  ThreadListResponse,
  ThreadResponse,
  BlockUserRequest,
  BlockedUserResponse,
  WebRTCSignalRequest,
  WebRTCSignalResponse,
  WebRTCConfig,
  ActiveCall,
} from '@/types/messaging'

// Thread API
export const getThreads = async (query: GetThreadsQuery = {}): Promise<ThreadListResponse> => {
  const params = new URLSearchParams()
  if (query.page) params.append('page', query.page.toString())
  if (query.per_page) params.append('per_page', query.per_page.toString())
  if (query.thread_type) params.append('thread_type', query.thread_type)
  if (query.search) params.append('search', query.search)

  const response = await api.get(`/messaging/threads?${params.toString()}`)
  return response.data
}

export const getThread = async (threadId: number): Promise<ThreadResponse> => {
  const response = await api.get(`/messaging/threads/${threadId}`)
  return response.data
}

export const createThread = async (request: CreateThreadRequest): Promise<ThreadResponse> => {
  const response = await api.post('/messaging/threads', request)
  return response.data
}

export const updateThread = async (
  threadId: number,
  request: UpdateThreadRequest
): Promise<ThreadResponse> => {
  const response = await api.put(`/messaging/threads/${threadId}`, request)
  return response.data
}

export const deleteThread = async (threadId: number): Promise<void> => {
  await api.delete(`/messaging/threads/${threadId}`)
}

// Messages API
export const getMessages = async (
  threadId: number,
  query: GetMessagesQuery = {}
): Promise<MessageListResponse> => {
  const params = new URLSearchParams()
  if (query.page) params.append('page', query.page.toString())
  if (query.per_page) params.append('per_page', query.per_page.toString())
  if (query.message_type) params.append('message_type', query.message_type)
  if (query.search) params.append('search', query.search)

  const response = await api.get(`/messaging/threads/${threadId}/messages?${params.toString()}`)
  return response.data
}

export const getMessage = async (messageId: number): Promise<MessageResponse> => {
  const response = await api.get(`/messaging/messages/${messageId}`)
  return response.data
}

export const sendMessage = async (request: SendMessageRequest): Promise<MessageResponse> => {
  const response = await api.post('/messaging/messages', request)
  return response.data
}

export const updateMessage = async (
  messageId: number,
  request: UpdateMessageRequest
): Promise<MessageResponse> => {
  const response = await api.put(`/messaging/messages/${messageId}`, request)
  return response.data
}

export const deleteMessage = async (messageId: number): Promise<void> => {
  await api.delete(`/messaging/messages/${messageId}`)
}

// Thread Participants API
export const getThreadParticipants = async (threadId: number): Promise<unknown> => {
  const response = await api.get(`/messaging/threads/${threadId}/participants`)
  return response.data
}

export const addParticipant = async (
  threadId: number,
  request: AddParticipantRequest
): Promise<unknown> => {
  const response = await api.post(`/messaging/threads/${threadId}/participants`, request)
  return response.data
}

export const updateParticipantRole = async (
  threadId: number,
  request: UpdateParticipantRoleRequest
): Promise<unknown> => {
  const response = await api.put(`/messaging/threads/${threadId}/participants`, request)
  return response.data
}

export const removeParticipant = async (threadId: number, userId: number): Promise<void> => {
  await api.delete(`/messaging/threads/${threadId}/participants/${userId}`)
}

// User Blocking API
export const blockUser = async (request: BlockUserRequest): Promise<void> => {
  await api.post('/messaging/blocks', request)
}

export const unblockUser = async (userId: number): Promise<void> => {
  await api.delete(`/messaging/blocks/${userId}`)
}

export const getBlockedUsers = async (): Promise<BlockedUserResponse> => {
  const response = await api.get('/messaging/blocks')
  return response.data
}

// WebRTC Signaling API
export const sendSignal = async (request: WebRTCSignalRequest): Promise<void> => {
  await api.post('/messaging/webrtc/signal', request)
}

export const getPendingSignals = async (): Promise<WebRTCSignalResponse[]> => {
  const response = await api.get('/messaging/webrtc/signals/me')
  return response.data
}

export const markSignalProcessed = async (signalId: number): Promise<void> => {
  await api.put(`/messaging/webrtc/signal/${signalId}/processed`)
}

export const cleanupExpiredSignals = async (): Promise<void> => {
  await api.delete('/messaging/webrtc/cleanup')
}

export const getActiveCalls = async (): Promise<ActiveCall[]> => {
  const response = await api.get('/messaging/webrtc/calls')
  return response.data
}

export const getWebRTCConfig = async (): Promise<WebRTCConfig> => {
  const response = await api.get('/messaging/webrtc/config')
  return response.data
}

// Health check
export const getMessagingHealth = async (): Promise<{ status: string }> => {
  const response = await api.get('/messaging/health')
  return response.data
}
