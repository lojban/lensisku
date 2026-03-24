/**
 * Search Queue Utility — prevents race conditions where older search results
 * overwrite newer ones (AbortController + latest-request wins).
 */

export class SearchQueue {
  currentRequestId: number | null = null
  abortController: AbortController | null = null
  pendingResults: Array<{ requestId: number; processFn: () => void }> = []

  createRequest(): { requestId: number; signal: AbortSignal } {
    if (this.abortController) {
      this.abortController.abort()
    }

    this.abortController = new AbortController()
    this.currentRequestId = Date.now() + Math.random()
    this.pendingResults = []

    return {
      requestId: this.currentRequestId,
      signal: this.abortController.signal,
    }
  }

  shouldProcess(requestId: number): boolean {
    return requestId === this.currentRequestId
  }

  addPendingResult(requestId: number, processFn: () => void): void {
    this.pendingResults.push({ requestId, processFn })
  }

  flushPendingResults(): void {
    if (this.pendingResults.length === 0) return

    this.pendingResults.sort((a, b) => b.requestId - a.requestId)

    const latestResult = this.pendingResults.find(
      (result) => result.requestId === this.currentRequestId
    )

    if (latestResult) {
      latestResult.processFn()
    }

    this.pendingResults = []
  }

  cancel(): void {
    if (this.abortController) {
      this.abortController.abort()
      this.abortController = null
    }
    this.currentRequestId = null
    this.pendingResults = []
  }

  hasActiveRequest(): boolean {
    return this.currentRequestId !== null
  }
}

export async function executeSearch<T>(
  queue: SearchQueue,
  searchFn: (signal: AbortSignal) => Promise<T>,
  onResult: (result: T) => void
): Promise<void> {
  const { requestId, signal } = queue.createRequest()

  try {
    const result = await searchFn(signal)

    if (queue.shouldProcess(requestId)) {
      onResult(result)
      queue.flushPendingResults()
    } else {
      queue.addPendingResult(requestId, () => onResult(result))
      queue.flushPendingResults()
    }
  } catch (error: unknown) {
    const err = error as { name?: string; code?: string }
    if (err.name === 'AbortError' || err.code === 'ERR_CANCELED') {
      return
    }

    if (queue.shouldProcess(requestId)) {
      throw error
    }
  }
}
