import { io, type Socket } from 'socket.io-client'
import { ref, reactive } from 'vue'
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
  'webrtc:call': (callData: any) => void
  'notification:new': (notification: any) => void
}

class WebSocketService {
  private socket: Socket | null = null
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectDelay = 1000

  // Reactive state
  public isConnected = ref(false)
  public isConnecting = ref(false)
  public connectionError = ref<string | null>(null)

  // Event listeners
  private eventListeners = new Map<keyof WebSocketEvents, Set<Function>>()

  constructor() {
    this.setupEventHandlers()
  }

  private setupEventHandlers() {
    // Initialize event listener maps
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
      'notification:new'
    ]

    events.forEach(event => {
      this.eventListeners.set(event, new Set())
    })
  }

  public connect(threadId?: number): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.socket?.connected) {
        resolve()
        return
      }

      this.isConnecting.value = true
      this.connectionError.value = null

      const wsUrl = import.meta.env.VITE_WS_URL || 'ws://localhost:20380'
      const endpoint = threadId ? `/messaging/ws/${threadId}` : '/messaging/ws'

      this.socket = io(`${wsUrl}${endpoint}`, {
        transports: ['websocket'],
        upgrade: false,
        rememberUpgrade: false,
        timeout: 10000,
        forceNew: true,
      })

      this.socket.on('connect', () => {
        console.log('WebSocket connected')
        this.isConnected.value = true
        this.isConnecting.value = false
        this.connectionError.value = null
        this.reconnectAttempts = 0
        resolve()
      })

      this.socket.on('disconnect', (reason) => {
        console.log('WebSocket disconnected:', reason)
        this.isConnected.value = false
        this.handleReconnect()
      })

      this.socket.on('connect_error', (error) => {
        console.error('WebSocket connection error:', error)
        this.isConnecting.value = false
        this.connectionError.value = error.message
        this.reconnectAttempts++
        
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
          reject(new Error('Failed to connect to WebSocket'))
        }
      })

      // Set up message handlers
      this.setupMessageHandlers()
    })
  }

  private setupMessageHandlers() {
    if (!this.socket) return

    this.socket.on('message', (data) => {
      this.handleIncomingMessage(data)
    })

    this.socket.on('thread_updated', (data) => {
      this.emit('thread:updated', data)
    })

    this.socket.on('user_status', (data) => {
      this.emit('user:status', data)
    })

    this.socket.on('typing_indicator', (data) => {
      this.emit('user:typing', data)
    })

    this.socket.on('webrtc_signal', (data) => {
      this.emit('webrtc:signal', data)
    })

    this.socket.on('notification', (data) => {
      this.emit('notification:new', data)
    })
  }

  private handleIncomingMessage(data: any) {
    const message: Message = {
      ...data,
      delivery_status: 'delivered',
      is_read: false
    }
    this.emit('message:new', message)
  }

  private handleReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      setTimeout(() => {
        console.log(`Attempting to reconnect... (${this.reconnectAttempts + 1}/${this.maxReconnectAttempts})`)
        this.connect()
      }, this.reconnectDelay * Math.pow(2, this.reconnectAttempts))
    }
  }

  public disconnect() {
    if (this.socket) {
      this.socket.disconnect()
      this.socket = null
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
      listeners.forEach(callback => {
        try {
          callback(...args)
        } catch (error) {
          console.error(`Error in WebSocket event handler for ${event}:`, error)
        }
      })
    }
  }

  // Send messages
  public sendMessage(message: any) {
    if (this.socket?.connected) {
      this.socket.emit('message', message)
    } else {
      console.warn('WebSocket not connected, cannot send message')
    }
  }

  public sendTypingIndicator(threadId: number, isTyping: boolean) {
    if (this.socket?.connected) {
      this.socket.emit('typing', { thread_id: threadId, is_typing: isTyping })
    }
  }

  public markMessageRead(messageId: number) {
    if (this.socket?.connected) {
      this.socket.emit('mark_read', { message_id: messageId })
    }
  }

  public sendWebRTCSignal(signal: WebRTCSignal) {
    if (this.socket?.connected) {
      this.socket.emit('webrtc_signal', signal)
    }
  }

  // Connection status
  public getConnectionState() {
    return {
      isConnected: this.isConnected.value,
      isConnecting: this.isConnecting.value,
      error: this.connectionError.value
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
