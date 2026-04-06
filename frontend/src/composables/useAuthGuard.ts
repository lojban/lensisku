import { jwtDecode } from 'jwt-decode'

interface DecodedToken {
  exp: number
  [key: string]: any
}

interface AuthInstance {
  state: {
    isLoggedIn: boolean
  }
  checkAuthStatus: () => Promise<boolean>
}

/** Optional legacy bridge; prefer injecting auth in modular setups. */
declare const globalAuth: AuthInstance | null | undefined

/**
 * Route guard helper: true if authenticated, false if not, undefined in SSR / non-browser.
 * Falls back to localStorage when `globalAuth` is not present.
 */
export async function checkAuthStateForGuard(): Promise<boolean | undefined> {
  if (typeof window === 'undefined') {
    console.warn('checkAuthStateForGuard called in non-browser environment.')
    return undefined
  }

  const authInstance = typeof globalAuth !== 'undefined' ? globalAuth : null

  if (!authInstance) {
    const accessToken = localStorage.getItem('accessToken')
    if (!accessToken) {
      return false
    }

    try {
      const decoded = jwtDecode<DecodedToken>(accessToken)
      const now = Math.floor(Date.now() / 1000)
      const isValid = now < decoded.exp
      return isValid
    } catch (error) {
      console.error('checkAuthStateForGuard: Error decoding token from localStorage.', error)
      localStorage.removeItem('accessToken')
      localStorage.removeItem('refreshToken')
      return false
    }
  }

  try {
    await authInstance.checkAuthStatus()
    return authInstance.state.isLoggedIn
  } catch (error) {
    console.error('checkAuthStateForGuard: Error during globalAuth.checkAuthStatus().', error)
    return false
  }
}
