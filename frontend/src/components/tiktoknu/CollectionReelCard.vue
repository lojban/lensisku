<template>
  <div
    class="collection-reel-card h-screen w-full flex items-center justify-center snap-start snap-always relative"
  >
    <div class="h-full w-full relative">
      <div class="absolute inset-0">
        <img
          loading="lazy"
          :src="imageSrc"
          :alt="heading"
          :class="[
            'w-full h-full object-cover transition-opacity duration-300 bg-gray-900',
            { 'opacity-100': imageLoaded, 'opacity-0': !imageLoaded },
          ]"
          @load="imageLoaded = true"
          @error="handleImageError"
        />
        <div v-if="!imageLoaded" class="absolute inset-0 bg-gray-900 animate-pulse" />
        <div class="absolute inset-0 bg-gradient-to-b from-black/40 to-black/80" />
      </div>

      <div class="absolute bottom-[10vh] left-0 right-0 p-6 text-white z-10">
        <div class="flex justify-between items-start gap-3 mb-3">
          <h2 class="text-2xl font-bold drop-shadow-lg min-w-0">
            {{ heading }}
          </h2>
          <RouterLink
            v-if="entryHref"
            :to="entryHref"
            class="shrink-0 text-sm px-3 py-1.5 rounded-full bg-white/15 backdrop-blur-sm hover:bg-white/25 transition-colors"
          >
            {{ t('components.tiktoknu.collectionReel.openEntry') }}
          </RouterLink>
        </div>

        <div class="space-y-3 text-gray-100 drop-shadow-lg">
          <div>
            <p class="text-xs uppercase tracking-wide text-white/60 mb-1">
              {{ t('components.tiktoknu.collectionReel.frontLabel') }}
            </p>
            <p class="text-sm sm:text-base line-clamp-4">{{ frontPlain }}</p>
          </div>
          <div>
            <p class="text-xs uppercase tracking-wide text-white/60 mb-1">
              {{ t('components.tiktoknu.collectionReel.backLabel') }}
            </p>
            <p class="text-sm sm:text-base line-clamp-6">{{ backPlain }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

export type CollectionReelItem = {
  item_id: number
  has_front_image: boolean
  has_back_image: boolean
  word?: string | null
  free_content_front?: string | null
  definition?: string | null
  free_content_back?: string | null
  valsi_id?: number | null
}

const props = defineProps<{
  collectionId: string | number
  item: CollectionReelItem
}>()

const { t } = useI18n()

const imageLoaded = ref(false)

const imageSide = computed(() => (props.item.has_front_image ? 'front' : 'back'))

const imageSrc = computed(
  () =>
    `/api/collections/${props.collectionId}/items/${props.item.item_id}/image/${imageSide.value}`
)

const heading = computed(() => {
  const w = props.item.word?.trim()
  if (w) return w
  return stripHtml(props.item.free_content_front) || '—'
})

const frontPlain = computed(() =>
  stripHtml(props.item.word ?? props.item.free_content_front) || '—'
)

const backPlain = computed(() =>
  stripHtml(props.item.definition ?? props.item.free_content_back) || '—'
)

const entryHref = computed(() => {
  const vid = props.item.valsi_id
  if (vid == null || vid === 0) return null
  return `/valsi/${vid}`
})

function stripHtml(s: string | null | undefined): string {
  if (!s) return ''
  return s
    .replace(/<[^>]*>/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
}

const handleImageError = () => {
  console.error('Collection reel image failed to load')
  imageLoaded.value = true
}
</script>
