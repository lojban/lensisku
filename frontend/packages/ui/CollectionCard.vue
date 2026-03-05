<template>
  <Card elevated class="collection-card">
    <template #header>
      <div class="space-y-2">
        <RouterLink
          :to="`/collections/${collection.collection_id}`"
          class="card-title"
        >
          {{ collection.name }}
        </RouterLink>
        <p v-if="collection.description" class="card-description">
          {{ collection.description }}
        </p>
      </div>
    </template>

    <template #footer>
      <div class="card-footer-block">
        <div class="card-study-area">
          <Button
            :variant="studyButtonVariant"
            :loading="studyLoading"
            class="!h-auto px-8 py-4 rounded-2xl text-base"
            @click="$emit('study', collection)"
          >
            <template #icon>
              <GraduationCap v-if="!studyLoading" class="w-5 h-5 shrink-0" />
            </template>
            {{ studyLoading ? studyingLabel : studyButtonLabel }}
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
              :to="`/collections/${collection.collection_id}/flashcards`"
              class="btn-empty btn-group-item"
            >
              <LayoutGrid class="w-4 h-4 shrink-0" />
              <span>{{ flashcardsButtonLabel }}</span>
            </RouterLink>
          </div>
        </div>
        <div class="card-footer-inner border-t border-gray-100 pt-4 mt-4">
          <div class="card-badges">
            <span
              class="badge"
              :class="collection.is_public ? 'badge-public' : 'badge-private'"
            >
              {{ collection.is_public ? publicLabel : privateLabel }}
            </span>
            <span class="badge badge-muted">
              {{ itemsCountLabel }}
            </span>
          </div>
          <div class="card-meta">
            <span class="card-meta-by">
              {{ createdByLabel }}
              <RouterLink
                :to="`/user/${collection.owner.user_id}`"
                class="card-meta-link"
              >
                {{ collection.owner.username }}
              </RouterLink>
            </span>
            <span class="card-meta-date">
              {{ updatedLabel }} {{ formatDate(collection.updated_at) }}
            </span>
          </div>
        </div>
      </div>
    </template>
  </Card>
</template>

<script setup>
import { computed } from 'vue'
import { GraduationCap, List, LayoutGrid } from 'lucide-vue-next'
import Card from './Card.vue'
import Button from './Button.vue'

const AQUA_VARIANT_WHEEL = [
  'aqua-white',
  'aqua-emerald',
  'aqua-orange',
  'aqua-sky',
  'aqua-purple',
  'aqua-rose',
  'aqua-blue',
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
  studyingLabel: { type: String, default: '…' },
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
</style>
