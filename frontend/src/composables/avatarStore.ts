import { getProfileImage } from '@/api'

const avatarCache = new Map<string, boolean>()
const pendingRequests = new Map<string, Promise<boolean>>()

export const useAvatarStore = () => {
  const checkProfileImage = async (username: string): Promise<boolean> => {
    if (avatarCache.has(username)) {
      return avatarCache.get(username) as boolean
    }

    if (pendingRequests.has(username)) {
      return pendingRequests.get(username) as Promise<boolean>
    }

    const requestPromise = (async () => {
      try {
        const response = await fetch(getProfileImage(username, { cached: true }))
        const hasAvatar = response.ok
        avatarCache.set(username, hasAvatar)
        return hasAvatar
      } catch (error) {
        console.error(`Failed to fetch avatar for ${username}:`, error)
        return false
      } finally {
        pendingRequests.delete(username)
      }
    })()

    pendingRequests.set(username, requestPromise)
    return requestPromise
  }

  const getProfileImageUrl = (username: string): string => {
    return getProfileImage(username, { cached: true })
  }

  return {
    checkProfileImage,
    getProfileImageUrl,
    _avatarCache: avatarCache,
  }
}
