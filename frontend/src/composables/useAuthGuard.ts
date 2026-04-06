import { jwtDecode } from 'jwt-decode'

// Define a basic type for the decoded JWT payload
interface DecodedToken {
  exp: number
  // Add other expected properties if known, e.g., username, authorities
  [key: string]: any
}

// Define a basic type for the auth instance structure based on usage
interface AuthInstance {
  state: {
    isLoggedIn: boolean
  }
  checkAuthStatus: () => Promise<boolean> // Assumes checkAuthStatus returns a promise resolving to boolean
}

// Attempt to access globalAuth if it exists in the global scope or module context.
// This reflects the dependency from the original file.
// In a modular setup, this should ideally be passed as an argument or imported.
declare const globalAuth: AuthInstance | null | undefined

/**
 * Checks the authentication state, suitable for use in route guards.
 * Reads directly from localStorage if the main auth instance isn't available.
 * Note: Relies on a 'globalAuth' instance potentially being available in the scope.
 *
 * @returns {Promise<boolean | undefined>} True if authenticated, false if not, undefined if run in a non-browser environment.
 */
export async function checkAuthStateForGuard(): Promise<boolean | undefined> {
  if (typeof window === 'undefined') {
    console.warn('checkAuthStateForGuard called in non-browser environment.')
    return undefined
  }

  // Check if globalAuth is defined and not null
  const authInstance = typeof globalAuth !== 'undefined' ? globalAuth : null

  if (!authInstance) {
    // Path 1: No globalAuth instance, check localStorage directly
    const accessToken = localStorage.getItem('accessToken')
    if (!accessToken) {
      return false // Not logged in
    }

    try {
      const decoded = jwtDecode<DecodedToken>(accessToken)
      const now = Math.floor(Date.now() / 1000)
      const isValid = now < decoded.exp
      if (!isValid) {
        // Optional: Clean up expired token? Original didn't explicitly do this here.
        // localStorage.removeItem('accessToken');
        // localStorage.removeItem('refreshToken');
      }
      return isValid // Return based on expiry check
    } catch (error) {
      console.error('checkAuthStateForGuard: Error decoding token from localStorage.', error)
      // If token is invalid/corrupt, treat as not logged in
      localStorage.removeItem('accessToken') // Clean up invalid token
      localStorage.removeItem('refreshToken')
      return false
    }
  } else {
    // Path 2: globalAuth instance exists, use its checkAuthStatus method
    try {
      // The original called checkAuthStatus and then returned state.isLoggedIn.
      // Let's replicate that. checkAuthStatus might handle refresh internally.
      await authInstance.checkAuthStatus() // Ensure status is up-to-date
      const isLoggedIn = authInstance.state.isLoggedIn
      return isLoggedIn // Return the state after checking
    } catch (error) {
      console.error('checkAuthStateForGuard: Error during globalAuth.checkAuthStatus().', error)
      // If the check fails, assume not logged in for guard purposes
      return false
    }
  }
}
