import { ref } from 'vue'
import type { Message, Thread, UserStatus, TypingIndicator, WebRTCSignal } from '@/types/messaging'

export interface WebSocketEvents {
  'message:new': (message: Message) => void
  'message:updated': (message: Message) => void
  'message:read': (messageId: number, userId: number) => void
  'thread:new': (thread: Thread) => void
  'thread:updated': (thread: Thread) => void
  'user:status': (status: UserStatus) => void
  'user:typing': (indicator: TypingIndicator) => void
  'webrtc:signal': (signal: WebRTCSignal) => void
  'webrtc:call': (callData: unknown) => void
  'notification:new': (notification: unknown) => void
}

// Minimal message envelope coming from the backend WebSocket handler.
interface ServerMessage {
  type: string
  [key: string]: unknown
}

class WebSocketService {
  private ws: WebSocket | null = null
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectDelay = 1000
  private currentThreadId?: number
  private connectResolve: (() => void) | null = null
  private connectReject: ((err: Error) => void) | null = null
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null

  // Reactive state
  public isConnected = ref(false)
  public isConnecting = ref(false)
  public connectionError = ref<string | null>(null)

  // Event listeners
  private eventListeners = new Map<keyof WebSocketEvents, Set<(...args: unknown[]) => void>>()

  constructor() {
    this.setupEventHandlers()
  }

  private setupEventHandlers() {
    const events: (keyof WebSocketEvents)[] = [
      'message:new',
      'message:updated',
      'message:read',
      'thread:new',
      'thread:updated',
      'user:status',
      'user:typing',
      'webrtc:signal',
      'webrtc:call',
      'notification:new',
    ]

    events.forEach((event) => {
      this.eventListeners.set(event, new Set())
    })
  }

  private getString(data: ServerMessage, key: string, fallback = ''): string {
    const value = data[key]
    if (typeof value === 'string') return value
    if (value === undefined || value === null) return fallback
    return String(value)
  }

  private getNumber(data: ServerMessage, key: string, fallback = 0): number {
    const value = data[key]
    if (typeof value === 'number') return value
    if (typeof value === 'string') return Number(value) || fallback
    return fallback
  }

  private getBoolean(data: ServerMessage, key: string, fallback = false): boolean {
    const value = data[key]
    if (typeof value === 'boolean') return value
    if (typeof value === 'string') return value === 'true'
    return fallback
  }

  private resolveBaseUrl(): string {
    const envUrl = import.meta.env.VITE_WS_URL as string | undefined
    if (envUrl) {
      return envUrl
    }
    if (typeof window !== 'undefined') {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
      return `${protocol}//${window.location.host}`
    }
    return 'ws://localhost:20380'
  }

  public connect(threadId?: number): Promise<void> {
    return new Promise((resolve, reject) => {
      if (
        this.ws &&
        (this.ws.readyState === WebSocket.CONNECTING || this.ws.readyState === WebSocket.OPEN)
      ) {
        resolve()
        return
      }

      this.isConnecting.value = true
      this.connectionError.value = null
      this.connectResolve = resolve
      this.connectReject = reject
      this.currentThreadId = threadId

      const baseUrl = this.resolveBaseUrl()
      const endpoint = threadId ? `/messaging/ws/${threadId}` : '/messaging/ws'
      const url = `${baseUrl}${endpoint}`

      try {
        this.ws = new WebSocket(url)
      } catch (err) {
        this.isConnecting.value = false
        this.connectionError.value =
          err instanceof Error ? err.message : 'Failed to create WebSocket'
        reject(new Error(this.connectionError.value))
        return
      }

      this.ws.onopen = () => {
        console.log('WebSocket connected')
        this.isConnected.value = true
        this.isConnecting.value = false
        this.connectionError.value = null
        this.reconnectAttempts = 0

        const resolveFn = this.connectResolve
        this.connectResolve = null
        this.connectReject = null
        resolveFn?.()
      }

      this.ws.onmessage = (event) => {
        this.handleMessage(event.data)
      }

      this.ws.onclose = () => {
        console.log('WebSocket disconnected')
        this.isConnected.value = false
        this.isConnecting.value = false

        if (this.connectReject) {
          this.connectReject(new Error('WebSocket closed before connecting'))
          this.connectReject = null
          this.connectResolve = null
        }

        this.handleReconnect()
      }

      this.ws.onerror = () => {
        console.error('WebSocket connection error')
        this.isConnecting.value = false
        this.connectionError.value = 'WebSocket connection error'

        if (this.connectReject) {
          this.connectReject(new Error(this.connectionError.value))
          this.connectReject = null
          this.connectResolve = null
        }
      }
    })
  }

  private handleMessage(data: unknown) {
    if (typeof data !== 'string') {
      return
    }

    let parsed: ServerMessage
    try {
      parsed = JSON.parse(data) as ServerMessage
    } catch {
      return
    }

    if (!parsed || typeof parsed !== 'object' || !parsed.type) {
      return
    }

    switch (parsed.type) {
      case 'chat': {
        this.emit('message:new', this.toMessage(parsed))
        break
      }
      case 'typing': {
        this.emit('user:typing', this.toTypingIndicator(parsed))
        break
      }
      case 'user_status': {
        this.emit('user:status', this.toUserStatus(parsed))
        break
      }
      case 'webrtc_signal':
      case 'webrtc': {
        this.emit('webrtc:signal', parsed as unknown as WebRTCSignal)
        this.emit('webrtc:call', parsed as unknown)
        break
      }
      case 'notification': {
        this.emit('notification:new', parsed)
        break
      }
      case 'error': {
        console.warn('WebSocket server error:', this.getString(parsed, 'message'))
        break
      }
      default:
        break
    }
  }

  private toMessage(data: ServerMessage): Message {
    const timestamp = this.getString(data, 'timestamp', new Date().toISOString())
    return {
      message_id: this.getNumber(data, 'id'),
      thread_id: this.getNumber(data, 'thread_id'),
      sender_id: this.getNumber(data, 'sender_id'),
      username: this.getString(data, 'sender_name'),
      message_type: 'text',
      encrypted_content: this.getString(data, 'content'),
      content_nonce: '',
      sender_key_signature: undefined,
      reply_to_message_id: undefined,
      created_at: timestamp,
      updated_at: timestamp,
      is_deleted: false,
      edit_count: 0,
      last_edited_at: undefined,
      is_from_sender: false,
      is_read: false,
      delivery_status: 'delivered',
    }
  }

  private toTypingIndicator(data: ServerMessage): TypingIndicator {
    return {
      user_id: this.getNumber(data, 'user_id', this.getNumber(data, 'sender_id')),
      username: this.getString(data, 'user_name', this.getString(data, 'sender_name')),
      thread_id: this.getNumber(data, 'thread_id'),
      is_typing: this.getBoolean(data, 'is_typing'),
      timestamp: new Date().toISOString(),
    }
  }

  private toUserStatus(data: ServerMessage): UserStatus {
    return {
      user_id: this.getNumber(data, 'user_id'),
      username: this.getString(data, 'user_name'),
      is_online: this.getString(data, 'status').toLowerCase() === 'online',
      last_seen: new Date().toISOString(),
      is_typing: false,
      typing_in_thread: this.getNumber(data, 'thread_id') || undefined,
    }
  }

  private handleReconnect() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
    }

    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.warn('Max WebSocket reconnect attempts reached')
      return
    }

    this.reconnectAttempts++
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts)

    this.reconnectTimer = setTimeout(() => {
      console.log(
        `Attempting to reconnect... (${this.reconnectAttempts}/${this.maxReconnectAttempts})`
      )
      this.connect(this.currentThreadId).catch(() => {})
    }, delay)
  }

  public disconnect() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }

    if (this.ws) {
      this.ws.onclose = null
      this.ws.close()
      this.ws = null
    }

    this.isConnected.value = false
    this.isConnecting.value = false
  }

  // Event management
  public on<K extends keyof WebSocketEvents>(event: K, callback: WebSocketEvents[K]) {
    const listeners = this.eventListeners.get(event)
    if (listeners) {
      listeners.add(callback)
    }
  }

  public off<K extends keyof WebSocketEvents>(event: K, callback: WebSocketEvents[K]) {
    const listeners = this.eventListeners.get(event)
    if (listeners) {
      listeners.delete(callback)
    }
  }

  private emit<K extends keyof WebSocketEvents>(event: K, ...args: Parameters<WebSocketEvents[K]>) {
    const listeners = this.eventListeners.get(event)
    if (listeners) {
      listeners.forEach((callback) => {
        try {
          callback(...args)
        } catch (error) {
          console.error(`Error in WebSocket event handler for ${event}:`, error)
        }
      })
    }
  }

  // Send messages
  public sendMessage(message: unknown) {
    if (!message || typeof message !== 'object' || Array.isArray(message)) {
      return
    }

    const msg = message as Record<string, unknown>
    const type = typeof msg.type === 'string' ? msg.type : ''
    let payload: Record<string, unknown>

    if (type === 'heartbeat') {
      payload = { type: 'ping' }
    } else if (type === 'call_reject' || type === 'call_ended') {
      // WebRTC control messages are not yet handled by the backend; avoid sending
      // malformed payloads.
      return
    } else if (type) {
      payload = msg
    } else {
      payload = { type: 'chat', ...msg }
    }

    this.sendJson(payload)
  }

  public sendTypingIndicator(threadId: number, isTyping: boolean) {
    this.sendJson({
      type: 'typing',
      thread_id: threadId,
      is_typing: isTyping,
    })
  }

  public markMessageRead(_messageId: number) {
    // Not yet implemented on the backend; avoid sending unknown payloads.
  }

  public sendWebRTCSignal(signal: WebRTCSignal) {
    this.sendJson({
      type: 'webrtc_signal',
      ...signal,
    })
  }

  private sendJson(payload: Record<string, unknown>) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(payload))
    } else {
      console.warn('WebSocket not connected, cannot send message')
    }
  }

  // Connection status
  public getConnectionState() {
    return {
      isConnected: this.isConnected.value,
      isConnecting: this.isConnecting.value,
      error: this.connectionError.value,
    }
  }
}

// Create singleton instance
export const webSocketService = new WebSocketService()

// Export composable for Vue components
export function useWebSocket() {
  return {
    isConnected: webSocketService.isConnected,
    isConnecting: webSocketService.isConnecting,
    connectionError: webSocketService.connectionError,
    connect: webSocketService.connect.bind(webSocketService),
    disconnect: webSocketService.disconnect.bind(webSocketService),
    sendMessage: webSocketService.sendMessage.bind(webSocketService),
    sendTypingIndicator: webSocketService.sendTypingIndicator.bind(webSocketService),
    markMessageRead: webSocketService.markMessageRead.bind(webSocketService),
    sendWebRTCSignal: webSocketService.sendWebRTCSignal.bind(webSocketService),
    on: webSocketService.on.bind(webSocketService),
    off: webSocketService.off.bind(webSocketService),
  }
}
