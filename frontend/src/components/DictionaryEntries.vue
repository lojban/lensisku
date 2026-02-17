<template>
  <div class="dictionary-entries space-y-4">
    <LoadingSpinner v-if="isLoading" />

    <!-- Corpus entries with batched rendering -->
    <div
      v-else-if="!isLoading && !error"
      class="grid gap-4 mb-6"
    >
      <!-- Decomposition display -->
      <AlertComponent
        v-if="decomposition?.length"
        type="tip"
        :label="t('components.dictionaryEntries.decomposition')"
      >
        <div class="inline-flex items-center gap-1">
          <template
            v-for="(word, index) in decomposition"
            :key="word"
          >
            <h2
              class="text-base font-semibold text-blue-700 hover:text-blue-800 hover:underline truncate flex-shrink-0"
            >
              <RouterLink :to="{ path: `/valsi/${word.replace(/ /g, '_')}`, query: { langid: definitions[0]?.langid } }">
                {{ word }}
              </RouterLink>
            </h2>
            <span
              v-if="index < decomposition.length - 1"
              class="text-aqua-500"
            >+</span>
          </template>
        </div>
      </AlertComponent>
      <template
        v-for="(batch, batchIndex) in definitionBatches"
        :key="batchIndex"
      >
        <div
          v-for="def in batch"
          v-show="isVisible(batchIndex)"
          :key="def.definitionid"
        >
          <DefinitionCard
            :definition="def"
            :languages="languages"
            :show-score="props.showScores"
            :disable-toolbar="true"
            :disable-owner-only-lock="true"
            :collections="collections"
            @collection-updated="$emit('collection-updated', $event)"
          />
        </div>
      </template>
    </div>

    <div
      v-if="!isLoading && definitions.length === 0"
      class="text-center py-8 text-gray-600"
    >
      {{ t('components.dictionaryEntries.noEntries') }}
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';

import AlertComponent from '@/components/AlertComponent.vue';
import LoadingSpinner from '@/components/LoadingSpinner.vue';
import DefinitionCard from './DefinitionCard.vue';

const { t } = useI18n();
defineEmits(['collection-updated']);

const props = defineProps({
  definitions: {
    type: Array,
    required: true,
  },
  isLoading: {
    type: Boolean,
    default: false,
  },
  error: {
    type: String,
    default: '',
  },
  languages: {
    type: Array,
    required: true,
  },
  showScores: {
    type: Boolean,
    default: false,
  },
  collections: {
    type: Array,
    default: () => [],
  },
  decomposition: {
    type: Array,
    default: () => [],
  },
})

const BATCH_SIZE = 10 // Process definitions in batches of 10
const visibleBatches = ref(new Set([0])) // Track which batches are visible

// Split definitions into batches
const definitionBatches = computed(() => {
  const batches = []
  for (let i = 0; i < props.definitions.length; i += BATCH_SIZE) {
    batches.push(props.definitions.slice(i, i + BATCH_SIZE))
  }
  return batches
})

// Check if a batch should be visible
const isVisible = (batchIndex) => {
  return visibleBatches.value.has(batchIndex)
}

// Intersection Observer to load more batches as user scrolls
onMounted(() => {
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          const batchIndex = parseInt(entry.target.dataset.batchIndex)
          if (!visibleBatches.value.has(batchIndex)) {
            visibleBatches.value.add(batchIndex)
            // Load next batch preemptively
            if (batchIndex + 1 < definitionBatches.value.length) {
              visibleBatches.value.add(batchIndex + 1)
            }
          }
        }
      })
    },
    { rootMargin: '100px' }
  )

  // Observe each batch container
  document.querySelectorAll('[data-batch-index]').forEach((el) => {
    observer.observe(el)
  })
})

// Reset visible batches when definitions change
watch(
  () => props.definitions,
  () => {
    visibleBatches.value = new Set([0])
  },
  { deep: true }
)
</script>
