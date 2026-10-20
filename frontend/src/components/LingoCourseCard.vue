<template>
  <button
    type="button"
    class="lingo-course-card"
    :disabled="disabled"
    @click="$emit('select', collection.collection_id)"
  >
    <div class="flex min-h-[24px] w-full items-center justify-end">
      <div v-if="isActive" class="flex items-center justify-center rounded-md bg-green-600 p-1.5">
        <Check class="h-4 w-4 stroke-[3] text-white" />
      </div>
    </div>

    <div
      class="flex h-[70px] w-[93px] shrink-0 items-center justify-center overflow-hidden rounded-lg border border-neutral-200 bg-neutral-50 drop-shadow-md"
    >
      <img
        v-if="coverImageUrl && !imageFailed"
        :src="coverImageUrl"
        :alt="collection.name"
        class="collection-cover-thumb"
        @error="imageFailed = true"
      />
      <PlayingCardsFanIcon v-else class="h-9 w-9 text-green-600" />
    </div>

    <div class="mt-3 flex w-full min-w-0 flex-col items-center gap-1 px-0.5 text-center">
      <p class="w-full font-bold text-neutral-700 line-clamp-2">{{ collection.name }}</p>
      <p v-if="collection.description" class="card-description w-full text-center">
        {{ collection.description }}
      </p>
    </div>
  </button>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { PlayingCardsFanIcon } from '@packages/ui'
import { Check } from '@lucide/vue'
import { getCollectionImage } from '@/api'

const props = defineProps({
  collection: { type: Object, required: true },
  isActive: { type: Boolean, default: false },
  disabled: { type: Boolean, default: false },
})

defineEmits(['select'])

const imageFailed = ref(false)

const coverImageUrl = computed(() => {
  if (!props.collection?.has_cover_image) return null
  return getCollectionImage(props.collection.collection_id, { cached: true })
})

watch(
  () => props.collection?.collection_id,
  () => {
    imageFailed.value = false
  }
)
</script>
