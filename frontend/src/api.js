import axios from 'axios'

const apiBaseUrl = (import.meta.env.VITE_BASE_URL ?? '/api')

// Create axios instance with base URL
export const api = axios.create({
  baseURL: apiBaseUrl,
  responseType: 'json',
  timeout: 60000,
  transformResponse: [
    (data, headers) => {
      if (headers['content-type']?.includes('image/')) {
        return data
      }
      return typeof data === 'string' ? JSON.parse(data) : data
    },
  ],
})

let isRefreshing = false

let refreshSubscribers = []
let authInstance = null

export const setAuthInstance = (auth) => {
  authInstance = auth
}

api.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config

    if (error.response?.status === 401 && !originalRequest._retry) {
      // Don't try to refresh for auth-related endpoints
      const isAuthEndpoint = originalRequest.url === '/auth/login' || 
                            originalRequest.url === '/auth/logout' || 
                            originalRequest.url === '/auth/refresh';
      
      if (isAuthEndpoint) {
        if (originalRequest.url === '/auth/refresh' && authInstance) {
          authInstance.logout();
        }
        return Promise.reject(error);
      }

      if (isRefreshing) {
        return new Promise((resolve) => {
          refreshSubscribers.push((token) => {
            originalRequest.headers['Authorization'] = `Bearer ${token}`
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

        const refreshed = await authInstance.refreshAccessToken();
        isRefreshing = false; // Reset refreshing status

        if (refreshed) {
          refreshSubscribers.forEach((callback) => callback(authInstance.state.accessToken));
          refreshSubscribers = [];
          originalRequest.headers['Authorization'] = `Bearer ${authInstance.state.accessToken}`; // Ensure header is set for retry
          return api(originalRequest);
        } else {
          // If refresh failed, logout the user
          authInstance.logout()
          return Promise.reject(error)
        }
      } catch (err) {
        isRefreshing = false; // Reset refreshing status
        refreshSubscribers = [];

        // On refresh error, logout and redirect
        if (authInstance) {
          authInstance.logout();
        }
        return Promise.reject(err); // Propagate the error from the refresh attempt
      }
    }
    return Promise.reject(error)
  }
)

// request interceptor
api.interceptors.request.use(
  (config) => {
    if (typeof window === 'undefined') return config
    // Get the access token from localStorage
    const accessToken = localStorage.getItem('accessToken')

    // If token exists, add it to the request headers
    if (accessToken) {
      config.headers.Authorization = `Bearer ${accessToken}`
    }

    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// API endpoints
export const getMessage = (id) => api.get(`/mail/message/${id}`)
export const getMessageDetails = (id) => api.get(`/mail/message/${id}`)
export const voteSpamMessage = (messageId) =>
  api.post(`/mail/messages/${messageId}/spam-vote`)


// Bulk Import endpoints
export const getBulkImportClients = () => api.get('/jbovlaste/bulk-import/clients')

export const getBulkImportClientDefinitions = (clientId, params) =>
  api.get(`/jbovlaste/bulk-import/clients/${clientId}/definitions`, { params })

// Updated to use client_id for cancellation
export const cancelBulkImport = (clientId) => api.post(`/jbovlaste/bulk-import/cancel/${clientId}`)
export const deleteBulkDefinitions = (clientId) =>
  api.post(`/jbovlaste/bulk-import/delete/${clientId}`)

export const getThread = (params) => api.get('/mail/thread', { params })
export const searchMuplis = (params) => api.get('/muplis/search', { params })
export const searchDictionary = (params) => api.get('/dictionary/search', { params })

// Auth endpoints
export const login = (credentials) => api.post('/auth/login', credentials)
export const signup = (userData) => api.post('/auth/signup', userData)
export const performBackendLogout = () => {
  // Create a new request config with Authorization header
  const config = {
    headers: { Authorization: `Bearer ${localStorage.getItem('accessToken')}` }
  };
  // Explicitly pass the empty body and config
  return api.post('/auth/logout', {}, config);
}

// Profile endpoints
export const getProfile = () => api.get('/auth/profile')
export const updateProfile = (profileData) => api.put('/auth/profile', profileData)
export const getUserProfile = (username) => api.get(`/users/${username}/profile`)

// User endpoints
export const listUsers = (params) => api.get('/users', { params })

// Valsi endpoints
export const getLanguages = () => api.get('/language/languages')
export const fetchDefinitionsTypes = () => api.get(`/jbovlaste/types`)
export const analyzeWord = (word, sourceLangId = 1) => api.post('/language/analyze_word', { word, source_langid: sourceLangId })
export const addValsi = (valsiData) => api.post('/jbovlaste/valsi', valsiData)
export const searchDefinitions = (params, signal) => {
  const endpoint = params.semantic ? '/jbovlaste/semantic-search' : '/jbovlaste/definitions'
  // Ensure source_langid is included if provided
  const finalParams = { ...params };
  if (finalParams.source_langid === 1) { // Don't send default
    delete finalParams.source_langid;
  }
  return api.get(endpoint, { params: finalParams, signal })
}

// Fast search - forces use of fast_search_definitions endpoint regardless of login status
export const fastSearchDefinitions = (params, signal) => {
  // Ensure source_langid is included if provided
  const finalParams = { ...params, fast: true };
  if (finalParams.source_langid === 1) { // Don't send default
    delete finalParams.source_langid;
  }
  // Force fast search by including 'fast=true' parameter
  return api.get('/jbovlaste/definitions', { params: finalParams, signal })
}
export const getValsiDetails = (id) => api.get(`/jbovlaste/valsi/${id}`)
export const getValsiDefinitions = (id) => api.get(`/jbovlaste/valsi/${id}/definitions`)
export const validateMathJax = (text) => api.post('/language/validate_mathjax', { text })
export const updateValsi = (id, valsiData) => api.put(`/jbovlaste/valsi/${id}`, valsiData)
export const getDefinition = (id) => api.get(`/jbovlaste/definition/${id}`)
export const addComment = (body) => api.post(`/comments`, body)
export const fetchComments = (queryString) => api.get(`/comments/thread?${queryString}`)

export const addEtymology = (data) => api.post('/jbovlaste/etymology', data)
export const getEtymology = (id) => api.get(`/jbovlaste/etymology/${id}`)
export const updateEtymology = (id, data) => api.put(`/jbovlaste/etymology/${id}`, data)

export const getValsiAndDefinitionDetails = async (valsiId, definitionId) => {
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

// Recent changes
export const getRecentChanges = (params) => api.get('/jbovlaste/changes', { params })

// Voting endpoints
export const voteDefinition = (definitionId, downvote = false) =>
  api.post('/jbovlaste/vote', {
    definition_id: definitionId,
    downvote,
  })

export const getCurrentUserVote = (definitionId) => api.get(`/jbovlaste/vote/${definitionId}`)

export const getBulkVotes = (params) => api.post('/jbovlaste/votes', params)

export const getVersionHistory = (definitionId, params) =>
  api.get(`/versions/${definitionId}/history`, { params })

export const getVersionDiff = (fromVersion, toVersion) =>
  api.get(`/versions/diff`, {
    params: { from_version: fromVersion, to_version: toVersion },
  })

export const revertToVersion = (versionId) => api.post(`/versions/${versionId}/revert`)

// Get subscription state for a valsi
export const getSubscriptionState = (valsiId) => api.get(`/subscriptions/${valsiId}/state`)

// Subscribe to a valsi
export const subscribeToValsi = (valsiId, triggerType) =>
  api.post('/subscriptions/subscribe', {
    valsi_id: valsiId,
    trigger_type: triggerType,
  })

// Unsubscribe from a valsi
export const unsubscribeFromValsi = (valsiId, triggerType) =>
  api.post(`/subscriptions/${valsiId}/unsubscribe/${triggerType}`)

// Collection related API endpoints
export const getCollections = (params) => api.get('/collections', { params })

export const getPublicCollections = (params) => api.get('/collections/public', { params })

export const getCollection = (id) => api.get(`/collections/${id}`)

export const createCollection = (data) => api.post('/collections', data)

export const updateCollection = (id, data) => api.put(`/collections/${id}`, data)

export const deleteCollection = (id) => api.delete(`/collections/${id}`)

export const addCollectionItem = (collectionId, data) =>
  api.post(`/collections/${collectionId}/items`, data)

export const updateItemPosition = (collectionId, itemId, position) =>
  api.put(`/collections/${collectionId}/items/${itemId}/position`, {
    position,
  })

export const removeCollectionItem = (collectionId, itemId) =>
  api.delete(`/collections/${collectionId}/items/${itemId}`)

export const updateItemNotes = (collectionId, itemId, data) =>
  api.put(`/collections/${collectionId}/items/${itemId}/notes`, data)

export const updateItemImages = (collectionId, itemId, data) =>
  api.put(`/collections/${collectionId}/items/${itemId}/images`, data)

export const listCollectionItems = (collectionId, params, signal) =>
  api.get(`/collections/${collectionId}/items`, { params, signal })

export const searchItems = (params, signal) =>
  api.get(`/collections/${params.user_id}/search`, {
    params: {
      q: params.q,
    },
    signal,
  })

export const cloneCollection = (collectionId) => api.post(`/collections/${collectionId}/clone`)

export const mergeCollection = (data) => api.post(`/collections/merge`, data)

export const exportDictionary = (language, params) =>
  api.get(`/export/dictionary/${language}?${params}`, {
    responseType: 'blob',
    timeout: 300000, // 5 minutes for large exports
  })

/** Full collection export: GET /collections/{id}/export (collection + items + levels + flashcard directions) */
export const exportCollectionFull = (collectionId) =>
  api.get(`/collections/${collectionId}/export`, { timeout: 300000 })

/** Full import: POST /collections/import/full — creates new collection with items, flashcards, and levels */
export const importCollectionFull = (data) => api.post('/collections/import/full', data)

export const listCachedExports = () => api.get('/export/cached')

export const downloadCachedExport = (languageTag, format) =>
  api.get(`/export/cached/${languageTag}/${format}`, {
    responseType: 'blob',
  })

export const importCollectionFromJson = (collectionId, data) => api.post(`/collections/${collectionId}/import/json`, data)

export const deleteComment = async (commentId) => {
  return await api.delete(`/comments/${commentId}`)
}

export const toggleLike = (commentId, like) =>
  api.post('/comments/like', {
    comment_id: commentId,
    action: like,
  })

export const toggleBookmark = (commentId, bookmark) =>
  api.post('/comments/bookmark', {
    comment_id: commentId,
    action: bookmark,
  })

export const toggleReaction = async (commentId, reaction) =>
  api.post('/comments/reactions', {
    comment_id: commentId,
    reaction,
  })

export const getBookmarks = (params) => api.get('/comments/bookmarks', { params })
export const getLikes = (params) => api.get('/comments/likes', { params })
export const getMyReactions = (params) => api.get('/comments/reactions/my', { params })

export const getUserComments = (username, params) =>
  api.get(`/users/${username}/comments`, { params })

export const getUserDefinitions = (username, params) =>
  api.get(`/users/${username}/definitions`, { params })

export const getUserVotes = (params) => api.get(`/users/votes`, { params })

export const resendConfirmation = (email) => api.post('/auth/resend-confirmation', { email })
export const confirmEmail = (token) => api.post('/auth/confirm-email', { token })

export const requestPasswordReset = (email) => api.post('/auth/request_password_reset', { email })
export const restorePassword = (data) => api.post('/auth/restore_password', data)

export const createFlashcard = async (collectionId, data) => {
  return api.post(`/flashcards/${collectionId}`, data)
}

export const updateCardPosition = async (cardId, newPosition) => {
  return api.patch(`/flashcards/${cardId}/position`, {
    position: newPosition,
  })
}

export const getFlashcards = async (params) => {
  return api.get('/flashcards', { params })
}

export const getDueCards = async (params) => {
  return api.get('/flashcards/due', { params })
}

export const reviewFlashcard = async (data) => {
  return api.post(`/flashcards/${data.flashcard_id}/review`, data)
}

export const submitFillinAnswer = async (data) => {
  return api.post(`/flashcards/${data.flashcard_id}/fillin`, data)
}

export const deleteFlashcard = async (flashcardId) => {
  return api.delete(`/flashcards/${flashcardId}`)
}

export const snoozeFlashcard = async (flashcardId) => {
  return api.post(`/flashcards/${flashcardId}/snooze`)
}

export const importFromCollection = (data) => api.post('/flashcards/collection/import', data)

export const resetProgress = async (flashcardId) => {
  return api.post(`/flashcards/${flashcardId}/reset`)
}

export const getStreak = async (days = 7) => {
  return api.get('/flashcards/streak', { params: { days } })
}

export const list_threads = (params) => api.get('/comments/threads', { params })

export const list_comments = (params) => api.get('/comments/list', { params })

export const list_definitions = (params) => {
  // Ensure source_langid is included if provided
  const finalParams = { ...params };
  if (finalParams.source_langid === 1) { // Don't send default
    delete finalParams.source_langid;
  }
  return api.get('/jbovlaste/definitions/list', { params: finalParams })
}

export const addCardsToLevel = async (levelId, data) => {
  return api.post(`/flashcards/cards/${levelId}`, data)
}

export const createLevel = async (collectionId, data) => {
  return api.post(`/flashcards/levels/${collectionId}`, data)
}

export const updateLevel = async (levelId, data) => {
  return api.put(`/flashcards/levels/${levelId}`, data)
}

export const getLevels = async (collectionId) => {
  return api.get(`/flashcards/levels/${collectionId}`)
}

export const getLevelCards = async (levelId, page = 1, perPage = 10) => {
  return api.get(`/flashcards/levels/${levelId}/cards`, {
    params: {
      page,
      per_page: perPage,
    },
  })
}

export const removeCardFromLevel = async (levelId, flashcardId) => {
  return api.delete(`/flashcards/levels/${levelId}/cards/${flashcardId}`)
}

export const deleteLevel = async (levelId) => {
  return api.delete(`/flashcards/levels/${levelId}`)
}

export const getProfileImage = (username, options = { cached: false }) => {
  return `${apiBaseUrl}/users/${username}/profile-image?${options.cached ? '' : Date.now()}`
}

export const updateProfileImage = (imageData) => {
  return api.post('/users/profile-image', imageData)
}

export const removeProfileImage = () => {
  return api.delete('/users/profile-image')
}

export const getTrendingComments = (params) => api.get('/comments/trending', { params })

export const searchComments = (params, signal) => api.get('/comments/search', { params, signal })

// Get available user roles
export const getRoles = () => api.get('/auth/roles')
export const getPermissions = () => api.get('/auth/permissions')
export const createRole = (data) => api.post('/auth/roles', data)
export const updateRole = (roleName, data) => api.put(`/auth/roles/${roleName}`, data)
export const deleteRole = (roleName, permissions = []) =>
  api.delete(`/auth/roles/${roleName}`, { data: { permissions } })

export const initiatePasswordChange = (data) => api.post('/auth/change-password/initiate', data)

export const completePasswordChange = (data) => api.post('/auth/change-password/complete', data)

export const updateItemNotesImages = async (collectionId, itemId, data) => {
  return await api.put(`/collections/${collectionId}/items/${itemId}/images`, data)
}

export const getItemImage = async (collectionId, itemId, side) => {
  return await api.get(`/collections/${collectionId}/items/${itemId}/image/${side}`, {
    responseType: 'blob',
  })
}

// Payment endpoints
export const createPayment = (paymentData) => api.post('/payments', paymentData)

export const getBalance = () => api.get('/payments/balance')

export const assignRole = (user_id, role) => api.post('/auth/assign-role', { user_id, role })

export const deleteDefinition = (id) => api.delete(`/jbovlaste/definition/${id}`)

// Definition Linking
export const linkDefinitions = (definitionId, translationId) =>
  api.post('/jbovlaste/definitions/link', {
    definition_id: parseInt(definitionId),
    translation_id: parseInt(translationId),
  })

export const unlinkDefinitions = (definitionId, translationId) =>
  api.delete(`/jbovlaste/definitions/link/${definitionId}/${translationId}`)

export const getDefinitionTranslations = (definitionId) =>
  api.get(`/jbovlaste/definitions/${definitionId}/translations`)

export const exportLinkedPairs = (fromLang, toLang) =>
  api.get('/jbovlaste/definitions/export-pairs', {
    params: { from_lang: fromLang, to_lang: toLang },
    responseType: 'blob',
  })
