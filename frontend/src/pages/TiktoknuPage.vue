<template>
  <div
    ref="scrollContainer"
    class="reel-container -m-4 h-full bg-black text-white overflow-y-scroll snap-y snap-mandatory touch-pan-y"
  >
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
  import { ref, onMounted, computed } from 'vue'
  import { useI18n } from 'vue-i18n';
  
  import LanguageSelector from '../components/LanguageSelector.vue'
  import ModalComponent from '../components/ModalComponent.vue'
  import LikesPanel from '../components/tiktoknu/LikesPanel.vue'
  import WikiCard from '../components/tiktoknu/WikiCard.vue'
  import { useLikedArticles } from '../composables/tiktoknu/useLikedArticles'
  import { useLocalization } from '../composables/tiktoknu/useLocalization'
  import { useWikiArticles } from '../composables/tiktoknu/useWikiArticles'


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
  const scrollContainer = ref(null)
  const showLikes = ref(false)
  const searchQuery = ref('')

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

  // Initial load
  onMounted(fetchArticles)
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
