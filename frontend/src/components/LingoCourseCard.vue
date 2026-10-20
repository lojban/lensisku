<template>
  <button
    type="button"
    class="lingo-course-card"
    :class="{ 'lingo-course-card--active': isActive }"
    :disabled="disabled"
    :aria-current="isActive ? 'true' : undefined"
    :aria-label="cardAriaLabel"
    @click="$emit('select', collection.collection_id)"
  >
    <span v-if="isActive" class="lingo-course-card__active-badge" :title="t('lingo.currentCourse')">
      <Dumbbell class="h-3.5 w-3.5" aria-hidden="true" />
      <span class="sr-only">{{ t('lingo.currentCourse') }}</span>
    </span>

    <div class="lingo-course-card__media">
      <img
        v-if="coverImageUrl && !imageFailed"
        :src="coverImageUrl"
        alt=""
        class="collection-cover-thumb"
        @error="imageFailed = true"
      />
      <PlayingCardsFanIcon v-else class="h-9 w-9 text-green-600" aria-hidden="true" />
    </div>

    <div class="lingo-course-card__copy">
      <p class="lingo-course-card__title">{{ collection.name }}</p>
      <p class="lingo-course-card__desc">
        {{ collection.description || '' }}
      </p>
    </div>
  </button>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { PlayingCardsFanIcon } from '@packages/ui'
import { Dumbbell } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import { getCollectionImage } from '@/api'

const props = defineProps({
  collection: { type: Object, required: true },
  isActive: { type: Boolean, default: false },
  disabled: { type: Boolean, default: false },
})

defineEmits(['select'])

const { t } = useI18n()
const imageFailed = ref(false)

const coverImageUrl = computed(() => {
  if (!props.collection?.has_cover_image) return null
  return getCollectionImage(props.collection.collection_id, { cached: true })
})

const cardAriaLabel = computed(() => {
  const name = String(props.collection?.name ?? '')
  return props.isActive ? `${name}. ${t('lingo.currentCourse')}` : name
})

watch(
  () => props.collection?.collection_id,
  () => {
    imageFailed.value = false
  }
)
</script>
