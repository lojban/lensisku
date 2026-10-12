/// <reference types="vite/client" />

declare module 'vue-router' {
  interface RouteMeta {
    /** Hide the global app header (logo + nav). Layout heights in `App.vue` adjust automatically. */
    hideTopBar?: boolean
    /** Hide the global floating action menu (FAB). */
    hideFab?: boolean
    /**
     * Hide the App.vue public-domain notice.
     * When omitted/false on a `fullHeight` route, App Teleports the notice into
     * `publicDomainHost` (default: `.feed-page__body, .public-domain-host`).
     */
    hideFooter?: boolean
    /**
     * CSS selector for the fullHeight scroll column Teleport target.
     * @default '.feed-page__body, .public-domain-host'
     */
    publicDomainHost?: string
    /** Hide the sticky page footer (live chat / marquee). */
    hidePageFooter?: boolean
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
