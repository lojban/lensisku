<template>
  <div class="inline-flex items-center gap-2">
    <!-- Lingo listen-tap: chunky speaker (Duo challenge-listenTap). -->
    <button
      v-if="variant === 'lingo-speaker' || variant === 'lingo-speaker-slow'"
      type="button"
      class="lingo-study-listen-speaker"
      :class="{
        'lingo-study-listen-speaker--slow': variant === 'lingo-speaker-slow',
        'lingo-study-listen-speaker--playing': isPlaying,
      }"
      :disabled="isLoading"
      :aria-label="
        variant === 'lingo-speaker-slow'
          ? t('flashcardStudy.playSlow')
          : t('flashcardStudy.playAudio')
      "
      @click="togglePlay"
    >
      <Loader v-if="isLoading" class="animate-spin" />
      <Pause v-else-if="isPlaying" />
      <template v-else-if="variant === 'lingo-speaker-slow'">
        <Volume2 class="lingo-study-listen-speaker__icon" />
        <span class="lingo-study-listen-speaker__rate" aria-hidden="true">½</span>
      </template>
      <Volume2 v-else class="lingo-study-listen-speaker__icon" />
    </button>
    <Button
      v-else
      variant="read"
      class="audio-player-btn"
      :disabled="isLoading"
      @click="togglePlay"
    >
      <Pause v-if="isPlaying" />
      <Loader v-else-if="isLoading" class="animate-spin" />
      <Play v-else />
    </Button>
  </div>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { Play, Pause, Loader, Volume2 } from '@lucide/vue'
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { useError } from '@/composables/useError'
import { getItemSoundBlob, getValsiSoundBlob } from '@/api'

const { showError, clearError } = useError()
const { t } = useI18n()
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
  /** When true, log load/play errors to console only (no user-facing showError). Use for auto-play contexts. */
  suppressPlayErrors: { type: Boolean, default: false },
  /** `default` | `lingo-speaker` | `lingo-speaker-slow` (0.5×). */
  variant: { type: String, default: 'default' },
  /** Playback rate (slow speaker uses 0.5 when variant is lingo-speaker-slow). */
  playbackRate: { type: Number, default: 1 },
})

const isPlaying = ref(false)
const isLoading = ref(false)
const audio = ref(null)

const CACHE_SIZE = 20
const CACHE_KEY = 'audioCache'

type AudioCacheEntry = { blob: string; lastAccessed: number }
type AudioCache = Map<string, AudioCacheEntry>

const validateBlobUrl = async (url: string) => {
  try {
    const response = await fetch(url)
    return response.ok
  } catch {
    return false
  }
}

const loadCacheFromStorage = async (): Promise<AudioCache> => {
  if (typeof window === 'undefined') return new Map()

  try {
    const cached = localStorage.getItem(CACHE_KEY)
    const cache: AudioCache = cached
      ? new Map(JSON.parse(cached) as [string, AudioCacheEntry][])
      : new Map()

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

const saveCacheToStorage = (cache: AudioCache) => {
  if (typeof window === 'undefined') return

  try {
    const entries = Array.from(cache.entries())
    localStorage.setItem(CACHE_KEY, JSON.stringify(entries))
  } catch (e) {
    console.error('Error saving to localStorage:', e)
  }
}

const cleanCache = (cache: AudioCache) => {
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
  props.url.includes('/collections/') &&
  props.url.includes('/items/') &&
  props.url.endsWith('/sound')

const getCacheKey = () => {
  // Prefer URL-based keys so valsi fallback and collection custom sound don't share cache
  const valsiKey = props.valsiIdOrWord ?? props.url.match(/\/jbovlaste\/valsi\/([^/]+)\/sound/)?.[1]
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
    applyPlaybackRate()
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
      const idOrWord =
        props.valsiIdOrWord ??
        (() => {
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
    applyPlaybackRate()
    audio.value.addEventListener('ended', handleEnded)

    cleanCache(cache)
    cache.set(cacheKey, {
      blob: blobUrl,
      lastAccessed: Date.now(),
    })
    saveCacheToStorage(cache)
  } catch (e) {
    console.error('Error loading audio:', e)
    if (!props.suppressPlayErrors) showError(t('audioPlayer.playError'))
  } finally {
    isLoading.value = false
  }
}

function effectivePlaybackRate() {
  if (props.variant === 'lingo-speaker-slow') return 0.5
  const rate = Number(props.playbackRate)
  return Number.isFinite(rate) && rate > 0 ? rate : 1
}

function applyPlaybackRate() {
  if (audio.value) audio.value.playbackRate = effectivePlaybackRate()
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
      applyPlaybackRate()
      audio.value.currentTime = 0
      await audio.value.play()
      isPlaying.value = true
    } catch (e) {
      console.error('Error playing audio:', e)
      if (!props.suppressPlayErrors) showError(t('audioPlayer.playError'))
    }
  }
}

const handleEnded = () => {
  isPlaying.value = false
}

watch(
  () => [props.playbackRate, props.variant],
  () => applyPlaybackRate()
)

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
