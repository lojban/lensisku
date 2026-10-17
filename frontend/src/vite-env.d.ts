/// <reference types="vite/client" />

declare module 'vue-router' {
  interface RouteMeta {
    /** Hide the global app header (logo + nav). Layout heights in `App.vue` adjust automatically. */
    hideTopBar?: boolean
    /** Hide the global floating action menu (FAB). */
    hideFab?: boolean
    /**
     * Hide the public-domain notice (`PublicDomainNotice` reads this meta).
     * On `fullHeight` routes the notice lives in the page scroll host, not App.vue.
     */
    hideFooter?: boolean
  }
}

/** Set in `main.ts` when MathJax loads; used by `LazyMathJax` and dictionary flash. */
declare global {
  interface Window {
    MathJax?: {
      typesetPromise?: (nodes?: unknown) => Promise<unknown>
    }
  }
}

export {}
