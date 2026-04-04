/**
 * Extract a human-readable message from an axios-style error response.
 * Handles JSON `{ message }`, `{ error }`, string bodies, and Actix JSON payload limit text.
 */
export function getApiErrorMessage(error: unknown): string | undefined {
  const err = error as {
    response?: { data?: unknown; status?: number }
    message?: string
  }
  const data = err.response?.data
  if (data == null || data === '') return undefined

  if (typeof data === 'string') {
    const trimmed = data.trim()
    if (trimmed.startsWith('{')) {
      try {
        const parsed = JSON.parse(trimmed) as { message?: string; error?: unknown }
        if (typeof parsed.message === 'string' && parsed.message) return parsed.message
        if (typeof parsed.error === 'string' && parsed.error) return parsed.error
      } catch {
        return trimmed
      }
    }
    return trimmed
  }

  if (typeof data === 'object' && data !== null) {
    const o = data as Record<string, unknown>
    if (typeof o.message === 'string' && o.message) return o.message
    if (typeof o.error === 'string' && o.error) return o.error
    if (o.error && typeof o.error === 'object' && o.error !== null) {
      const inner = o.error as Record<string, unknown>
      if (typeof inner.message === 'string' && inner.message) return inner.message
    }
  }

  return undefined
}

const PAYLOAD_LIMIT_PATTERN =
  /payload.*larger than allowed|larger than allowed.*limit|JSON payload.*larger than allowed/i

/** True when the server (or proxy) rejected the request body for exceeding size limits. */
export function isPayloadLimitError(error: unknown): boolean {
  const e = error as { response?: { status?: number }; message?: string }
  if (e.response?.status === 413) return true
  const raw = getApiErrorMessage(error)
  if (raw && PAYLOAD_LIMIT_PATTERN.test(raw)) return true
  if (e.message && PAYLOAD_LIMIT_PATTERN.test(e.message)) return true
  return false
}

/** User-facing message when saving a collection item (add/update) fails. */
export function getCollectionItemSaveErrorMessage(
  error: unknown,
  t: (key: string) => string
): string {
  if (isPayloadLimitError(error)) {
    return t('collectionDetail.payloadTooLarge')
  }
  const api = getApiErrorMessage(error)
  if (api?.trim()) return api
  return t('collectionDetail.itemSaveFailed')
}

/** Localized message for profile/collection image upload failures. */
export function uploadImageErrorMessage(
  error: unknown,
  t: (key: string) => string
): string {
  const status = (error as { response?: { status?: number } }).response?.status
  const raw = getApiErrorMessage(error)

  if (status === 413 || (raw && PAYLOAD_LIMIT_PATTERN.test(raw))) {
    return t('profile.uploadError.payloadTooLarge')
  }

  if (raw && raw.trim().length > 0) {
    return raw
  }

  return t('profile.uploadError.uploadFailed')
}
