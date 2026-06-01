import { ref, reactive, computed } from 'vue'
import { webSocketService } from './WebSocketService'
import type { UserStatus, UserPresence } from '@/types/messaging'

class PresenceService {
  // Reactive state
  private userPresences = reactive(new Map<number, UserPresence>())
  private onlineUsers = ref(new Set<number>())
  private lastSeenCache = reactive(new Map<number, string>())

  // Computed properties
  public onlineCount = computed(() => this.onlineUsers.value.size)
  public onlineUserIds = computed(() => Array.from(this.onlineUsers.value))

  constructor() {
    this.setupWebSocketListeners()
    this.startHeartbeat()
  }

  private setupWebSocketListeners() {
    webSocketService.on('user:status', (status: UserStatus) => {
      this.updateUserStatus(status)
    })
  }

  private updateUserStatus(status: UserStatus) {
    const presence: UserPresence = {
      user_id: status.user_id,
      status: status.is_online ? 'online' : 'offline',
      last_seen: status.last_seen,
      current_thread: status.typing_in_thread
    }

    this.userPresences.set(status.user_id, presence)

    if (status.is_online) {
      this.onlineUsers.value.add(status.user_id)
    } else {
      this.onlineUsers.value.delete(status.user_id)
      if (status.last_seen) {
        this.lastSeenCache.set(status.user_id, status.last_seen)
      }
    }
  }

  private startHeartbeat() {
    // Send heartbeat every 30 seconds to maintain online status
    setInterval(() => {
      if (webSocketService.isConnected.value) {
        webSocketService.sendMessage({ type: 'heartbeat' })
      }
    }, 30000)
  }

  // Public API
  public isUserOnline(userId: number): boolean {
    return this.onlineUsers.value.has(userId)
  }

  public getUserPresence(userId: number): UserPresence | undefined {
    return this.userPresences.get(userId)
  }

  public getLastSeen(userId: number): string | undefined {
    return this.lastSeenCache.get(userId)
  }

  public formatLastSeen(timestamp: string): string {
    const date = new Date(timestamp)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMins / 60)
    const diffDays = Math.floor(diffHours / 24)

    if (diffMins < 1) {
      return 'just now'
    } else if (diffMins < 60) {
      return `${diffMins}m ago`
    } else if (diffHours < 24) {
      return `${diffHours}h ago`
    } else if (diffDays < 7) {
      return `${diffDays}d ago`
    } else {
      return date.toLocaleDateString()
    }
  }

  public getUserStatusText(userId: number): string {
    if (this.isUserOnline(userId)) {
      return 'Online'
    }

    const lastSeen = this.getLastSeen(userId)
    if (lastSeen) {
      return `Last seen ${this.formatLastSeen(lastSeen)}`
    }

    return 'Offline'
  }

  // Update current user's typing status
  public setCurrentThread(threadId: number | null) {
    // This would be called when user navigates to a thread
    // Implementation depends on backend requirements
  }

  // Cleanup
  public cleanup() {
    this.userPresences.clear()
    this.onlineUsers.value.clear()
    this.lastSeenCache.clear()
  }
}

// Create singleton instance
export const presenceService = new PresenceService()

// Export composable for Vue components
export function usePresence() {
  return {
    onlineCount: presenceService.onlineCount,
    onlineUserIds: presenceService.onlineUserIds,
    isUserOnline: presenceService.isUserOnline.bind(presenceService),
    getUserPresence: presenceService.getUserPresence.bind(presenceService),
    getLastSeen: presenceService.getLastSeen.bind(presenceService),
    formatLastSeen: presenceService.formatLastSeen.bind(presenceService),
    getUserStatusText: presenceService.getUserStatusText.bind(presenceService),
    setCurrentThread: presenceService.setCurrentThread.bind(presenceService),
  }
}
