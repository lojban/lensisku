<template>
  <!--
    h-dvh: matches visible viewport on mobile (100vh is often taller than the screen due to browser chrome).
    Text lives in a max-height panel with overflow so long definitions never push past the viewport.
  -->
  <div
    class="collection-reel-card h-dvh max-h-dvh w-full snap-start snap-always relative overflow-hidden min-h-0"
  >
    <!-- Image background -->
    <div v-if="hasImage" class="absolute inset-0 min-h-0">
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

      <div
        class="absolute inset-0 bg-gradient-to-b from-black/40 via-black/20 to-black/85 pointer-events-none"
      />
    </div>

    <!-- Text-only fallback: atmospheric gradient + large word hero -->
    <div
      v-else
      class="absolute inset-0 min-h-0 bg-gradient-to-br from-slate-950 via-emerald-950/80 to-slate-900"
      aria-hidden="true"
    >
      <div
        class="pointer-events-none absolute inset-0 opacity-[0.12]"
        style="
          background-image:
            radial-gradient(circle at 20% 20%, #34d399 0%, transparent 45%),
            radial-gradient(circle at 80% 30%, #38bdf8 0%, transparent 40%),
            radial-gradient(circle at 50% 80%, #a78bfa 0%, transparent 45%);
        "
      />
      <div
        class="absolute inset-x-0 top-[12dvh] bottom-[42dvh] flex flex-col items-center justify-center px-6 text-center"
      >
        <p
          class="max-w-xl text-4xl font-bold leading-tight tracking-tight text-white drop-shadow-lg sm:text-5xl [overflow-wrap:anywhere]"
        >
          {{ heading }}
        </p>
        <p
          v-if="backPlain !== '—' && backPlain !== heading"
          class="mt-4 max-w-lg text-base text-white/75 line-clamp-4 sm:text-lg [overflow-wrap:anywhere]"
        >
          {{ backPlain }}
        </p>
      </div>
    </div>

    <!--
      Nested overflow+touch-pan-y blocks reel snap swipes when the panel does not
      actually scroll. Only become a scrollport when content overflows.
    -->
    <div
      ref="textPanelRef"
      class="absolute inset-x-0 bottom-0 z-10 max-h-[55dvh] text-white bg-gradient-to-t from-black/92 via-black/80 to-transparent px-4 pb-[max(1rem,env(safe-area-inset-bottom,0px))] pt-12 sm:px-6"
      :class="
        panelScrollable
          ? 'overflow-y-auto overscroll-y-contain touch-pan-y [-webkit-overflow-scrolling:touch]'
          : 'overflow-hidden'
      "
    >
      <div class="flex justify-between items-start gap-3 mb-3">
        <h2
          class="text-xl font-bold drop-shadow-lg min-w-0 flex-1 break-words sm:text-2xl [overflow-wrap:anywhere]"
          :class="{ 'sr-only': !hasImage }"
        >
          {{ heading }}
        </h2>
        <RouterLink
          v-if="entryHref"
          :to="entryHref"
          class="shrink-0 self-start text-sm px-3 py-1.5 rounded-full bg-white/15 backdrop-blur-sm hover:bg-white/25 transition-colors"
        >
          {{ t('components.tiktoknu.collectionReel.openEntry') }}
        </RouterLink>
      </div>

      <div class="space-y-3 text-gray-100 drop-shadow-lg text-sm sm:text-base pb-1">
        <div>
          <p class="text-xs uppercase tracking-wide text-white/60 mb-1">
            {{ t('components.tiktoknu.collectionReel.frontLabel') }}
          </p>

          <p class="break-words [overflow-wrap:anywhere]">{{ frontPlain }}</p>
        </div>

        <div>
          <p class="text-xs uppercase tracking-wide text-white/60 mb-1">
            {{ t('components.tiktoknu.collectionReel.backLabel') }}
          </p>

          <p class="break-words [overflow-wrap:anywhere]">{{ backPlain }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

type CollectionReelItem = {
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
/** After a failed image fetch, treat the card as text-only. */
const imageFailed = ref(false)

const textPanelRef = ref<HTMLElement | null>(null)
/** True only when the caption panel’s content exceeds max-height (needs its own pan). */
const panelScrollable = ref(false)
let panelResizeObserver: ResizeObserver | null = null

function updatePanelScrollable() {
  const el = textPanelRef.value
  if (!el) {
    panelScrollable.value = false
    return
  }
  panelScrollable.value = el.scrollHeight > el.clientHeight + 1
}

onMounted(() => {
  void nextTick(() => {
    updatePanelScrollable()
    if (!textPanelRef.value || typeof ResizeObserver === 'undefined') return
    panelResizeObserver = new ResizeObserver(() => updatePanelScrollable())
    panelResizeObserver.observe(textPanelRef.value)
  })
})

onUnmounted(() => {
  panelResizeObserver?.disconnect()
  panelResizeObserver = null
})

const hasImage = computed(
  () => !imageFailed.value && (props.item.has_front_image || props.item.has_back_image)
)

watch(
  () => [props.item.item_id, props.item.has_front_image, props.item.has_back_image],
  () => {
    imageLoaded.value = false
    imageFailed.value = false
    void nextTick(updatePanelScrollable)
  }
)

const imageSide = computed(() => (props.item.has_front_image ? 'front' : 'back'))

const imageSrc = computed(
  () =>
    `/api/collections/${props.collectionId}/items/${props.item.item_id}/image/${imageSide.value}`
)

function stripHtml(s: string | null | undefined): string {
  if (!s) return ''
  return s
    .replace(/<[^>]*>/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
}

const heading = computed(() => {
  const w = props.item.word?.trim()
  if (w) return w
  return stripHtml(props.item.free_content_front) || '—'
})

const frontPlain = computed(
  () => stripHtml(props.item.word ?? props.item.free_content_front) || '—'
)

const backPlain = computed(
  () => stripHtml(props.item.definition ?? props.item.free_content_back) || '—'
)

const entryHref = computed(() => {
  const vid = props.item.valsi_id
  if (vid == null || vid === 0) return null
  return `/valsi/${vid}`
})

watch([frontPlain, backPlain, heading], () => {
  void nextTick(updatePanelScrollable)
})

const handleImageError = () => {
  imageFailed.value = true
  imageLoaded.value = true
  void nextTick(updatePanelScrollable)
}
</script>
