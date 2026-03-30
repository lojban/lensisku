<template>
  <div class="bg-white border rounded-lg p-4 sm:p-6 mb-6 flex flex-col gap-4">
    <template v-if="loading">
      <div class="flex flex-row items-stretch gap-3 sm:gap-4">
        <div
          class="w-16 md:w-24 lg:w-28 self-stretch min-h-[4.5rem] bg-gray-200 animate-pulse rounded-lg sm:rounded-xl shrink-0"
          aria-hidden="true"
        />
        <div class="min-w-0 flex-1 flex flex-col gap-2 pt-0.5">
          <div class="h-4 w-32 bg-gray-200 animate-pulse rounded" aria-hidden="true" />
          <div class="h-8 w-72 max-w-full bg-gray-200 animate-pulse rounded" aria-hidden="true" />
          <div class="h-12 w-full max-w-md bg-gray-200 animate-pulse rounded" aria-hidden="true" />
        </div>
      </div>
      <div class="flex flex-wrap gap-2">
        <div class="h-6 w-16 bg-gray-200 animate-pulse rounded" aria-hidden="true" />
        <div class="h-6 w-24 bg-gray-200 animate-pulse rounded" aria-hidden="true" />
      </div>
      <slot name="toolbar-skeleton">
        <div class="flex flex-wrap justify-center md:justify-start gap-2">
          <div class="h-10 w-24 bg-gray-200 animate-pulse rounded" aria-hidden="true" />
          <div class="h-10 w-28 bg-gray-200 animate-pulse rounded" aria-hidden="true" />
          <div class="h-10 w-20 bg-gray-200 animate-pulse rounded" aria-hidden="true" />
        </div>
      </slot>
    </template>
    <template v-else-if="collection">
      <div class="flex flex-row items-stretch gap-3 sm:gap-4">
        <CollectionCoverLightbox
          v-if="coverImageUrl"
          :image-url="coverImageUrl"
          :alt="coverImageAlt"
          :aria-label="coverLightboxDialogLabel"
          :close-aria-label="coverLightboxCloseLabel"
        >
          <div class="collection-header-logo">
            <img
              :src="coverImageUrl"
              :alt="coverImageAlt"
              class="h-full w-full object-cover"
              loading="lazy"
              decoding="async"
            />
          </div>
        </CollectionCoverLightbox>
        <div v-else class="collection-header-logo-placeholder" aria-hidden="true">
          <BookOpen class="h-5 w-5 sm:h-7 sm:w-7 md:h-9 md:w-9" />
        </div>
        <div class="min-w-0 flex-1 flex flex-col gap-2 pt-0.5">
          <slot name="hint" />
          <slot name="title" />
          <div v-if="collection.description" class="min-w-0 max-h-32 text-sm overflow-y-auto">
            <LazyMathJax :content="collection.description" />
          </div>
        </div>
      </div>
      <div class="flex flex-row gap-2 items-center justify-between text-sm text-gray-500">
        <div class="flex flex-wrap items-center gap-2">
          <span
            class="text-sm px-2 py-1 rounded-full select-none shrink-0"
            :class="
              collection.is_public ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-700'
            "
          >
            {{ collection.is_public ? publicLabel : privateLabel }}
          </span>
          <span v-if="collection.owner">
            {{ createdByLabel }}
            <RouterLink
              :to="`/user/${collection.owner.username}`"
              class="text-blue-600 hover:text-blue-800 hover:underline"
            >
              {{ collection.owner.username }}
            </RouterLink>
          </span>
          <span>{{ itemsCountLabel }}</span>
        </div>
        <slot name="meta-actions" />
      </div>
      <slot name="toolbar" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { BookOpen } from 'lucide-vue-next'
import LazyMathJax from '@/components/LazyMathJax.vue'
import { CollectionCoverLightbox } from '@packages/ui'

/** Minimal collection shape for header + meta row. */
export interface CollectionPageHeaderModel {
  name: string
  description?: string | null
  is_public: boolean
  owner?: { username: string }
  item_count: number
}

defineProps<{
  /** When true, shows layout skeleton (e.g. collection not loaded yet). */
  loading?: boolean
  collection: CollectionPageHeaderModel | null
  coverImageUrl: string | null
  coverImageAlt: string
  coverLightboxDialogLabel: string
  coverLightboxCloseLabel: string
  /** Meta line: "Public" / "Private" badge text — defaults use i18n-friendly prop pass-through from parent. */
  publicLabel: string
  privateLabel: string
  createdByLabel: string
  /** Already-localized items count line, e.g. "989 items". */
  itemsCountLabel: string
}>()
</script>
