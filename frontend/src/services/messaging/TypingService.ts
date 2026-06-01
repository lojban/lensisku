import { ref, reactive, computed } from 'vue'
import { webSocketService } from './WebSocketService'
import type { TypingIndicator } from '@/types/messaging'

class TypingService {
  // Reactive state
  private typingIndicators = reactive(new Map<number, TypingIndicator>())
  private currentTypingUsers = ref(new Set<number>())
  private typingTimeouts = reactive(new Map<number, number>())

  // Computed properties
  public typingUserIds = computed(() => Array.from(this.currentTypingUsers.value))
  public typingCount = computed(() => this.currentTypingUsers.value.size)

  constructor() {
    this.setupWebSocketListeners()
  }

  private setupWebSocketListeners() {
    webSocketService.on('user:typing', (indicator: TypingIndicator) => {
      this.handleTypingIndicator(indicator)
    })
  }

  private handleTypingIndicator(indicator: TypingIndicator) {
    const userId = indicator.user_id
    const threadId = indicator.thread_id

    if (indicator.is_typing) {
      // User started typing
      this.typingIndicators.set(userId, indicator)
      this.currentTypingUsers.value.add(userId)

      // Clear existing timeout for this user
      const existingTimeout = this.typingTimeouts.get(userId)
      if (existingTimeout) {
        clearTimeout(existingTimeout)
      }

      // Set timeout to remove typing indicator after 3 seconds of inactivity
      const timeout = setTimeout(() => {
        this.stopTyping(userId)
      }, 3000)

      this.typingTimeouts.set(userId, timeout)
    } else {
      // User stopped typing
      this.stopTyping(userId)
    }
  }

  private stopTyping(userId: number) {
    this.typingIndicators.delete(userId)
    this.currentTypingUsers.value.delete(userId)

    const timeout = this.typingTimeouts.get(userId)
    if (timeout) {
      clearTimeout(timeout)
      this.typingTimeouts.delete(userId)
    }
  }

  // Public API
  public sendTypingIndicator(threadId: number, isTyping: boolean) {
    webSocketService.sendTypingIndicator(threadId, isTyping)

    // Also update local state immediately for better UX
    if (isTyping) {
      // We don't track our own typing state in the indicators map
      // But we could if we wanted to show "You are typing..." somewhere
    } else {
      // Clear our own typing timeout if we stop typing
      const timeout = this.typingTimeouts.get(-1) // Use -1 for current user
      if (timeout) {
        clearTimeout(timeout)
        this.typingTimeouts.delete(-1)
      }
    }
  }

  public isUserTyping(userId: number): boolean {
    return this.currentTypingUsers.value.has(userId)
  }

  public getTypingIndicator(userId: number): TypingIndicator | undefined {
    return this.typingIndicators.get(userId)
  }

  public getTypingUsersInThread(threadId: number): TypingIndicator[] {
    return Array.from(this.typingIndicators.values()).filter(
      indicator => indicator.thread_id === threadId
    )
  }

  public getTypingText(threadId: number, currentUserId: number): string {
    const typingUsers = this.getTypingUsersInThread(threadId)
      .filter(indicator => indicator.user_id !== currentUserId)

    if (typingUsers.length === 0) {
      return ''
    } else if (typingUsers.length === 1) {
      return `${typingUsers[0].username} is typing...`
    } else if (typingUsers.length === 2) {
      return `${typingUsers[0].username} and ${typingUsers[1].username} are typing...`
    } else {
      return `${typingUsers.length} people are typing...`
    }
  }

  // Cleanup
  public cleanup() {
    // Clear all timeouts
    this.typingTimeouts.forEach(timeout => clearTimeout(timeout))
    this.typingTimeouts.clear()

    // Clear state
    this.typingIndicators.clear()
    this.currentTypingUsers.value.clear()
  }
}

// Create singleton instance
export const typingService = new TypingService()

// Export composable for Vue components
export function useTyping() {
  return {
    typingUserIds: typingService.typingUserIds,
    typingCount: typingService.typingCount,
    isUserTyping: typingService.isUserTyping.bind(typingService),
    getTypingIndicator: typingService.getTypingIndicator.bind(typingService),
    getTypingUsersInThread: typingService.getTypingUsersInThread.bind(typingService),
    getTypingText: typingService.getTypingText.bind(typingService),
    sendTypingIndicator: typingService.sendTypingIndicator.bind(typingService),
  }
}
