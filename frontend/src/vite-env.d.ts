/// <reference types="vite/client" />

/** Set in `main.ts` when MathJax loads; used by `LazyMathJax` and dictionary flash. */
declare global {
  interface Window {
    MathJax?: {
      typesetPromise?: (nodes?: unknown) => Promise<unknown>
    }
  }
}

export {}
