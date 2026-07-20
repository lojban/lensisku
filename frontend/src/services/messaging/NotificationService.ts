import { ref } from 'vue'
import { webSocketService } from './WebSocketService'
import type { Message } from '@/types/messaging'

// Type definition for PWA install prompt event
interface BeforeInstallPromptEvent extends Event {
  readonly platforms: string[]
  readonly userChoice: Promise<{
    outcome: 'accepted' | 'dismissed'
    platform: string
  }>
  prompt(): Promise<void>
}

interface PushNotification {
  id: string
  title: string
  body: string
  icon?: string
  badge?: string
  tag?: string
  data?: unknown
  requireInteraction?: boolean
  actions?: NotificationAction[]
}

interface NotificationAction {
  action: string
  title: string
  icon?: string
}

class NotificationService {
  private isSupported = ref(false)
  private permission = ref<NotificationPermission>('default')
  private isSubscribed = ref(false)
  private subscription: PushSubscription | null = null
  private registration: ServiceWorkerRegistration | null = null

  constructor() {
    this.checkSupport()
    this.setupWebSocketListeners()
  }

  private async checkSupport() {
    if (typeof window === 'undefined' || typeof navigator === 'undefined') {
      this.isSupported.value = false
      return
    }
    this.isSupported.value =
      'Notification' in window && 'serviceWorker' in navigator && 'PushManager' in window

    if (this.isSupported.value) {
      this.permission.value = await Notification.requestPermission()
      await this.getRegistration()
    }
  }

  private async getRegistration() {
    if (typeof navigator === 'undefined') {
      return
    }
    if ('serviceWorker' in navigator) {
      this.registration = await navigator.serviceWorker.ready
    }
  }

  private setupWebSocketListeners() {
    webSocketService.on('notification:new', (notification: unknown) => {
      this.handleIncomingNotification(notification)
    })

    webSocketService.on('message:new', (message: Message) => {
      this.handleNewMessage(message)
    })
  }

  private async handleIncomingNotification(notification: unknown) {
    if (typeof document === 'undefined' || !document.hidden) {
      return
    }
    const notif = notification as {
      id: string
      title: string
      body: string
      thread_id?: number
      message_id?: number
    }
    await this.showNotification({
      id: notif.id,
      title: notif.title,
      body: notif.body,
      icon: '/icons/icon-192x192.png',
      badge: '/icons/badge-72x72.png',
      tag: notif.thread_id?.toString(),
      data: {
        threadId: notif.thread_id,
        messageId: notif.message_id,
        type: 'message',
      },
      requireInteraction: true,
      actions: [
        {
          action: 'reply',
          title: 'Reply',
          icon: '/icons/reply.png',
        },
        {
          action: 'mark-read',
          title: 'Mark as Read',
          icon: '/icons/check.png',
        },
      ],
    })
  }

  private async handleNewMessage(message: Message) {
    if (typeof document === 'undefined' || !document.hidden || message.is_from_sender) {
      return
    }
    await this.showNotification({
      id: `message-${message.message_id}`,
      title: 'New Message',
      body: 'You have a new message',
      icon: '/icons/icon-192x192.png',
      badge: '/icons/badge-72x72.png',
      tag: `thread-${message.thread_id}`,
      data: {
        threadId: message.thread_id,
        messageId: message.message_id,
        type: 'message',
      },
      requireInteraction: false,
    })
  }

  public async requestPermission(): Promise<NotificationPermission> {
    if (!this.isSupported.value) {
      return 'denied'
    }

    this.permission.value = await Notification.requestPermission()
    return this.permission.value
  }

  public async showNotification(notification: PushNotification): Promise<void> {
    if (!this.isSupported.value || this.permission.value !== 'granted') {
      return
    }

    try {
      const notificationOptions: NotificationOptions = {
        body: notification.body,
        icon: notification.icon,
        badge: notification.badge,
        tag: notification.tag,
        data: notification.data,
        requireInteraction: notification.requireInteraction,
        silent: false,
      }

      const browserNotification = new Notification(notification.title, notificationOptions)

      // Handle notification clicks
      browserNotification.onclick = (event) => {
        event.preventDefault()
        this.handleNotificationClick(notification.data)
        browserNotification.close()
      }

      // Auto-close after 5 seconds if not requireInteraction
      if (!notification.requireInteraction) {
        setTimeout(() => {
          browserNotification.close()
        }, 5000)
      }
    } catch (error) {
      console.error('Failed to show notification:', error)
    }
  }

  private handleNotificationClick(data: unknown) {
    if (typeof window === 'undefined') {
      return
    }
    const notificationData = data as {
      type?: string
      threadId?: number
    }
    if (notificationData.type === 'message' && notificationData.threadId) {
      // Navigate to the thread
      window.focus()
      window.location.href = `/messages/${notificationData.threadId}`
    }
  }

  // Push subscription management
  public async subscribeToPush(): Promise<boolean> {
    if (!this.registration || !this.isSupported.value) {
      return false
    }

    try {
      const subscription = await this.registration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: this.urlBase64ToUint8Array(
          import.meta.env.VITE_VAPID_PUBLIC_KEY || ''
        ) as BufferSource,
      })

      this.subscription = subscription
      this.isSubscribed.value = true

      // Send subscription to server
      await this.sendSubscriptionToServer(subscription)

      return true
    } catch (error) {
      console.error('Failed to subscribe to push notifications:', error)
      return false
    }
  }

  public async unsubscribeFromPush(): Promise<boolean> {
    if (!this.subscription) {
      return true
    }

    try {
      await this.subscription.unsubscribe()
      this.subscription = null
      this.isSubscribed.value = false

      // Remove subscription from server
      await this.removeSubscriptionFromServer()

      return true
    } catch (error) {
      console.error('Failed to unsubscribe from push notifications:', error)
      return false
    }
  }

  private async sendSubscriptionToServer(subscription: PushSubscription): Promise<void> {
    try {
      await fetch('/api/notifications/subscribe', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(subscription as unknown),
      })
    } catch (error) {
      console.error('Failed to send subscription to server:', error)
    }
  }

  private async removeSubscriptionFromServer(): Promise<void> {
    try {
      await fetch('/api/notifications/unsubscribe', {
        method: 'DELETE',
      })
    } catch (error) {
      console.error('Failed to remove subscription from server:', error)
    }
  }

  private urlBase64ToUint8Array(base64String: string): Uint8Array {
    const padding = '='.repeat((4 - (base64String.length % 4)) % 4)
    const base64 = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/')
    if (typeof window === 'undefined' || typeof window.atob !== 'function') {
      throw new Error('urlBase64ToUint8Array requires a browser environment')
    }
    const rawData = window.atob(base64)
    const outputArray = new Uint8Array(rawData.length)

    for (let i = 0; i < rawData.length; ++i) {
      outputArray[i] = rawData.charCodeAt(i)
    }

    return outputArray
  }

  // PWA Installation Prompt
  private deferredPrompt: BeforeInstallPromptEvent | null = null

  public setupInstallPrompt() {
    if (typeof window === 'undefined') {
      return
    }
    window.addEventListener('beforeinstallprompt', (e) => {
      e.preventDefault()
      this.deferredPrompt = e as BeforeInstallPromptEvent
    })
  }

  public async showInstallPrompt(): Promise<boolean> {
    if (!this.deferredPrompt) {
      return false
    }

    try {
      await this.deferredPrompt.prompt()
      const result = await this.deferredPrompt.userChoice

      this.deferredPrompt = null
      return result.outcome === 'accepted'
    } catch (error) {
      console.error('Failed to show install prompt:', error)
      return false
    }
  }

  // Getters
  public get supported() {
    return this.isSupported.value
  }

  public get notificationPermission() {
    return this.permission.value
  }

  public get pushSubscribed() {
    return this.isSubscribed.value
  }

  public get canInstallPWA() {
    return !!this.deferredPrompt
  }
}

// Create singleton instance
export const notificationService = new NotificationService()

// Export composable for Vue components
export function useNotifications() {
  return {
    supported: notificationService.supported,
    notificationPermission: notificationService.notificationPermission,
    pushSubscribed: notificationService.pushSubscribed,
    canInstallPWA: notificationService.canInstallPWA,
    requestPermission: notificationService.requestPermission.bind(notificationService),
    showNotification: notificationService.showNotification.bind(notificationService),
    subscribeToPush: notificationService.subscribeToPush.bind(notificationService),
    unsubscribeFromPush: notificationService.unsubscribeFromPush.bind(notificationService),
    showInstallPrompt: notificationService.showInstallPrompt.bind(notificationService),
    setupInstallPrompt: notificationService.setupInstallPrompt.bind(notificationService),
  }
}
