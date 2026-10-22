/** Cached copy of users.metadata.notifications_enabled. Missing means on. */
export const NOTIFICATIONS_ENABLED_KEY = 'lensisku.notificationsEnabled'

export function notificationsEnabledLocal(): boolean {
  if (typeof localStorage === 'undefined') return true
  return localStorage.getItem(NOTIFICATIONS_ENABLED_KEY) !== 'false'
}

export function setNotificationsEnabledLocal(enabled: boolean): void {
  localStorage.setItem(NOTIFICATIONS_ENABLED_KEY, enabled ? 'true' : 'false')
}

export function clearNotificationsEnabledLocal(): void {
  localStorage.removeItem(NOTIFICATIONS_ENABLED_KEY)
}
