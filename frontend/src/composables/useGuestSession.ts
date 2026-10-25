/**
 * Backend guest study session: disposable user + JWT.
 * localStorage holds only the session tokens (not progress blobs).
 */

import axios from 'axios'
import { getApiBaseUrl } from '@/api'

const STORAGE_KEY = 'lensisku_guest_session'

export type GuestSessionTokens = {
  access_token: string
  refresh_token: string
}

function readSession(): GuestSessionTokens | null {
  if (typeof window === 'undefined') return null
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return null
    const parsed = JSON.parse(raw) as Partial<GuestSessionTokens>
    if (
      typeof parsed.access_token === 'string' &&
      parsed.access_token &&
      typeof parsed.refresh_token === 'string' &&
      parsed.refresh_token
    ) {
      return { access_token: parsed.access_token, refresh_token: parsed.refresh_token }
    }
  } catch {
    /* ignore */
  }
  return null
}

function writeSession(tokens: GuestSessionTokens): void {
  if (typeof window === 'undefined') return
  localStorage.setItem(STORAGE_KEY, JSON.stringify(tokens))
}

export function getGuestAccessToken(): string | null {
  return readSession()?.access_token ?? null
}

export function getGuestRefreshToken(): string | null {
  return readSession()?.refresh_token ?? null
}

export function clearGuestSession(): void {
  if (typeof window === 'undefined') return
  localStorage.removeItem(STORAGE_KEY)
}

/** True when a guest study session exists (user still appears logged out in UI). */
export function hasGuestSession(): boolean {
  return !!getGuestAccessToken()
}

/** True when the JWT's `exp` is in the past (or within 30 s). Unparseable → false. */
function isJwtExpired(token: string): boolean {
  try {
    const payload = token.split('.')[1]
    if (!payload) return false
    const json = JSON.parse(atob(payload.replace(/-/g, '+').replace(/_/g, '/')))
    return typeof json.exp === 'number' && json.exp * 1000 < Date.now() + 30_000
  } catch {
    return false
  }
}

let ensurePromise: Promise<GuestSessionTokens | null> | null = null

/**
 * Ensure a guest backend session exists. No-op when the user already has a real login token.
 */
export async function ensureGuestSession(): Promise<GuestSessionTokens | null> {
  if (typeof window === 'undefined') return null
  if (localStorage.getItem('accessToken')) return null

  const existing = readSession()
  if (existing && !isJwtExpired(existing.access_token)) return existing
  // Expired guest access token (e.g. a session left over from an earlier visit): refresh it,
  // or start a new guest session if the refresh token is no longer accepted. Otherwise the
  // optional-auth levels API silently answers as anonymous (only level 1 unlocked) while
  // study calls 401.
  if (existing) {
    const refreshed = await refreshGuestAccessToken()
    if (refreshed) return readSession()
  }

  if (ensurePromise) return ensurePromise

  ensurePromise = (async () => {
    try {
      const base = getApiBaseUrl()
      const { data } = await axios.post<GuestSessionTokens>(
        `${base}/auth/guest`,
        {},
        { headers: { Accept: 'application/json' } }
      )
      if (!data?.access_token || !data?.refresh_token) return null
      writeSession(data)
      return data
    } catch (err) {
      console.warn('Failed to create guest study session:', err)
      return null
    } finally {
      ensurePromise = null
    }
  })()

  return ensurePromise
}

/** Refresh guest access token; clears session on failure. */
export async function refreshGuestAccessToken(): Promise<string | null> {
  const session = readSession()
  if (!session?.refresh_token) return null
  try {
    const base = getApiBaseUrl()
    const { data } = await axios.post<{
      access_token?: string
      refresh_token?: string
    }>(`${base}/auth/refresh`, { refresh_token: session.refresh_token })
    if (!data?.access_token) {
      clearGuestSession()
      return null
    }
    writeSession({
      access_token: data.access_token,
      refresh_token: data.refresh_token || session.refresh_token,
    })
    return data.access_token
  } catch {
    clearGuestSession()
    return null
  }
}

/**
 * After real login: merge guest progress into the account and drop the guest session.
 * Call while real `accessToken` is already set.
 */
export async function claimGuestProgressIntoAccount(): Promise<void> {
  const guestAccess = getGuestAccessToken()
  if (!guestAccess) return
  const realAccess = localStorage.getItem('accessToken')
  if (!realAccess) return
  try {
    const base = getApiBaseUrl()
    await axios.post(
      `${base}/flashcards/progress/claim-guest`,
      { guest_access_token: guestAccess },
      {
        headers: {
          Accept: 'application/json',
          Authorization: `Bearer ${realAccess}`,
        },
      }
    )
  } catch (err) {
    console.warn('Claim guest progress failed:', err)
  } finally {
    clearGuestSession()
  }
}
