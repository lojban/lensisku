<template>

  <div class="w-full max-w-2xl h-[80vh] flex flex-col">

    <div class="flex justify-between items-center mb-4">

      <h2 class="text-xl font-bold text-gray-900">
         {{ t('components.tiktoknu.likesPanel.title') }}
      </h2>
       <button
        v-if="(likedArticles ?? []).length > 0"
        class="btn-aqua-rose"
        :title="t('components.tiktoknu.likesPanel.exportButtonTitle')"
        @click="$emit('export')"
      >
         <Download class="w-4 h-4" /> {{ t('components.tiktoknu.likesPanel.exportButton') }} </button
      >
    </div>

    <div class="relative mb-4">
       <input
        type="text"
        :value="searchQuery"
        :placeholder="t('components.tiktoknu.likesPanel.searchPlaceholder')"
        class="w-full bg-gray-100 text-gray-900 px-4 py-2 pl-10 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
      /> <Search class="w-5 h-5 text-gray-400 absolute left-3 top-1/2 transform -translate-y-1/2" />

    </div>

    <div class="flex-1 overflow-y-auto min-h-0">

      <p v-if="(filteredLikedArticles ?? []).length === 0" class="text-gray-500">
         {{
          searchQuery
            ? t('components.tiktoknu.likesPanel.noMatches')
            : t('components.tiktoknu.likesPanel.noLikes')
        }}
      </p>

      <div v-else class="space-y-4">

        <div
          v-for="article in filteredLikedArticles"
          :key="article.pageid"
          class="flex gap-4 items-start group"
        >
           <img
            v-if="article.thumbnail"
            :src="article.thumbnail.source"
            :alt="article.title"
            class="w-20 h-20 object-cover rounded"
          />
          <div class="flex-1">

            <div class="flex justify-between items-start">
               <a
                :href="article.url"
                target="_blank"
                rel="noopener noreferrer"
                class="font-bold text-gray-900 hover:text-blue-600"
                > {{ article.title }} </a
              > <button
                class="text-gray-500 hover:text-red-600 p-1 rounded-full md:opacity-0 md:group-hover:opacity-100 transition-opacity"
                :aria-label="t('components.tiktoknu.likesPanel.removeFromLikes')"
                @click.stop="$emit('remove', article)"
              >
                 <X class="w-4 h-4" /> </button
              >
            </div>

            <p class="text-sm text-gray-600 line-clamp-2"> {{ article.extract }} </p>

          </div>

        </div>

      </div>

    </div>

  </div>

</template>

<script setup lang="ts">
import { Download, Search, X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

import type { WikiArticle } from '../../types/wiki'

const { t } = useI18n()

defineProps<{
  likedArticles: WikiArticle[]
  filteredLikedArticles: WikiArticle[]
  searchQuery: string
}>()

defineEmits(['update:searchQuery', 'export', 'remove'])
</script>

