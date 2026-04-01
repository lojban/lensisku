import axios, {
  AxiosHeaders,
  type AxiosInstance,
  type AxiosResponse,
  type InternalAxiosRequestConfig,
  type RawAxiosResponseHeaders,
} from 'axios'

const apiBaseUrl = import.meta.env.VITE_BASE_URL ?? '/api'

/** Injected by useAuth / setAuthInstance for token refresh on 401. */
export interface ApiAuthInstance {
  state: { accessToken: string }
  refreshAccessToken: () => Promise<boolean | undefined>
  logout: () => void | Promise<void>
}

type RetryableRequestConfig = InternalAxiosRequestConfig & { _retry?: boolean }

let isRefreshing = false
const refreshSubscribers: Array<(token: string) => void> = []
let authInstance: ApiAuthInstance | null = null

export const setAuthInstance = (auth: ApiAuthInstance | null) => {
  authInstance = auth
}

// Create axios instance with base URL
export const api: AxiosInstance = axios.create({
  baseURL: apiBaseUrl,
  responseType: 'json',
  timeout: 60000,
  transformResponse: [
    (data: unknown, headers?: RawAxiosResponseHeaders) => {
      const ct = headers?.['content-type']
      const contentType = Array.isArray(ct) ? ct[0] : ct
      if (typeof contentType === 'string' && contentType.includes('image/')) {
        return data
      }
      return typeof data === 'string' ? JSON.parse(data) : data
    },
  ],
})

api.interceptors.response.use(
  (response: AxiosResponse) => response,
  async (error: unknown) => {
    const err = error as { config?: RetryableRequestConfig; response?: { status?: number } }
    const originalRequest = err.config
    if (!originalRequest) {
      return Promise.reject(error)
    }

    if (err.response?.status === 401 && !originalRequest._retry) {
      const isAuthEndpoint =
        originalRequest.url === '/auth/login' ||
        originalRequest.url === '/auth/logout' ||
        originalRequest.url === '/auth/refresh'

      if (isAuthEndpoint) {
        if (originalRequest.url === '/auth/refresh' && authInstance) {
          authInstance.logout()
        }
        return Promise.reject(error)
      }

      if (isRefreshing) {
        return new Promise((resolve) => {
          refreshSubscribers.push((token: string) => {
            if (!originalRequest.headers) {
              originalRequest.headers = new AxiosHeaders()
            }
            originalRequest.headers.Authorization = `Bearer ${token}`
            resolve(api(originalRequest))
          })
        })
      }

      originalRequest._retry = true
      isRefreshing = true

      try {
        if (!authInstance) {
          throw new Error('Auth instance not initialized')
        }

        const refreshed = await authInstance.refreshAccessToken()
        isRefreshing = false

        if (refreshed) {
          refreshSubscribers.forEach((callback) => callback(authInstance!.state.accessToken))
          refreshSubscribers.length = 0
          if (!originalRequest.headers) {
            originalRequest.headers = new AxiosHeaders()
          }
          originalRequest.headers.Authorization = `Bearer ${authInstance.state.accessToken}`
          return api(originalRequest)
        } else {
          authInstance.logout()
          return Promise.reject(error)
        }
      } catch (refreshErr) {
        isRefreshing = false
        refreshSubscribers.length = 0

        if (authInstance) {
          authInstance.logout()
        }
        return Promise.reject(refreshErr)
      }
    }
    return Promise.reject(error)
  }
)

api.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    if (typeof window === 'undefined') return config
    const accessToken = localStorage.getItem('accessToken')

    if (accessToken) {
      if (!config.headers) {
        config.headers = new AxiosHeaders()
      }
      config.headers.Authorization = `Bearer ${accessToken}`
    }

    return config
  },
  (error: unknown) => Promise.reject(error)
)

// API endpoints
export const getMessageDetails = (id: number | string) => api.get(`/mail/message/${id}`)
export const voteSpamMessage = (messageId: number | string) =>
  api.post(`/mail/messages/${messageId}/spam-vote`)

export const getBulkImportClients = () => api.get('/jbovlaste/bulk-import/clients')

export const getBulkImportClientDefinitions = (
  clientId: string | number,
  params?: Record<string, unknown>
) => api.get(`/jbovlaste/bulk-import/clients/${clientId}/definitions`, { params })

export const cancelBulkImport = (clientId: string | number) =>
  api.post(`/jbovlaste/bulk-import/cancel/${clientId}`)
export const deleteBulkDefinitions = (clientId: string | number) =>
  api.post(`/jbovlaste/bulk-import/delete/${clientId}`)

export const getThread = (params?: Record<string, unknown>) => api.get('/mail/thread', { params })

export const login = (credentials: Record<string, unknown>) => api.post('/auth/login', credentials)
export const signup = (userData: Record<string, unknown>) => api.post('/auth/signup', userData)
export const performBackendLogout = () => {
  const config = {
    headers: { Authorization: `Bearer ${localStorage.getItem('accessToken')}` },
  }
  return api.post('/auth/logout', {}, config)
}

export const getProfile = () => api.get('/auth/profile')
export const updateProfile = (profileData: Record<string, unknown>) =>
  api.put('/auth/profile', profileData)
export const getUserProfile = (username: string) => api.get(`/users/${username}/profile`)

export const listUsers = (params?: Record<string, unknown>) => api.get('/users', { params })

export const getLanguages = () => api.get('/language/languages')
export const fetchDefinitionsTypes = () => api.get(`/jbovlaste/types`)
export const analyzeWord = (word: string, sourceLangId = 1) =>
  api.post('/language/analyze_word', { word, source_langid: sourceLangId })
export const addValsi = (valsiData: Record<string, unknown>) =>
  api.post('/jbovlaste/valsi', valsiData)
export const searchDefinitions = (params: Record<string, unknown>, signal?: AbortSignal) => {
  const endpoint = params.semantic ? '/jbovlaste/semantic-search' : '/jbovlaste/definitions'
  const finalParams = { ...params }
  if (finalParams.source_langid === 1) {
    delete finalParams.source_langid
  }
  return api.get(endpoint, { params: finalParams, signal })
}

export const fastSearchDefinitions = (params: Record<string, unknown>, signal?: AbortSignal) => {
  const finalParams: Record<string, unknown> = { ...params, fast: true }
  if (finalParams.source_langid === 1) {
    delete finalParams.source_langid
  }
  return api.get('/jbovlaste/definitions', { params: finalParams, signal })
}
export const getValsiDetails = (id: number | string) => api.get(`/jbovlaste/valsi/${id}`)
export const getValsiDefinitions = (id: number | string) =>
  api.get(`/jbovlaste/valsi/${id}/definitions`)

export const getValsiSoundBlob = (idOrWord: string | number) =>
  api.get(`/jbovlaste/valsi/${encodeURIComponent(String(idOrWord))}/sound`, {
    responseType: 'blob',
  })
export const validateMathJax = (text: string) => api.post('/language/validate_mathjax', { text })
export const updateValsi = (id: number | string, valsiData: Record<string, unknown>) =>
  api.put(`/jbovlaste/valsi/${id}`, valsiData)
export const getDefinition = (id: number | string) => api.get(`/jbovlaste/definition/${id}`)
export const addComment = (body: Record<string, unknown>) => api.post(`/comments`, body)
export const fetchComments = (queryString: string) => api.get(`/comments/thread?${queryString}`)

export const getValsiAndDefinitionDetails = async (
  valsiId: number | string,
  definitionId: number | string | null | undefined
) => {
  const valsiRes = await getValsiDetails(valsiId)

  if (!definitionId) {
    return {
      valsi: valsiRes.data,
      definition: null,
    }
  }

  const defRes = await getDefinition(definitionId)
  return {
    valsi: valsiRes.data,
    definition: defRes.data,
  }
}

export const getRecentChanges = (params?: Record<string, unknown>, signal?: AbortSignal) =>
  api.get('/jbovlaste/changes', { params, signal })

export const voteDefinition = (definitionId: number | string, downvote = false) =>
  api.post('/jbovlaste/vote', {
    definition_id: definitionId,
    downvote,
  })

export const getBulkVotes = (params: Record<string, unknown>) =>
  api.post('/jbovlaste/votes', params)

export const getVersionHistory = (
  definitionId: number | string,
  params?: Record<string, unknown>
) => api.get(`/versions/${definitionId}/history`, { params })

export const getVersionDiff = (fromVersion: number | string, toVersion: number | string) =>
  api.get(`/versions/diff`, {
    params: { from_version: fromVersion, to_version: toVersion },
  })

export const revertToVersion = (versionId: number | string) =>
  api.post(`/versions/${versionId}/revert`)

export const getSubscriptionState = (valsiId: number | string) =>
  api.get(`/subscriptions/${valsiId}/state`)

export const subscribeToValsi = (valsiId: number | string, triggerType: string) =>
  api.post('/subscriptions/subscribe', {
    valsi_id: valsiId,
    trigger_type: triggerType,
  })

export const unsubscribeFromValsi = (valsiId: number | string, triggerType: string) =>
  api.post(`/subscriptions/${valsiId}/unsubscribe/${triggerType}`)

export const getCollections = (params?: Record<string, unknown>) =>
  api.get('/collections', { params })

export const getPublicCollections = (params?: Record<string, unknown>) =>
  api.get('/collections/public', { params })

export const getCollection = (id: number | string) => api.get(`/collections/${id}`)

export const createCollection = (data: Record<string, unknown>) => api.post('/collections', data)

export const updateCollection = (id: number | string, data: Record<string, unknown>) =>
  api.put(`/collections/${id}`, data)

export const deleteCollection = (id: number | string) => api.delete(`/collections/${id}`)

export const addCollectionItem = (collectionId: number | string, data: Record<string, unknown>) =>
  api.post(`/collections/${collectionId}/items`, data)

export const updateItemPosition = (
  collectionId: number | string,
  itemId: number | string,
  position: number
) =>
  api.put(`/collections/${collectionId}/items/${itemId}/position`, {
    position,
  })

export const removeCollectionItem = (collectionId: number | string, itemId: number | string) =>
  api.delete(`/collections/${collectionId}/items/${itemId}`)

export const bulkRemoveCollectionItems = (
  collectionId: number | string,
  itemIds: number[]
) => api.post(`/collections/${collectionId}/items/bulk-remove`, { item_ids: itemIds })

export const listCollectionItems = (
  collectionId: number | string,
  params?: Record<string, unknown>,
  signal?: AbortSignal
) => api.get(`/collections/${collectionId}/items`, { params, signal })

export const listCustomTextBulkItems = (collectionId: number | string) =>
  api.get(`/collections/${collectionId}/items/custom-text-bulk`)

export const bulkUpdateCustomTextItems = (collectionId: number | string, data: unknown) =>
  api.put(`/collections/${collectionId}/items/custom-text-bulk`, data)

export const searchItems = (
  params: { user_id: number | string; q: string },
  signal?: AbortSignal
) =>
  api.get(`/collections/${params.user_id}/search`, {
    params: {
      q: params.q,
    },
    signal,
  })

export const cloneCollection = (collectionId: number | string) =>
  api.post(`/collections/${collectionId}/clone`)

export const mergeCollection = (data: Record<string, unknown>) =>
  api.post(`/collections/merge`, data)

export const exportDictionary = (language: string, params: string) =>
  api.get(`/export/dictionary/${language}?${params}`, {
    responseType: 'blob',
    timeout: 300000,
  })

export const exportCollectionFull = (collectionId: number | string) =>
  api.get(`/collections/${collectionId}/export`, { timeout: 300000 })

export const importCollectionFull = (data: unknown) => api.post('/collections/import/full', data)

export const listCachedExports = () => api.get('/export/cached')

export const getApiBaseUrl = () => (import.meta.env.VITE_BASE_URL ?? '/api').replace(/\/$/, '')

/** GET list / POST create `/assistant/chats`. */
export const getAssistantChatsCollectionUrl = () => `${getApiBaseUrl()}/assistant/chats`

/** GET/PUT/DELETE `/assistant/chats/:id` — not the SSE stream. */
export const getAssistantChatUrl = (chatId: string) =>
  `${getApiBaseUrl()}/assistant/chats/${encodeURIComponent(chatId)}`

/**
 * POST streams `text/event-stream` SSE; use `fetch()` + `response.body`, not axios (JSON-only).
 */
export const getAssistantChatStreamPostUrl = (chatId: string) =>
  `${getAssistantChatUrl(chatId)}/stream`

/** Anonymous stream: POST full `messages` JSON (no DB chat id). */
export const getAssistantPublicStreamPostUrl = () => `${getApiBaseUrl()}/assistant/chat/stream`

export const getAuthHeaders = (): Record<string, string> => {
  const token = typeof window !== 'undefined' ? localStorage.getItem('accessToken') : null
  return token ? { Authorization: `Bearer ${token}` } : {}
}

export const deleteComment = async (commentId: number | string) => {
  return await api.delete(`/comments/${commentId}`)
}

export const toggleBookmark = (commentId: number | string, bookmark: boolean) =>
  api.post('/comments/bookmark', {
    comment_id: commentId,
    action: bookmark,
  })

export const toggleReaction = async (commentId: number | string, reaction: string) =>
  api.post('/comments/reactions', {
    comment_id: commentId,
    reaction,
  })

export const getBookmarks = (params?: Record<string, unknown>) =>
  api.get('/comments/bookmarks', { params })
export const getMyReactions = (params?: Record<string, unknown>) =>
  api.get('/comments/reactions/my', { params })

export const getUserComments = (username: string, params?: Record<string, unknown>) =>
  api.get(`/users/${username}/comments`, { params })

export const getUserDefinitions = (username: string, params?: Record<string, unknown>) =>
  api.get(`/users/${username}/definitions`, { params })

export const getUserVotes = (params?: Record<string, unknown>) =>
  api.get(`/users/votes`, { params })

export const resendConfirmation = (email: string) =>
  api.post('/auth/resend-confirmation', { email })
export const confirmEmail = (token: string) => api.post('/auth/confirm-email', { token })

export const requestPasswordReset = (email: string) =>
  api.post('/auth/request_password_reset', { email })
export const restorePassword = (data: Record<string, unknown>) =>
  api.post('/auth/restore_password', data)

export const createFlashcard = async (
  collectionId: number | string,
  data: Record<string, unknown>
) => {
  return api.post(`/flashcards/${collectionId}`, data)
}

export const updateCardPosition = async (cardId: number | string, newPosition: number) => {
  return api.patch(`/flashcards/${cardId}/position`, {
    position: newPosition,
  })
}

export const getFlashcards = async (params?: Record<string, unknown>) => {
  return api.get('/flashcards', { params })
}

export const getDueCards = async (params?: Record<string, unknown>) => {
  return api.get('/flashcards/due', { params })
}

export const reviewFlashcard = async (data: Record<string, unknown>) => {
  return api.post(`/flashcards/${data.flashcard_id}/review`, data)
}

export const submitFillinAnswer = async (data: Record<string, unknown>) => {
  return api.post(`/flashcards/${data.flashcard_id}/fillin`, data)
}

export const submitQuizAnswer = async (data: Record<string, unknown>) => {
  return api.post('/flashcards/quiz/submit', data)
}

export const deleteFlashcard = async (flashcardId: number | string) => {
  return api.delete(`/flashcards/${flashcardId}`)
}

export const snoozeFlashcard = async (flashcardId: number | string) => {
  return api.post(`/flashcards/${flashcardId}/snooze`)
}

export const importFromCollection = (data: Record<string, unknown>) =>
  api.post('/flashcards/collection/import', data)

export const getStreak = async (days = 7) => {
  return api.get('/flashcards/streak', { params: { days } })
}

export const mergeProgress = async (data: Record<string, unknown>) => {
  return api.post('/flashcards/progress/merge', data)
}

export const list_comments = (params?: Record<string, unknown>, signal?: AbortSignal) =>
  api.get('/comments/list', { params, signal })

export const list_definitions = (params: Record<string, unknown>, signal?: AbortSignal) => {
  const finalParams = { ...params }
  if (finalParams.source_langid === 1) {
    delete finalParams.source_langid
  }
  return api.get('/jbovlaste/definitions/list', { params: finalParams, signal })
}

export const addCardsToLevel = async (levelId: number | string, data: Record<string, unknown>) => {
  return api.post(`/flashcards/cards/${levelId}`, data)
}

export const createLevel = async (collectionId: number | string, data: Record<string, unknown>) => {
  return api.post(`/flashcards/levels/${collectionId}`, data)
}

export const updateLevel = async (levelId: number | string, data: Record<string, unknown>) => {
  return api.put(`/flashcards/levels/${levelId}`, data)
}

export const getLevels = async (collectionId: number | string) => {
  return api.get(`/flashcards/levels/${collectionId}`)
}

export const getCollectionFlashcardsPublic = async (collectionId: number | string) => {
  return api.get(`/collections/${collectionId}/flashcards`)
}

export const getLevelCards = async (levelId: number | string, page = 1, perPage = 10) => {
  return api.get(`/flashcards/levels/${levelId}/cards`, {
    params: {
      page,
      per_page: perPage,
    },
  })
}

export const removeCardFromLevel = async (
  levelId: number | string,
  flashcardId: number | string
) => {
  return api.delete(`/flashcards/levels/${levelId}/cards/${flashcardId}`)
}

export const deleteLevel = async (levelId: number | string) => {
  return api.delete(`/flashcards/levels/${levelId}`)
}

export const getProfileImage = (
  username: string,
  options: { cached?: boolean } = { cached: false }
) => {
  return `${apiBaseUrl}/users/${username}/profile-image?${options.cached ? '' : Date.now()}`
}

export const updateProfileImage = (imageData: FormData | Record<string, unknown>) => {
  return api.post('/users/profile-image', imageData)
}

export const removeProfileImage = () => {
  return api.delete('/users/profile-image')
}

export const getCollectionImage = (
  collectionId: number | string,
  options: { cached?: boolean } = { cached: false }
) => {
  return `${apiBaseUrl}/collections/${collectionId}/image?${options.cached ? '' : Date.now()}`
}

export const updateCollectionImage = (
  collectionId: number | string,
  imageData: Record<string, unknown>
) => {
  return api.post(`/collections/${collectionId}/image`, imageData)
}

export const removeCollectionImage = (collectionId: number | string) => {
  return api.delete(`/collections/${collectionId}/image`)
}

export const getTrendingComments = (params?: Record<string, unknown>) =>
  api.get('/comments/trending', { params })

export const searchWaves = (params?: Record<string, unknown>, signal?: AbortSignal) =>
  api.get('/waves/search', { params, signal })
export const list_wave_threads = (params?: Record<string, unknown>, signal?: AbortSignal) =>
  api.get('/waves/threads', { params, signal })

export const getRoles = () => api.get('/auth/roles')
export const getPermissions = () => api.get('/auth/permissions')
export const createRole = (data: Record<string, unknown>) => api.post('/auth/roles', data)
export const updateRole = (roleName: string, data: Record<string, unknown>) =>
  api.put(`/auth/roles/${roleName}`, data)
export const deleteRole = (roleName: string, permissions: unknown[] = []) =>
  api.delete(`/auth/roles/${roleName}`, { data: { permissions } })

export const initiatePasswordChange = (data: Record<string, unknown>) =>
  api.post('/auth/change-password/initiate', data)

export const completePasswordChange = (data: Record<string, unknown>) =>
  api.post('/auth/change-password/complete', data)

export const getItemImage = async (
  collectionId: number | string,
  itemId: number | string,
  side: string
) => {
  return await api.get(`/collections/${collectionId}/items/${itemId}/image/${side}`, {
    responseType: 'blob',
  })
}

export const getItemSoundBlob = (collectionId: number | string, itemId: number | string) => {
  return api.get(`/collections/${collectionId}/items/${itemId}/sound`, {
    responseType: 'blob',
  })
}

/** Authenticated. Synthesizes Lojban text to Ogg Opus (rate-limited server-side). */
export const generateKittenTts = (data: { text: string; voice: string; speed?: number }) =>
  api.post('/collections/kitten-tts', data, {
    responseType: 'blob',
    timeout: 120000,
  })

export const getBalance = () => api.get('/payments/balance')

export const assignRole = (user_id: number | string, role: string) =>
  api.post('/auth/assign-role', { user_id, role })

export const deleteDefinition = (id: number | string) => api.delete(`/jbovlaste/definition/${id}`)

export const linkDefinitions = (definitionId: number | string, translationId: number | string) =>
  api.post('/jbovlaste/definitions/link', {
    definition_id: parseInt(String(definitionId), 10),
    translation_id: parseInt(String(translationId), 10),
  })

export const unlinkDefinitions = (definitionId: number | string, translationId: number | string) =>
  api.delete(`/jbovlaste/definitions/link/${definitionId}/${translationId}`)

export const getDefinitionTranslations = (definitionId: number | string) =>
  api.get(`/jbovlaste/definitions/${definitionId}/translations`)

export const getDefinitionLink = (id: number | string) =>
  api.get(`/jbovlaste/definition-links/${id}`)

export const exportLinkedPairs = (fromLang: string | number, toLang: string | number) =>
  api.get('/jbovlaste/definitions/export-pairs', {
    params: { from_lang: fromLang, to_lang: toLang },
    responseType: 'blob',
  })
