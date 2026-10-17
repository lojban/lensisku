import { provide, ref, type InjectionKey, type Ref } from 'vue'

/** Mount point for `PaginationComponent` when a page uses the feed-page footer. */
export const feedPaginationHostKey: InjectionKey<Ref<HTMLElement | null>> =
  Symbol('feedPaginationHost')

export function useFeedPaginationHost() {
  const host = ref<HTMLElement | null>(null)
  provide(feedPaginationHostKey, host)
  return host
}
