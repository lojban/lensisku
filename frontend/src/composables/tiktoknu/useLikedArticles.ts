import { ref, watch, computed } from 'vue'

import type { WikiArticle } from '../../types/wiki'

const LIKED_KEY = 'likedArticles'

export function useLikedArticles() {
  if (typeof window === 'undefined')
    return {
      likedArticles: ref([]),
      toggleLike: () => {},
      isLiked: () => false,
      likedArticlesCount: 0,
    }

  const likedArticles = ref<WikiArticle[]>(JSON.parse(localStorage.getItem(LIKED_KEY) || '[]'))

  watch(
    likedArticles,
    (newVal) => {
      localStorage.setItem(LIKED_KEY, JSON.stringify(newVal))
    },
    { deep: true }
  )

  const toggleLike = (article: WikiArticle) => {
    const index = likedArticles.value.findIndex((a) => a.pageid === article.pageid)
    if (index > -1) {
      likedArticles.value.splice(index, 1)
    } else {
      likedArticles.value.push(article)
    }
  }

  const isLiked = (pageid: number) =>
    likedArticles.value.some((article) => article.pageid === pageid)

  const likedArticlesCount = computed(() => likedArticles.value.length)

  return { likedArticles, toggleLike, isLiked, likedArticlesCount }
}
