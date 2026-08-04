<template>
  <div class="surface-definition-compact min-w-0">
    <div class="flex flex-col sm:flex-row justify-between items-start gap-4">
      <div class="flex-1 w-full min-w-0 space-y-3">
        <div class="w-full flex flex-col gap-2">
          <div class="flex flex-wrap items-center justify-between gap-2">
            <h2 v-if="card.valsiword" class="link-heading-primary">
              <RouterLink :to="valsiLink">
                {{ card.valsiword }}
              </RouterLink>
            </h2>
            <span
              v-if="card.type_name"
              class="px-2 py-1 text-xs font-medium rounded-full"
              :class="getTypeClass(card.type_name)"
            >
              {{ t(`wordTypes.${card.type_name.replace(/'/g, 'h').replace(/ /g, '-')}`) }}
            </span>
            <RouterLink
              v-if="card.selmaho"
              :to="{ path: '/', query: { selmaho: card.selmaho } }"
              class="badge-definition-tag badge-definition-tag--pill inline-block"
            >
              {{ card.selmaho }}
            </RouterLink>
          </div>

          <p v-if="card.langrealname" class="text-xs text-gray-500">
            {{ card.langrealname }}
          </p>

          <div v-for="(field, idx) in card.fields" :key="idx" class="space-y-1">
            <h3 class="text-xs font-semibold text-gray-500 uppercase tracking-wide">
              {{ fieldLabel(field) }}
            </h3>
            <div class="text-sm text-gray-800">
              <LazyMathJax :content="field.exact_text" :enable-markdown="true" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { getTypeClass } from '@/utils/wordTypeUtils'
import LazyMathJax from './LazyMathJax.vue'

const { t } = useI18n()

const props = defineProps({
  card: {
    type: Object,
    required: true,
  },
})

interface CardField {
  field: string
  exampleid?: number | null
  exact_text: string
}

const valsiLink = computed(() =>
  props.card.definitionid
    ? `/valsi/${encodeURIComponent(props.card.valsiword.replace(/ /g, '_'))}?highlight_definition_id=${props.card.definitionid}`
    : '#'
)

const fieldLabel = (field: CardField) => {
  const base = field.field.charAt(0).toUpperCase() + field.field.slice(1)
  if (field.field === 'example') {
    return `${base} #${field.exampleid ?? ''}`
  }
  return base
}
</script>
