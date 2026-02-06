<template>
  <div class="wiki-card h-screen w-full flex items-center justify-center snap-start snap-always relative">
    <div class="h-full w-full relative">
      <div v-if="article.thumbnail" class="absolute inset-0">
        <img loading="lazy" :src="article.thumbnail.source" :alt="article.title" :class="[
          'w-full h-full object-cover transition-opacity duration-300 bg-white',
          { 'opacity-100': imageLoaded, 'opacity-0': !imageLoaded },
        ]" @load="imageLoaded = true" @error="handleImageError">
        <div v-if="!imageLoaded" class="absolute inset-0 bg-gray-900 animate-pulse" />
        <div class="absolute inset-0 bg-gradient-to-b from-black/40 to-black/80" />
      </div>
      <div v-else class="absolute inset-0 bg-gray-900" />

      <div class="absolute bottom-[10vh] left-0 right-0 p-6 text-white z-10">
        <div class="flex justify-between items-start mb-3">
          <a :href="article.url" target="_blank" rel="noopener noreferrer"
            class="hover:text-gray-200 transition-colors">
            <h2 class="text-2xl font-bold drop-shadow-lg">{{ article.title }}</h2>
          </a>
          <div class="flex gap-2">
            <button :class="[
              'p-2 rounded-full backdrop-blur-sm transition-colors',
              isLiked ? 'bg-red-500 hover:bg-red-600' : 'bg-white/10 hover:bg-white/20',
            ]" :aria-label="t('components.tiktoknu.wikiCard.likeArticle')" @click="$emit('like', article)">
              <Heart :class="['w-5 h-5', { 'fill-white': isLiked }]" />
            </button>
            <button class="p-2 rounded-full bg-white/10 backdrop-blur-sm hover:bg-white/20 transition-colors"
              :aria-label="t('components.tiktoknu.wikiCard.shareArticle')" @click="handleShare">
              <Share2 class="w-5 h-5" />
            </button>
          </div>
        </div>
        <p class="text-gray-100 mb-4 drop-shadow-lg line-clamp-6">
          {{ article.extract }}
        </p>
        <a :href="article.url" target="_blank" rel="noopener noreferrer"
          class="inline-block text-white hover:text-gray-200 drop-shadow-lg">
          {{ t('components.tiktoknu.wikiCard.readMore') }}
        </a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Heart, Share2 } from 'lucide-vue-next';
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';

import type { WikiArticle } from '../../types/wiki';

const { t } = useI18n();

const props = defineProps<{
  article: WikiArticle
  isLiked: boolean
}>()

defineEmits(['like'])

const imageLoaded = ref(false)

const handleImageError = () => {
  console.error('Image failed to load')
  imageLoaded.value = true
}

const handleShare = async () => {
  if (navigator.share) {
    try {
      await navigator.share({
        title: props.article.title,
        text: props.article.extract || '',
        url: props.article.url,
      })
    } catch (error) {
      console.error(t('components.tiktoknu.wikiCard.shareError'), error)
    }
  } else {
    if (props.article.url) {
      await navigator.clipboard.writeText(props.article.url)
      alert(t('components.tiktoknu.wikiCard.copySuccess'))
    }
  }
}
</script>
