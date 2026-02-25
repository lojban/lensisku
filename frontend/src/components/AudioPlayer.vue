<template>
  <div class="inline-flex items-center gap-2">
    <button
      class="btn-get"
      :disabled="isLoading"
      @click="togglePlay"
    >
      <PauseCircle
        v-if="isPlaying"
        class="w-4 h-4"
      />
      <Loader
        v-else-if="isLoading"
        class="w-4 h-4 animate-spin"
      />
      <PlayCircle
        v-else
        class="w-4 h-4"
      />
    </button>
  </div>
</template>

<script setup>
  import { PlayCircle, PauseCircle, Loader } from 'lucide-vue-next';
  import { ref, onMounted, onBeforeUnmount } from 'vue';
  import { useI18n } from 'vue-i18n';

  import { useError } from '@/composables/useError';
  import { getItemSoundBlob, getValsiSoundBlob } from '@/api';

  const { showError, clearError } = useError();
  const { t } = useI18n();
  const props = defineProps({
    url: {
      type: String,
      required: true,
    },
    /** When set with itemId, load sound via api (Bearer sent). Omit for external URLs. */
    collectionId: { type: Number, default: null },
    itemId: { type: Number, default: null },
    /** When set, load valsi sound via api (Bearer sent). Use when url is /api/jbovlaste/valsi/{id}/sound. */
    valsiIdOrWord: { type: String, default: null },
  })

  const isPlaying = ref(false)
  const isLoading = ref(false)
  const audio = ref(null)

  const CACHE_SIZE = 20
  const CACHE_KEY = 'audioCache'

  const validateBlobUrl = async (url) => {
    try {
      const response = await fetch(url)
      return response.ok
    } catch {
      return false
    }
  }

  const loadCacheFromStorage = async () => {
    if (typeof window === 'undefined') return;

    try {
      const cached = localStorage.getItem(CACHE_KEY)
      const cache = cached ? new Map(JSON.parse(cached)) : new Map()

      // Validate all blob URLs in cache
      for (const [key, value] of cache.entries()) {
        if (!(await validateBlobUrl(value.blob))) {
          cache.delete(key)
        }
      }

      return cache
    } catch {
      return new Map()
    }
  }

  const saveCacheToStorage = (cache) => {
    if (typeof window === 'undefined') return;

    try {
      const entries = Array.from(cache.entries())
      localStorage.setItem(CACHE_KEY, JSON.stringify(entries))
    } catch (e) {
      console.error('Error saving to localStorage:', e)
    }
  }

  const cleanCache = (cache) => {
    if (cache.size > CACHE_SIZE) {
      const entries = Array.from(cache.entries())
      const sortedEntries = entries.sort((a, b) => a[1].lastAccessed - b[1].lastAccessed)
      while (cache.size > CACHE_SIZE) {
        const [oldestKey] = sortedEntries.shift()
        const oldBlob = cache.get(oldestKey)?.blob
        if (oldBlob) {
          URL.revokeObjectURL(oldBlob)
        }
        cache.delete(oldestKey)
      }
      saveCacheToStorage(cache)
    }
  }

  /** True when url is our jbovlaste valsi sound endpoint (use getValsiSoundBlob). */
  const isValsiSoundUrl = () =>
    props.url.includes('/jbovlaste/valsi/') && props.url.endsWith('/sound')

  /** True when url is our collection item sound endpoint (use getItemSoundBlob). */
  const isCollectionItemSoundUrl = () =>
    props.url.includes('/collections/') && props.url.includes('/items/') && props.url.endsWith('/sound')

  const getCacheKey = () => {
    // Prefer URL-based keys so valsi fallback and collection custom sound don't share cache
    const valsiKey = props.valsiIdOrWord ?? (props.url.match(/\/jbovlaste\/valsi\/([^/]+)\/sound/)?.[1])
    if (valsiKey) return `valsi:${valsiKey}`
    if (props.collectionId != null && props.itemId != null && isCollectionItemSoundUrl()) {
      return `collection:${props.collectionId}:${props.itemId}`
    }
    return props.url
  }

  const loadAudio = async () => {
    const cacheKey = getCacheKey()
    const cache = await loadCacheFromStorage()
    const cacheEntry = cache.get(cacheKey)

    if (cacheEntry && (await validateBlobUrl(cacheEntry.blob))) {
      const blob = await fetch(cacheEntry.blob).then((r) => r.blob())
      audio.value = new Audio(URL.createObjectURL(blob))
      cache.set(cacheKey, {
        blob: cacheEntry.blob,
        lastAccessed: Date.now(),
      })
      saveCacheToStorage(cache)
      return
    }

    isLoading.value = true
    clearError()

    try {
      let blob
      // Prefer URL type: valsi fallback vs collection item custom sound (both can have collectionId/itemId set)
      if (props.valsiIdOrWord != null || isValsiSoundUrl()) {
        const idOrWord = props.valsiIdOrWord ?? (() => {
          const m = props.url.match(/\/jbovlaste\/valsi\/([^/]+)\/sound/)
          return m ? decodeURIComponent(m[1]) : null
        })()
        if (idOrWord) {
          const response = await getValsiSoundBlob(idOrWord)
          blob = response.data
        } else {
          const response = await fetch(props.url)
          if (!response.ok) throw new Error('Failed to load audio')
          blob = await response.blob()
        }
      } else if (props.collectionId != null && props.itemId != null && isCollectionItemSoundUrl()) {
        const response = await getItemSoundBlob(props.collectionId, props.itemId)
        blob = response.data
      } else {
        const response = await fetch(props.url)
        if (!response.ok) throw new Error('Failed to load audio')
        blob = await response.blob()
      }

      const blobUrl = URL.createObjectURL(blob)
      audio.value = new Audio(blobUrl)
      audio.value.addEventListener('ended', handleEnded)

      cleanCache(cache)
      cache.set(cacheKey, {
        blob: blobUrl,
        lastAccessed: Date.now(),
      })
      saveCacheToStorage(cache)
    } catch (e) {
      console.error('Error loading audio:', e)
      showError(t('audioPlayer.playError'))
    } finally {
      isLoading.value = false
    }
  }

  const togglePlay = async () => {
    if (!audio.value) {
      await loadAudio()
    }

    if (!audio.value) return

    if (isPlaying.value) {
      audio.value.pause()
      isPlaying.value = false
    } else {
      try {
        await audio.value.play()
        isPlaying.value = true
      } catch (e) {
        console.error('Error playing audio:', e)
        showError(t('audioPlayer.playError'))
      }
    }
  }

  const handleEnded = () => {
    isPlaying.value = false
  }

  onMounted(() => {
    if (audio.value) {
      audio.value.addEventListener('ended', handleEnded)
    }
  })

  onBeforeUnmount(() => {
    if (audio.value) {
      audio.value.removeEventListener('ended', handleEnded)
      URL.revokeObjectURL(audio.value.src)
    }
  })

  defineExpose({
    play: togglePlay,
  })
</script>
