<template>
  <div v-if="loading" class="page-header-shell mb-6" aria-busy="true" aria-hidden="true">
    <div class="flex flex-col gap-4">
      <div class="flex flex-row items-stretch gap-3 sm:gap-4">
        <div
          class="collection-header-logo-placeholder self-stretch min-h-[4.5rem] skeleton-pulse"
        />

        <div class="flex min-w-0 flex-1 flex-col gap-2 pt-0.5">
          <div class="flex items-center gap-2">
            <div class="skeleton-bone h-5 w-5 shrink-0" />
            <div class="skeleton-bone--soft h-4 w-40 max-w-[70%]" />
          </div>
          <div class="skeleton-bone h-7 w-64 max-w-full sm:h-8" />
          <div class="skeleton-bone--soft h-16 w-full max-w-md" />
        </div>
      </div>

      <div class="flex flex-wrap gap-2">
        <div class="skeleton-bone h-6 w-16 rounded-full" />
        <div class="skeleton-bone--soft h-6 w-28 rounded-full" />
        <div class="skeleton-bone--soft h-6 w-20 rounded-full" />
      </div>
      <slot name="toolbar-skeleton">
        <div class="flex flex-wrap justify-center gap-2 md:justify-start">
          <div class="skeleton-bone h-6 w-24 rounded-full" />
          <div class="skeleton-bone h-6 w-28 rounded-full" />
          <div class="skeleton-bone h-6 w-20 rounded-full" />
        </div>
      </slot>
    </div>
  </div>
  <PageHeader v-else-if="collection" title-as="h2" title-tone="secondary" stack-gap="comfortable">
    <template #leading>
      <CollectionCoverLightbox
        v-if="coverImageUrl"
        class="shrink-0 self-stretch"
        :image-url="coverImageUrl"
        :alt="coverImageAlt"
        :aria-label="coverLightboxDialogLabel"
        :close-aria-label="coverLightboxCloseLabel"
      >
        <div class="collection-header-logo">
          <img
            :src="coverImageUrl"
            :alt="coverImageAlt"
            class="collection-header-cover-thumb"
            loading="lazy"
            decoding="async"
          />
        </div>
      </CollectionCoverLightbox>
    </template>
    <template #eyebrow> <slot name="hint" /> </template>
    <template #title> <slot name="title" /> </template>
    <template v-if="collection.description" #description>
      <div class="max-h-32 min-w-0 overflow-y-auto text-sm">
        <LazyMathJax :content="collection.description" />
      </div>
    </template>
    <template #meta>
      <span
        class="inline-flex h-6 shrink-0 items-center select-none rounded-full px-2 py-1 text-xs"
        :class="collection.is_public ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-700'"
      >
        {{ collection.is_public ? publicLabel : privateLabel }}
      </span>
      <span v-if="collection.owner" class="inline-flex h-6 items-center text-xs text-gray-500">
        {{ createdByLabel }}
        <RouterLink
          :to="`/user/${collection.owner.username}`"
          class="ml-1 text-blue-600 hover:text-blue-800 hover:underline"
        >
          {{ collection.owner.username }}
        </RouterLink>
      </span>
      <span class="inline-flex h-6 items-center text-xs text-gray-500">{{ itemsCountLabel }}</span>
    </template>
    <template #meta-trailing> <slot name="meta-actions" /> </template>
    <template #toolbar> <slot name="toolbar" /> </template>
  </PageHeader>
</template>

<script setup lang="ts">
import LazyMathJax from '@/components/LazyMathJax.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import { CollectionCoverLightbox } from '@packages/ui'

/** Minimal collection shape for header + meta row. */
interface CollectionPageHeaderModel {
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
