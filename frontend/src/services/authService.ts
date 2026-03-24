import { jwtDecode } from 'jwt-decode'

import { api } from '../api'

interface AccessTokenPayload {
  exp: number
}

class AuthService {
  async checkAuthStatus(): Promise<boolean | undefined> {
    if (typeof window === 'undefined') return

    const accessToken = localStorage.getItem('accessToken')
    if (!accessToken) return false

    try {
      const decoded = jwtDecode<AccessTokenPayload>(accessToken)
      const now = Math.floor(Date.now() / 1000)

      if (decoded.exp <= now) {
        return await this.refreshAccessToken()
      }

      return true
    } catch (error) {
      console.warn('Token validation failed:', error)
      return await this.refreshAccessToken()
    }
  }

  async refreshAccessToken(): Promise<boolean | undefined> {
    if (typeof window === 'undefined') return

    try {
      const refreshToken = localStorage.getItem('refreshToken')
      if (!refreshToken) return false

      const response = await api.post<{
        access_token?: string
        refresh_token?: string
      }>('/auth/refresh', {
        refresh_token: refreshToken,
      })

      if (response.data.access_token) {
        localStorage.setItem('accessToken', response.data.access_token)
        if (response.data.refresh_token) {
          localStorage.setItem('refreshToken', response.data.refresh_token)
        }
        return true
      }
    } catch (error) {
      console.error('Token refresh failed:', error)
      localStorage.removeItem('accessToken')
      localStorage.removeItem('refreshToken')
      return false
    }
    return false
  }

  isLoggedIn(): boolean {
    return !!localStorage.getItem('accessToken')
  }
}

export const authService = new AuthService()
