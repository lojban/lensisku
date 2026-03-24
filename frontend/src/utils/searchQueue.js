/**
 * Search Queue Utility
 *
 * Manages search request queues to prevent race conditions where
 * older search results overwrite newer ones in the UI.
 *
 * Features:
 * - Tracks request IDs to identify the latest request
 * - Cancels previous requests using AbortController
 * - Only processes results if they match the latest request
 * - Flushes queue when multiple results arrive, showing only the last one
 */

export class SearchQueue {
  constructor() {
    this.currentRequestId = null
    this.abortController = null
    this.pendingResults = []
  }

  /**
   * Create a new search request
   * @returns {Object} Object with requestId and abortController
   */
  createRequest() {
    // Cancel any previous request
    if (this.abortController) {
      this.abortController.abort()
    }

    // Create new abort controller and request ID
    this.abortController = new AbortController()
    this.currentRequestId = Date.now() + Math.random() // Unique ID
    this.pendingResults = []

    return {
      requestId: this.currentRequestId,
      signal: this.abortController.signal,
    }
  }

  /**
   * Check if a result should be processed
   * @param {number} requestId - The request ID from createRequest
   * @returns {boolean} True if this is the latest request
   */
  shouldProcess(requestId) {
    return requestId === this.currentRequestId
  }

  /**
   * Add a pending result to the queue
   * @param {number} requestId - The request ID
   * @param {Function} processFn - Function to process the result
   */
  addPendingResult(requestId, processFn) {
    this.pendingResults.push({ requestId, processFn })
  }

  /**
   * Process all pending results, but only apply the latest one
   * This ensures that if multiple results arrive, we flush the queue
   * and only show the last result
   */
  flushPendingResults() {
    if (this.pendingResults.length === 0) return

    // Sort by requestId (newest first)
    this.pendingResults.sort((a, b) => b.requestId - a.requestId)

    // Find the latest valid result (matching currentRequestId)
    const latestResult = this.pendingResults.find(
      (result) => result.requestId === this.currentRequestId
    )

    // If we have a latest result, process it
    if (latestResult) {
      latestResult.processFn()
    }

    // Clear the queue
    this.pendingResults = []
  }

  /**
   * Cancel the current request and clear the queue
   */
  cancel() {
    if (this.abortController) {
      this.abortController.abort()
      this.abortController = null
    }
    this.currentRequestId = null
    this.pendingResults = []
  }

  /**
   * Check if there's an active request
   * @returns {boolean}
   */
  hasActiveRequest() {
    return this.currentRequestId !== null
  }
}

/**
 * Helper function to wrap an async search function with queue management
 * @param {SearchQueue} queue - The search queue instance
 * @param {Function} searchFn - The async search function that returns a promise
 * @param {Function} onResult - Callback to process the result
 * @returns {Promise} The search promise
 */
export async function executeSearch(queue, searchFn, onResult) {
  const { requestId, signal } = queue.createRequest()

  try {
    const result = await searchFn(signal)

    // Check if this is still the latest request
    if (queue.shouldProcess(requestId)) {
      onResult(result)
      // Flush any other pending results (shouldn't happen, but safety check)
      queue.flushPendingResults()
    } else {
      // This is an older request, add to queue
      queue.addPendingResult(requestId, () => onResult(result))
      // Try to flush - if this is the latest, it will be processed
      queue.flushPendingResults()
    }
  } catch (error) {
    // Ignore abort errors
    if (error.name === 'AbortError' || error.code === 'ERR_CANCELED') {
      return
    }

    // Only process errors for the latest request
    if (queue.shouldProcess(requestId)) {
      throw error
    }
  }
}
