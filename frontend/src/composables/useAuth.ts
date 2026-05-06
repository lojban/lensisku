import { jwtDecode } from 'jwt-decode'
import { reactive, provide, inject, type InjectionKey } from 'vue'
import { useRouter } from 'vue-router'
import {
  setAuthInstance,
  api,
  performBackendLogout,
  mergeProgress,
  getApiBaseUrl,
  getAuthHeaders,
} from '@/api'

import { getAllProgressForMerge, clearAfterMerge } from '@/composables/useAnonymousProgress'

const ASSISTANT_CHATS_STORAGE_KEY = 'lensisku-assistant-chats-v1'

interface DecodedAccessToken {
  exp: number
  username: string
  authorities?: string[]
  role?: string
  email_confirmed?: boolean
}

export interface AuthState {
  isLoggedIn: boolean
  isLoading: boolean
  username: string
  accessToken: string
  refreshToken: string
  refreshAttempts: number
  lastRefreshTime: number | null
  authorities: string[]
  role: string
  email_confirmed: boolean
}

export interface AuthApi {
  state: AuthState
  login: (accessToken: string, refreshToken: string, username: string) => void
  logout: () => void
  checkAuthStatus: () => Promise<boolean | undefined>
  refreshAccessToken: () => Promise<boolean | undefined>
}

const authKey = Symbol() as InjectionKey<AuthApi>

const REFRESH_MARGIN = 5 * 60
const MAX_REFRESH_ATTEMPTS = 3
const TOKEN_VERIFY_INTERVAL = 30000

let isRefreshing = false
let refreshSubscribers: Array<(value: boolean) => void> = []

export function provideAuth(): AuthApi {
  const router = useRouter()

  const state = reactive<AuthState>({
    isLoggedIn: false,
    isLoading: true,
    username: '',
    accessToken: '',
    refreshToken: '',
    refreshAttempts: 0,
    lastRefreshTime: null,
    authorities: [],
    role: '',
    email_confirmed: false,
  })

  setTimeout(() => {
    void checkAuthStatus()
  }, 0)

  let refreshTimer: ReturnType<typeof setTimeout> | null = null
  let verificationTimer: ReturnType<typeof setInterval> | null = null
  let visibilityHandler: (() => void) | null = null

  const verifyAndRefreshToken = async (): Promise<boolean> => {
    if (typeof window === 'undefined') return false

    const accessToken = localStorage.getItem('accessToken')

    if (!accessToken) {
      return false
    }

    try {
      const decoded = jwtDecode<DecodedAccessToken>(accessToken)
      const now = Math.floor(Date.now() / 1000)

      if (decoded.exp - now < REFRESH_MARGIN) {
        return (await refreshAccessToken()) ?? false
      }

      return true
    } catch (error) {
      console.warn('Token validation failed:', error)
      return (await refreshAccessToken()) ?? false
    }
  }

  async function refreshAccessToken(): Promise<boolean | undefined> {
    if (state.refreshAttempts >= MAX_REFRESH_ATTEMPTS) {
      logout()
      return false
    }

    if (isRefreshing) {
      return new Promise((resolve) => {
        refreshSubscribers.push(resolve)
      })
    }

    isRefreshing = true

    try {
      const refreshToken = localStorage.getItem('refreshToken')
      if (!refreshToken) {
        logout()
        return false
      }

      const response = await api.post<{
        access_token?: string
        refresh_token?: string
      }>('/auth/refresh', {
        refresh_token: refreshToken,
      })

      if (response.data.access_token) {
        state.accessToken = response.data.access_token
        localStorage.setItem('accessToken', response.data.access_token)

        if (response.data.refresh_token) {
          state.refreshToken = response.data.refresh_token
          localStorage.setItem('refreshToken', response.data.refresh_token)
        }

        state.refreshAttempts = 0
        state.lastRefreshTime = Date.now()

        refreshSubscribers.forEach((callback) => callback(true))
        refreshSubscribers = []

        const decoded = jwtDecode<DecodedAccessToken>(response.data.access_token)
        state.username = decoded.username
        state.authorities = decoded.authorities || []
        state.role = decoded.role || ''
        state.email_confirmed = decoded.email_confirmed || false
        localStorage.setItem('username', decoded.username)
        scheduleTokenRefresh(decoded.exp)
        return true
      }
    } catch (error) {
      state.refreshAttempts++
      console.error(`Token refresh failed (attempt ${state.refreshAttempts}):`, error)

      refreshSubscribers.forEach((callback) => callback(false))
      refreshSubscribers = []

      if (state.refreshAttempts >= MAX_REFRESH_ATTEMPTS) {
        logout()
        return false
      }

      return false
    } finally {
      isRefreshing = false
    }

    return false
  }

  async function logout(): Promise<void> {
    performBackendLogout().catch((error: unknown) => {
      console.error('Backend logout failed. Proceeding with client-side logout.', error)
    })

    localStorage.removeItem('accessToken')
    localStorage.removeItem('refreshToken')
    localStorage.removeItem('username')

    state.isLoggedIn = false
    state.username = ''
    state.accessToken = ''
    state.refreshToken = ''
    state.refreshAttempts = 0
    state.lastRefreshTime = null
    state.authorities = []
    state.role = ''
    state.email_confirmed = false

    if (refreshTimer) {
      clearTimeout(refreshTimer)
      refreshTimer = null
    }

    if (verificationTimer) {
      clearInterval(verificationTimer)
      verificationTimer = null
    }
    if (visibilityHandler) {
      document.removeEventListener('visibilitychange', visibilityHandler)
      visibilityHandler = null
    }

    isRefreshing = false
    refreshSubscribers = []

    if (router) {
      void router.push('/login')
    } else {
      console.warn('Router instance not available in logout function.')
    }
  }

  function scheduleTokenRefresh(expiryTime: number): void {
    if (refreshTimer) {
      clearTimeout(refreshTimer)
    }

    const now = Math.floor(Date.now() / 1000)
    const timeUntilRefresh = Math.max(0, expiryTime - REFRESH_MARGIN - now)

    refreshTimer = setTimeout(() => {
      void refreshAccessToken()
    }, timeUntilRefresh * 1000)
  }

  const startTokenVerification = (): void => {
    if (verificationTimer) {
      clearInterval(verificationTimer)
    }

    verificationTimer = setInterval(async () => {
      const isValid = await verifyAndRefreshToken()
      if (!isValid && state.isLoggedIn) {
        console.warn('Token invalid during verification check, logging out')
        void logout()
      }
    }, TOKEN_VERIFY_INTERVAL)

    visibilityHandler = () => {
      if (document.visibilityState === 'visible') {
        void verifyAndRefreshToken()
      }
    }
    document.addEventListener('visibilitychange', visibilityHandler)
  }

  function login(accessToken: string, refreshToken: string, username: string): void {
    localStorage.setItem('accessToken', accessToken)
    localStorage.setItem('refreshToken', refreshToken)
    localStorage.setItem('username', username)

    state.isLoggedIn = true
    state.username = username
    state.accessToken = accessToken
    state.refreshToken = refreshToken
    state.refreshAttempts = 0

    const decoded = parseToken(accessToken)
    if (decoded) {
      state.authorities = decoded.authorities || []
      state.role = decoded.role || ''
      state.email_confirmed = decoded.email_confirmed || false
      scheduleTokenRefresh(decoded.exp)
    } else {
      state.authorities = []
      state.role = ''
      state.email_confirmed = false
    }

    startTokenVerification()
    void mergeAnonymousProgressThenClear()
    void mergeAssistantChatsThenClear()
  }

  async function mergeAssistantChatsThenClear(): Promise<void> {
    try {
      const raw = localStorage.getItem(ASSISTANT_CHATS_STORAGE_KEY)
      if (!raw) return
      const data = JSON.parse(raw) as {
        sessions?: Array<{
          title?: string
          messages?: unknown[]
          primaryModelId?: string | null
          scrollTop?: number
        }>
      }
      const sessions = Array.isArray(data.sessions) ? data.sessions : []
      if (!sessions.length) {
        localStorage.removeItem(ASSISTANT_CHATS_STORAGE_KEY)
        return
      }
      const token = localStorage.getItem('accessToken')
      if (!token) return
      const base = getApiBaseUrl()
      const res = await fetch(`${base}/assistant/chats/import`, {
        method: 'POST',
        headers: {
          ...getAuthHeaders(),
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          sessions: sessions.map((s) => ({
            title: s.title || '',
            messages: s.messages ?? [],
            primaryModelId: s.primaryModelId ?? null,
            scrollTop: typeof s.scrollTop === 'number' ? s.scrollTop : 0,
          })),
        }),
      })
      if (!res.ok) {
        console.warn('Merge assistant chats failed:', await res.text())
        return
      }
      localStorage.removeItem(ASSISTANT_CHATS_STORAGE_KEY)
    } catch (err) {
      console.warn('Merge assistant chats failed:', err)
    }
  }

  async function mergeAnonymousProgressThenClear(): Promise<void> {
    try {
      const payloads = getAllProgressForMerge()
      for (const p of payloads) {
        await mergeProgress({ collection_id: p.collection_id, level_progress: p.level_progress })
      }
      if (payloads.length) clearAfterMerge()
    } catch (err) {
      console.warn('Merge anonymous progress failed:', err)
    }
  }

  function parseToken(token: string): DecodedAccessToken | null {
    try {
      return jwtDecode<DecodedAccessToken>(token)
    } catch (error) {
      console.error('Failed to decode token:', error)
      return null
    }
  }

  async function checkAuthStatus(): Promise<boolean | undefined> {
    if (typeof window === 'undefined') return

    state.isLoading = true
    try {
      const accessToken = localStorage.getItem('accessToken')
      const refreshToken = localStorage.getItem('refreshToken')

      if (!accessToken || !refreshToken) {
        state.isLoggedIn = false
        return false
      }

      const decoded = parseToken(accessToken)
      if (!decoded) {
        state.isLoggedIn = false
        return false
      }

      const now = Math.floor(Date.now() / 1000)
      if (now >= decoded.exp) {
        if (refreshToken) {
          state.refreshToken = refreshToken
          const refreshed = await refreshAccessToken()
          state.isLoggedIn = refreshed ?? false
          return refreshed
        } else {
          void logout()
          return false
        }
      } else {
        state.isLoggedIn = true
        state.username = decoded.username
        state.authorities = decoded.authorities || []
        state.role = decoded.role || ''
        state.email_confirmed = decoded.email_confirmed || false
        state.accessToken = accessToken
        state.refreshToken = refreshToken

        scheduleTokenRefresh(decoded.exp)
        startTokenVerification()
        return true
      }
    } finally {
      state.isLoading = false
    }
  }

  const auth: AuthApi = {
    state,
    login,
    logout,
    checkAuthStatus,
    refreshAccessToken,
  }

  setAuthInstance(auth)

  provide(authKey, auth)
  return auth
}

export function useAuth(): AuthApi {
  const auth = inject(authKey)
  if (!auth) {
    throw new Error('useAuth() called without provider')
  }
  return auth
}
