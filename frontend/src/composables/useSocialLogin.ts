import { localePrefixRegex } from '@/config/locales'
import { completeOAuth, listOAuthProviders, startOAuthAuthorize } from '@/api'

export type SocialProviderId = 'github' | 'google'

export function isAuthRedirect(path: string): boolean {
  try {
    const pathname = new URL(path, window.location.origin).pathname
    const normalized = pathname.replace(localePrefixRegex, '') || '/'
    return (
      normalized === '/login' ||
      normalized === '/signup' ||
      normalized.startsWith('/login/') ||
      normalized.startsWith('/signup/') ||
      normalized.startsWith('/lingo/login') ||
      normalized.startsWith('/lingo/signup') ||
      normalized.startsWith('/oauth/')
    )
  } catch {
    return false
  }
}

export async function fetchConfiguredSocialProviders(): Promise<string[]> {
  const response = await listOAuthProviders()
  return Array.isArray(response.data.providers) ? response.data.providers : []
}

export async function beginSocialLogin(provider: string, returnTo?: string): Promise<void> {
  const response = await startOAuthAuthorize(provider, returnTo)
  const url = response.data.authorize_url
  if (!url) {
    throw new Error('Missing authorization URL')
  }
  window.location.assign(url)
}

export async function finishSocialLogin(provider: string, code: string, state: string) {
  const response = await completeOAuth(provider, { code, state })
  return response.data
}
