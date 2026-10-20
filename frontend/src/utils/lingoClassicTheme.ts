import type { RouteLocationNormalizedLoaded } from 'vue-router'

/**
 * Map a Lingo route to the equivalent classic (collections / flashcards) path.
 * Returns a locale-free path for `useLocalePath()`.
 *
 * | Lingo                                      | Classic                         |
 * |--------------------------------------------|---------------------------------|
 * | `/lingo/courses` (and other `/lingo/*`)    | `/collections`                  |
 * | `/collections/:id/lingo/levels`            | `/collections/:id/levels`       |
 * | `/collections/:id/lingo/study`             | `/collections/:id/flashcards/study` (+ query) |
 */
export function classicThemePathForRoute(
  route: Pick<RouteLocationNormalizedLoaded, 'path' | 'query' | 'params'>
): string {
  const path = route.path
  const collectionId =
    typeof route.params.collectionId === 'string'
      ? route.params.collectionId
      : Array.isArray(route.params.collectionId)
        ? route.params.collectionId[0]
        : null

  if (collectionId && /\/lingo\/study\/?$/.test(path)) {
    const qs = queryString(route.query)
    return `/collections/${collectionId}/flashcards/study${qs}`
  }

  if (collectionId && /\/lingo\/levels\/?$/.test(path)) {
    return `/collections/${collectionId}/levels`
  }

  // Catalog, learn redirect, leaderboard, quests, shop, auth, …
  return '/collections'
}

function queryString(query: RouteLocationNormalizedLoaded['query']): string {
  const params = new URLSearchParams()
  for (const [key, value] of Object.entries(query)) {
    if (value == null) continue
    if (Array.isArray(value)) {
      for (const item of value) {
        if (item != null) params.append(key, String(item))
      }
    } else {
      params.set(key, String(value))
    }
  }
  const s = params.toString()
  return s ? `?${s}` : ''
}
