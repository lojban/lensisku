<template>
  <div
    v-if="loading"
    class="mb-6 rounded-lg border border-gray-200 bg-white p-4 sm:p-6"
  >
    <div class="flex flex-col gap-4">
        <div class="flex flex-row items-stretch gap-3 sm:gap-4">
          <div
            class="w-16 shrink-0 self-stretch animate-pulse rounded-lg bg-gray-200 sm:rounded-xl md:w-24 lg:w-28"
            aria-hidden="true"
          />
          <div class="flex min-w-0 flex-1 flex-col gap-2 pt-0.5">
            <div
              class="h-4 w-32 animate-pulse rounded bg-gray-200"
              aria-hidden="true"
            />
            <div
              class="h-8 w-72 max-w-full animate-pulse rounded bg-gray-200"
              aria-hidden="true"
            />
            <div
              class="h-12 w-full max-w-md animate-pulse rounded bg-gray-200"
              aria-hidden="true"
            />
          </div>
        </div>
        <div class="flex flex-wrap gap-2">
          <div
            class="h-6 w-16 animate-pulse rounded bg-gray-200"
            aria-hidden="true"
          />
          <div
            class="h-6 w-24 animate-pulse rounded bg-gray-200"
            aria-hidden="true"
          />
        </div>
        <slot name="toolbar-skeleton">
          <div class="flex flex-wrap justify-center gap-2 md:justify-start">
            <div
              class="h-10 w-24 animate-pulse rounded bg-gray-200"
              aria-hidden="true"
            />
            <div
              class="h-10 w-28 animate-pulse rounded bg-gray-200"
              aria-hidden="true"
            />
            <div
              class="h-10 w-20 animate-pulse rounded bg-gray-200"
              aria-hidden="true"
            />
          </div>
        </slot>
    </div>
  </div>
  <PageHeader
    v-else-if="collection"
    title-as="h2"
    title-tone="secondary"
    stack-gap="comfortable"
  >
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
        <div
          v-else
          class="collection-header-logo-placeholder self-stretch"
          aria-hidden="true"
        >
          <BookOpen class="h-5 w-5 sm:h-7 sm:w-7 md:h-9 md:w-9" />
        </div>
      </template>
      <template #eyebrow>
        <slot name="hint" />
      </template>
      <template #title>
        <slot name="title" />
      </template>
      <template v-if="collection.description" #description>
        <div class="max-h-32 min-w-0 overflow-y-auto text-sm">
          <LazyMathJax :content="collection.description" />
        </div>
      </template>
      <template #meta>
        <span
          class="shrink-0 select-none rounded-full px-2 py-1 text-sm"
          :class="
            collection.is_public
              ? 'bg-green-100 text-green-700'
              : 'bg-gray-100 text-gray-700'
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
      </template>
      <template #meta-trailing>
        <slot name="meta-actions" />
      </template>
      <template #toolbar>
        <slot name="toolbar" />
      </template>
  </PageHeader>
</template>

<script setup lang="ts">
import { BookOpen } from 'lucide-vue-next'
import LazyMathJax from '@/components/LazyMathJax.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
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
