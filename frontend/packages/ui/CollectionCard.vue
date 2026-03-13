<template>
  <Card elevated class="collection-card">
    <template #header>
      <div class="space-y-2 flex flex-col items-center">
        <div class="flex items-center justify-center gap-2 w-full min-w-0">
          <RouterLink
            :to="collection.has_flashcards
              ? `/collections/${collection.collection_id}/flashcards`
              : `/collections/${collection.collection_id}`"
            class="card-title min-w-0 flex-1 line-clamp-2 whitespace-normal text-center"
          >
            {{ collection.name }}
          </RouterLink>
        </div>
        <p v-if="collection.description" class="card-description text-center">
          {{ collection.description }}
        </p>
      </div>
    </template>

    <template #footer>
      <div class="card-footer-block">
        <div v-if="collection.has_flashcards" class="card-study-area card-study-area-compact">
          <Button
            :variant="studyButtonVariant"
            :loading="studyLoading"
            class="!h-auto px-5 py-2.5 rounded-xl text-sm"
            @click="$emit('study', collection)"
          >
            <template #icon>
              <GraduationCap v-if="!studyLoading" class="w-4 h-4 shrink-0" />
            </template>
            {{ studyButtonLabel }}
          </Button>
        </div>
        <div class="card-actions">
          <div class="btn-group-forced flex flex-nowrap justify-center" role="group">
            <RouterLink
              :to="`/collections/${collection.collection_id}`"
              class="btn-empty btn-group-item"
            >
              <List class="w-4 h-4 shrink-0" />
              <span>{{ collectionButtonLabel }}</span>
            </RouterLink>
            <RouterLink
              v-if="collection.has_flashcards"
              :to="`/collections/${collection.collection_id}/flashcards`"
              class="btn-empty btn-group-item"
            >
              <LayoutGrid class="w-4 h-4 shrink-0" />
              <span>{{ flashcardsButtonLabel }}</span>
            </RouterLink>
          </div>
        </div>
        <div class="card-footer-inner border-t border-gray-100 pt-3 mt-3 w-full">
          <div class="card-meta card-meta-row">
            <span class="card-meta-by">
              {{ createdByLabel }}
              <RouterLink
                :to="`/user/${collection.owner.user_id}`"
                class="card-meta-link"
              >
                {{ collection.owner.username }}
              </RouterLink>
            </span>
            <span
              class="card-meta-date"
              :aria-label="updatedLabel + ' ' + formatDate(collection.updated_at)"
              :title="updatedLabel + ' ' + formatDate(collection.updated_at)"
            >
              <CalendarClock class="card-meta-icon" aria-hidden="true" />
              {{ formatDate(collection.updated_at) }}
            </span>
            <!-- <span class="badge badge-muted">
              {{ itemsCountLabel }}
            </span> -->
          </div>
        </div>
      </div>
    </template>
  </Card>
</template>

<script setup>
import { computed } from 'vue'
import { GraduationCap, List, LayoutGrid, CalendarClock } from 'lucide-vue-next'
import Card from './Card.vue'
import Button from './Button.vue'

const AQUA_VARIANT_WHEEL = [
  'aqua-white',
  'aqua-red',
  'aqua-orange',
  'aqua-amber',
  'aqua-yellow',
  'aqua-lime',
  'aqua-teal',
  'aqua-emerald',
  'aqua-cyan',
  'aqua-sky',
  'aqua-blue',
  'aqua-indigo',
  'aqua-violet',
  'aqua-purple',
  'aqua-fuchsia',
  'aqua-pink',
  'aqua-rose',
  'aqua-slate',
  'aqua-zinc',
]

function hashString(str) {
  if (!str || typeof str !== 'string') return 0
  let h = 0
  for (let i = 0; i < str.length; i++) {
    h = ((h << 5) - h) + str.charCodeAt(i)
    h |= 0
  }
  return h
}

const props = defineProps({
  collection: {
    type: Object,
    required: true,
  },
  studyLoading: {
    type: Boolean,
    default: false,
  },
  formatDate: {
    type: Function,
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
})

const studyButtonVariant = computed(() => {
  const name = props.collection?.name ?? ''
  const index = ((hashString(name) % AQUA_VARIANT_WHEEL.length) + AQUA_VARIANT_WHEEL.length) % AQUA_VARIANT_WHEEL.length
  return AQUA_VARIANT_WHEEL[index]
})

defineEmits(['study'])
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
