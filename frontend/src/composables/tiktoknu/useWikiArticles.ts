import { ref } from 'vue'
import type { Ref } from 'vue'

import type { WikiArticle } from '../../types/wiki'

class PreloadQueue {
  private queue: string[] = []
  private isProcessing = false

  async processNext() {
    if (this.queue.length === 0) {
      this.isProcessing = false
      return
    }

    this.isProcessing = true
    const src = this.queue.shift()!

    try {
      const img = new Image()
      img.src = src
      await new Promise((resolve, reject) => {
        img.onload = resolve
        img.onerror = reject
      })
    } finally {
      this.processNext() // Continue processing next item
    }
  }

  add(src: string) {
    this.queue.push(src)
    if (!this.isProcessing) {
      this.processNext()
    }
  }
}

// Single instance for all preloading
const preloadQueue = new PreloadQueue()

const preloadImage = (src: string) => {
  preloadQueue.add(src)
}

export function useWikiArticles(currentLanguage: Ref<{ api: string }>) {
  const articles = ref<WikiArticle[]>([])
  const loading = ref(false)
  const buffer = ref<WikiArticle[]>([])

  const fetchArticles = async (forBuffer = false) => {
    if (loading.value) return
    loading.value = true

    try {
      const response = await fetch(
        currentLanguage.value.api +
          new URLSearchParams({
            action: 'query',
            format: 'json',
            generator: 'random',
            grnnamespace: '0',
            prop: 'extracts|pageimages|info',
            inprop: 'url',
            grnlimit: '20',
            exintro: '1',
            exlimit: 'max',
            exsentences: '5',
            explaintext: '1',
            piprop: 'thumbnail',
            pithumbsize: '400',
            origin: '*',
          })
      )

      const data = await response.json()
      const newArticles = Object.values<WikiArticle>(data.query.pages)
        .map((page) => ({
          title: page.title,
          extract: page.extract,
          pageid: page.pageid,
          thumbnail: page.thumbnail,
          canonicalurl: page.canonicalurl,
        }))
        .filter(
          (article: WikiArticle) =>
            article.thumbnail?.source && article.canonicalurl && article.extract
        )

      // Queue images for sequential preloading in background
      newArticles.forEach((article: WikiArticle) => {
        if (article.thumbnail) {
          preloadImage(article.thumbnail.source)
        }
      })

      if (forBuffer) {
        buffer.value = newArticles
      } else {
        // Keep only last 30 articles
        articles.value = [...articles.value, ...newArticles].slice(-30)
        fetchArticles(true)
      }
    } catch (error) {
      console.error('Error fetching articles:', error)
    } finally {
      loading.value = false
    }
  }

  return { articles, loading, fetchArticles }
}
