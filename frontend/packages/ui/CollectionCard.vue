<template>
   <Card elevated class="collection-card"
    > <template #header
      >
      <div class="flex flex-row items-start gap-3 w-full min-w-0">
        <RouterLink
          v-if="coverImageUrl"
          :to="collectionHeaderTo"
          class="flex shrink-0 max-w-full rounded-lg sm:rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500/60 focus-visible:ring-offset-2"
          :aria-label="collection.name"
        >
          <div class="collection-card-logo">
            <img
              :src="coverImageUrl"
              alt=""
              class="collection-cover-thumb"
              loading="lazy"
              decoding="async"
            />
          </div>
        </RouterLink>
        <div v-else class="collection-card-logo-placeholder" aria-hidden="true">
          <BookOpen class="h-6 w-6 sm:h-8 sm:w-8" />
        </div>
        <div class="min-w-0 flex-1 flex flex-col gap-1.5 text-left">
          <RouterLink :to="collectionHeaderTo" class="card-title--multiline">
            {{ collection.name }}
          </RouterLink>
          <p v-if="collection.description" class="card-description">
            {{ collection.description }}
          </p>
        </div>
      </div>
       </template
    > <template #footer
      >
      <div class="card-footer-block">

        <div v-if="collection.has_flashcards" class="card-study-area card-study-area-compact">
           <Button
            :variant="studyButtonVariant"
            :loading="studyLoading"
            class="!h-auto px-5 py-2.5 rounded-xl text-sm"
            @click="$emit('study', collection)"
            > <template #icon
              > <GraduationCap v-if="!studyLoading" class="w-4 h-4 shrink-0" /> </template
            > {{ studyButtonLabel }} </Button
          >
        </div>

        <div class="card-actions">

          <div class="btn-group-forced flex flex-nowrap justify-center" role="group">
             <RouterLink
              :to="`/collections/${collection.collection_id}`"
              class="ui-btn--empty ui-btn--group-item"
              > <List class="w-4 h-4 shrink-0" /> <span>{{ collectionButtonLabel }}</span
              > </RouterLink
            > <RouterLink
              v-if="collection.has_flashcards"
              :to="`/collections/${collection.collection_id}/flashcards`"
              class="ui-btn--empty ui-btn--group-item"
              > <GalleryHorizontalEnd class="w-4 h-4 shrink-0" /> <span>{{ flashcardsButtonLabel }}</span
              > </RouterLink
            >
          </div>

        </div>

        <div class="card-footer-inner border-t border-gray-100 pt-3 mt-3 w-full">

          <div class="card-meta card-meta-row">
             <span class="card-meta-by"
              > {{ createdByLabel }} <RouterLink
                :to="`/user/${collection.owner.user_id}`"
                class="card-meta-link"
                > {{ collection.owner.username }} </RouterLink
              > </span
            > <span
              class="card-meta-date"
              :aria-label="updatedLabel + ' ' + formatDate(collection.updated_at)"
              :title="updatedLabel + ' ' + formatDate(collection.updated_at)"
              > <CalendarClock class="card-meta-icon" aria-hidden="true" /> {{
                formatDate(collection.updated_at)
              }} </span
            > <!-- <span class="badge badge-muted">
              {{ itemsCountLabel }}
            </span> -->
          </div>

        </div>

      </div>
       </template
    > </Card
  >
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { BookOpen, GraduationCap, List, GalleryHorizontalEnd, CalendarClock } from 'lucide-vue-next'
import Card from './Card.vue'
import Button from './Button.vue'

export interface CollectionCardCollection {
  collection_id: number
  name: string
  description?: string | null
  has_flashcards: boolean
  /** Collection cover thumbnail (`collection_images`). */
  has_cover_image?: boolean
  /** Cover or item card images (Tiktoknu CTA, etc.). */
  has_collection_image?: boolean
  /** Discussion comments on this collection's thread (0 if none). */
  comment_count?: number
  updated_at: string
  owner: { user_id: number; username: string }
}

/** `Button` variant suffixes (`ui-btn--${id}`); rotates study CTA color by collection name. */
const STUDY_BUTTON_VARIANT_WHEEL = [
  'neutral',
  'palette-red',
  'warning-orange',
  'amber',
  'warning-yellow',
  'palette-lime',
  'palette-teal',
  'palette-emerald',
  'palette-cyan',
  'palette-sky',
  'palette-blue',
  'palette-indigo',
  'palette-violet',
  'palette-purple',
  'palette-fuchsia',
  'palette-pink',
  'palette-rose',
  'palette-slate',
  'palette-zinc',
]

function hashString(str: string | undefined | null): number {
  if (!str || typeof str !== 'string') return 0
  let h = 0
  for (let i = 0; i < str.length; i++) {
    h = (h << 5) - h + str.charCodeAt(i)
    h |= 0
  }
  return h
}

const props = defineProps({
  collection: {
    type: Object as () => CollectionCardCollection,
    required: true,
  },
  studyLoading: {
    type: Boolean,
    default: false,
  },
  formatDate: {
    type: Function as unknown as () => (d: string) => string,
    required: true,
  },
  studyButtonLabel: { type: String, default: 'Study' },
  collectionButtonLabel: { type: String, default: 'Collection' },
  flashcardsButtonLabel: { type: String, default: 'Flashcards' },
  createdByLabel: { type: String, default: 'by' },
  updatedLabel: { type: String, default: 'Updated' },
  publicLabel: { type: String, default: 'Public' },
  privateLabel: { type: String, default: 'Private' },
  itemsCountLabel: { type: String, default: '0 items' },
  /** Resolved cover URL when `has_cover_image`; omit for placeholder icon. */
  coverImageUrl: { type: String, default: null },
})

const collectionHeaderTo = computed(() =>
  props.collection.has_flashcards
    ? `/collections/${props.collection.collection_id}/flashcards`
    : `/collections/${props.collection.collection_id}`,
)

const studyButtonVariant = computed(() => {
  const name = props.collection?.name ?? ''
  const index =
    ((hashString(name) % STUDY_BUTTON_VARIANT_WHEEL.length) + STUDY_BUTTON_VARIANT_WHEEL.length) %
    STUDY_BUTTON_VARIANT_WHEEL.length
  return STUDY_BUTTON_VARIANT_WHEEL[index]
})

defineEmits<{ study: [collection: CollectionCardCollection] }>()
</script>

<style scoped>
.collection-card {
  @apply flex flex-col h-full;
}

:deep(.card-study-area-compact) {
  min-height: 0;
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
}

.card-meta-row {
  @apply flex flex-wrap items-center gap-x-3 gap-y-1 w-full justify-between;
}

.card-meta-icon {
  @apply w-3.5 h-3.5 inline-block align-middle mr-0.5 text-gray-400;
}
</style>

