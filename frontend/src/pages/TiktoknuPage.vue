<template>
  <div
    ref="scrollContainer"
    class="reel-container -m-4 h-screen bg-black text-white overflow-y-scroll snap-y snap-mandatory touch-pan-y"
  >
    <!-- Header -->
    <div class="z-30">
      <button
        class="text-2xl font-bold text-white drop-shadow-lg hover:opacity-80 transition-opacity"
        @click="resetPage"
      >
        tiktoknu
      </button>
    </div>

    <!-- Controls -->
    <div class="absolute top-4 right-4 z-30 flex flex-col items-end gap-2">
      <button
        class="text-sm text-white/70 hover:text-white transition-colors"
        @click="showLikes = true"
      >
        {{ t('components.tiktoknu.likesCount', { count: likedArticlesCount }) }}
      </button>
      <LanguageSelector />
    </div>

    <!-- Articles -->
    <WikiCard
      v-for="article in articles"
      :key="article.pageid"
      :article="article"
      :is-liked="isLiked(article.pageid)"
      @like="toggleLike(article)"
    />

    <div
      v-if="loading"
      class="h-screen w-full flex items-center justify-center gap-2"
    >
      <Loader2 class="h-6 w-6 animate-spin" />
      <span>{{ t('components.tiktoknu.loading') }}</span>
    </div>

    <ModalComponent
      :show="showLikes"
      class="mt-16"
      @close="showLikes = false"
    >
      <LikesPanel
        :liked-articles="likedArticles"
        :filtered-liked-articles="filteredLikedArticles"
        :search-query="searchQuery"
        @update:search-query="(val) => (searchQuery = val)"
        @export="handleExport"
        @remove="toggleLike"
      />
    </ModalComponent>
    <!-- Observer target at very bottom -->
    <div
      ref="observerTarget"
      class="h-1 w-full mb-[10rem]"
      aria-hidden="true"
    />
  </div>
</template>

<script setup lang="ts">
  import { useIntersectionObserver } from '@vueuse/core'
  import { Loader2 } from 'lucide-vue-next'
  import { ref, onMounted, computed, onUnmounted } from 'vue'
  import { useI18n } from 'vue-i18n';
  
  import LanguageSelector from '../components/LanguageSelector.vue'
  import ModalComponent from '../components/ModalComponent.vue'
  import LikesPanel from '../components/tiktoknu/LikesPanel.vue'
  import WikiCard from '../components/tiktoknu/WikiCard.vue'
  import { useLikedArticles } from '../composables/tiktoknu/useLikedArticles'
  import { useLocalization } from '../composables/tiktoknu/useLocalization'
  import { useWikiArticles } from '../composables/tiktoknu/useWikiArticles'

  const REEL_DEBOUNCE_MS = 2000

  const { t } = useI18n()
  const { currentLanguage } = useLocalization()
  const { articles, loading, fetchArticles } = useWikiArticles(currentLanguage)
  const { likedArticles, toggleLike, isLiked, likedArticlesCount } = useLikedArticles()
  const filteredLikedArticles = computed(() =>
    likedArticles.value.filter(
      (article) =>
        article.title.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
        article.extract.toLowerCase().includes(searchQuery.value.toLowerCase())
    )
  )
  const observerTarget = ref<HTMLElement | null>(null)
  const scrollContainer = ref<HTMLElement | null>(null)
  const showLikes = ref(false)
  const searchQuery = ref('')

  let lockedReelIndex = 0
  let lastReelChangeTime = 0

  function getReelIndexFromScrollTop(scrollTop: number, viewportHeight: number): number {
    return Math.round(scrollTop / viewportHeight)
  }

  function scrollToReel(index: number) {
    const el = scrollContainer.value
    if (!el) return
    const vh = el.clientHeight
    const top = Math.max(0, index * vh)
    el.scrollTo({ top, behavior: 'smooth' })
  }

  function handleReelScrollEnd() {
    const el = scrollContainer.value
    if (!el) return
    const vh = el.clientHeight
    const currentReel = getReelIndexFromScrollTop(el.scrollTop, vh)
    const now = Date.now()
    const withinCooldown = now - lastReelChangeTime < REEL_DEBOUNCE_MS

    if (currentReel !== lockedReelIndex) {
      if (withinCooldown) {
        scrollToReel(lockedReelIndex)
      } else {
        const maxReel = Math.max(0, Math.floor((el.scrollHeight - vh) / vh))
        const oneStep = lockedReelIndex + Math.sign(currentReel - lockedReelIndex)
        const allowedReel = Math.max(0, Math.min(oneStep, maxReel))
        if (allowedReel !== currentReel) {
          scrollToReel(allowedReel)
        }
        lockedReelIndex = allowedReel
        lastReelChangeTime = now
      }
    }
  }

  const resetPage = () => window.location.reload()

  const handleExport = () => {
    const dataStr = JSON.stringify(likedArticles.value, null, 2)
    const dataUri = 'data:application/json;charset=utf-8,' + encodeURIComponent(dataStr)
    const fileName = `tiktoknu-favorites-${new Date().toISOString().split('T')[0]}.json`

    const link = document.createElement('a')
    link.href = dataUri
    link.download = fileName
    link.click()
  }

  // Infinite scroll observer with proper configuration
  useIntersectionObserver(
    observerTarget,
    ([{ isIntersecting }]) => {
      if (isIntersecting && !loading.value) {
        fetchArticles()
      }
    },
    {
      root: scrollContainer,
      rootMargin: '800px',
      threshold: 0,
    }
  )

  let scrollEndTimeout: ReturnType<typeof setTimeout> | null = null

  function onScroll() {
    if (scrollEndTimeout) clearTimeout(scrollEndTimeout)
    scrollEndTimeout = setTimeout(() => {
      scrollEndTimeout = null
      handleReelScrollEnd()
    }, 150)
  }

  onMounted(() => {
    fetchArticles()
    const el = scrollContainer.value
    if (el) {
      el.addEventListener('scroll', onScroll, { passive: true })
    }
  })

  onUnmounted(() => {
    const el = scrollContainer.value
    if (el) {
      el.removeEventListener('scroll', onScroll)
    }
    if (scrollEndTimeout) clearTimeout(scrollEndTimeout)
  })
</script>

<style scoped>
  .reel-container {
    overscroll-behavior-y: contain;
    -webkit-overflow-scrolling: touch;
  }
</style>

<style>
  html,
  body {
    overscroll-behavior-y: contain;
    overflow: hidden;
  }

  ::-webkit-scrollbar {
    display: none;
  }
</style>
