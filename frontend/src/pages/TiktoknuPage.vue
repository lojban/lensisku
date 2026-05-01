<template>
  <div
    ref="scrollContainer"
    class="reel-container h-full bg-black text-white overflow-y-scroll snap-y snap-mandatory touch-pan-y"
  >
    <template v-if="isCollectionMode">
      <div class="absolute top-4 left-4 z-30">
        <RouterLink
          :to="`/collections/${collectionIdStr}`"
          class="text-sm text-white/80 hover:text-white transition-colors"
        >
          {{ t('components.tiktoknu.collectionReel.backToCollection') }}
        </RouterLink>
      </div>
      <div class="absolute top-4 right-4 z-30">
        <LanguageSelector />
      </div>
      <CollectionReelCard
        v-for="item in reelItems"
        :key="item.item_id"
        :collection-id="collectionIdStr"
        :item="item"
      />
      <div
        v-if="reelLoading && reelItems.length === 0"
        class="h-dvh max-h-dvh w-full flex shrink-0 items-center justify-center gap-2"
      >
        <Loader2 class="h-6 w-6 animate-spin" />
        <span>{{ t('components.tiktoknu.loading') }}</span>
      </div>
      <div
        v-else-if="!reelLoading && reelItems.length === 0 && !reelHasMore"
        class="h-dvh max-h-dvh w-full flex shrink-0 items-center justify-center px-6 text-center text-white/80"
      >
        {{ t('components.tiktoknu.collectionReel.empty') }}
      </div>
      <div
        v-if="reelLoading && reelItems.length > 0"
        class="h-24 w-full flex items-center justify-center gap-2 text-white/70"
      >
        <Loader2 class="h-5 w-5 animate-spin" />
        <span>{{ t('components.tiktoknu.loading') }}</span>
      </div>
    </template>

    <template v-else>
      <div class="absolute top-4 right-4 z-30 flex flex-col items-end gap-2">
        <button
          class="text-sm text-white/70 hover:text-white transition-colors"
          @click="showLikes = true"
        >
          {{ t('components.tiktoknu.likesCount', { count: likedArticlesCount }) }}
        </button>
        <LanguageSelector />
      </div>
      <WikiCard
        v-for="article in articles"
        :key="article.pageid"
        :article="article"
        :is-liked="isLiked(article.pageid)"
        @like="toggleLike(article)"
      />
      <div v-if="loading" class="h-screen w-full flex items-center justify-center gap-2">
        <Loader2 class="h-6 w-6 animate-spin" />
        <span>{{ t('components.tiktoknu.loading') }}</span>
      </div>
      <ModalComponent :show="showLikes" class="mt-16" @close="showLikes = false">
        <LikesPanel
          :liked-articles="likedArticles"
          :filtered-liked-articles="filteredLikedArticles"
          :search-query="searchQuery"
          @update:search-query="(val) => (searchQuery = val)"
          @export="handleExport"
          @remove="toggleLike"
        />
      </ModalComponent>
    </template>

    <div ref="observerTarget" class="h-1 w-full mb-[10rem]" aria-hidden="true" />
  </div>
</template>

<script setup lang="ts">
import { useIntersectionObserver } from '@vueuse/core'
import { Loader2 } from 'lucide-vue-next'
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'

import LanguageSelector from '../components/LanguageSelector.vue'
import ModalComponent from '../components/ModalComponent.vue'
import CollectionReelCard from '../components/tiktoknu/CollectionReelCard.vue'
import LikesPanel from '../components/tiktoknu/LikesPanel.vue'
import WikiCard from '../components/tiktoknu/WikiCard.vue'
import { useSeoHead } from '@/composables/useSeoHead'
import { getCollection, listCollectionItems } from '@/api'
import { useLikedArticles } from '../composables/tiktoknu/useLikedArticles'
import { useLocalization } from '../composables/tiktoknu/useLocalization'
import { useWikiArticles } from '../composables/tiktoknu/useWikiArticles'

const props = defineProps<{
  collectionId?: string
}>()

const { t } = useI18n()

const isCollectionMode = computed(() => Boolean(props.collectionId && String(props.collectionId).trim()))
const collectionIdStr = computed(() => (props.collectionId ? String(props.collectionId) : ''))

const collectionTitle = ref('')
const seoTitle = computed(() => {
  if (isCollectionMode.value && collectionTitle.value) {
    return `${collectionTitle.value} — ${t('components.tiktoknu.pageTitle')}`
  }
  return t('components.tiktoknu.pageTitle')
})
useSeoHead({ title: seoTitle, robots: 'noindex, nofollow' })

const PER_PAGE = 30
const reelItems = ref<
  Array<{
    item_id: number
    has_front_image: boolean
    has_back_image: boolean
    word?: string | null
    free_content_front?: string | null
    definition?: string | null
    free_content_back?: string | null
    valsi_id?: number | null
  }>
>([])
const reelLoading = ref(false)
const reelHasMore = ref(true)
const reelNextPage = ref(1)
const reelTotal = ref(0)

async function fetchMoreReels() {
  if (!isCollectionMode.value || reelLoading.value || !reelHasMore.value) return
  reelLoading.value = true
  try {
    const res = await listCollectionItems(collectionIdStr.value, {
      page: reelNextPage.value,
      per_page: PER_PAGE,
      has_card_image_only: true,
    })
    const total = Number(res.data.total) ?? 0
    const batch = res.data.items ?? []
    reelTotal.value = total
    reelItems.value.push(...batch)
    reelNextPage.value += 1
    if (batch.length === 0 || reelItems.value.length >= total) {
      reelHasMore.value = false
    }
  } catch (e) {
    console.error(e)
    reelHasMore.value = false
  } finally {
    reelLoading.value = false
  }
}

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

const handleExport = () => {
  const dataStr = JSON.stringify(likedArticles.value, null, 2)
  const dataUri = 'data:application/json;charset=utf-8,' + encodeURIComponent(dataStr)
  const fileName = `tiktoknu-favorites-${new Date().toISOString().split('T')[0]}.json`
  const link = document.createElement('a')
  link.href = dataUri
  link.download = fileName
  link.click()
}

useIntersectionObserver(
  observerTarget,
  ([{ isIntersecting }]) => {
    if (!isIntersecting) return
    if (isCollectionMode.value) {
      if (!reelLoading.value && reelHasMore.value) fetchMoreReels()
    } else if (!loading.value) {
      fetchArticles()
    }
  },
  {
    root: scrollContainer,
    rootMargin: '800px',
    threshold: 0,
  }
)

onMounted(() => {
  if (isCollectionMode.value) {
    getCollection(collectionIdStr.value)
      .then((r) => {
        collectionTitle.value = r.data.name ?? ''
      })
      .catch(() => {
        collectionTitle.value = ''
      })
    fetchMoreReels()
  } else {
    fetchArticles()
  }
})
</script>

<style scoped>
.reel-container {
  overscroll-behavior-y: contain;
  -webkit-overflow-scrolling: touch;
}

.reel-container::-webkit-scrollbar {
  display: none;
}
</style>
