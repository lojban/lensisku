<template>

  <div class="surface-definition-compact">
    <!-- Header Section -->
    <div class="flex flex-col sm:flex-row justify-between items-start gap-4">
      <!-- Word and Type Info -->
      <div class="flex-1 w-full min-w-0 space-y-3">

        <div class="w-full flex flex-col gap-2">

          <div class="flex flex-wrap items-center justify-between gap-2">

            <div v-if="definition.definitionid" class="w-auto min-w-0 max-w-[14rem] sm:max-w-[18rem] md:max-w-[22rem]">

              <h2 v-if="(definition.valsiword ?? definition.word)" class="link-heading-primary">
                <span v-if="isValsiTruncated" class="cursor-pointer"
                  :title="t('components.definitionCard.clickToSeeFullWord')" @click="showValsiModal = true"> {{
                    displayedValsi }} </span>
                <RouterLink v-else :to="valsiDefinitionLink"> {{ definition.valsiword ?? definition.word }}
                </RouterLink>
              </h2>

            </div>

          </div>
          <div
            v-if="definition.definitionid && ((definition.type_name && showWordType) || definition.selmaho)"
            class="flex flex-wrap items-center gap-2">
            <span v-if="definition.type_name && showWordType" class="px-2 py-1 text-xs font-medium rounded-full"
              :class="getTypeClass(definition.type_name)">
              {{ t(`wordTypes.${definition.type_name.replace(/'/g,
                'h').replace(/ /g, '-')}`) }}
            </span>
            <RouterLink v-if="definition.selmaho" :to="{ path: '/', query: selmahoLinkQuery }"
              class="badge-definition-tag badge-definition-tag--pill inline-block"
              :title="definition.selmaho.length > MAX_VALSI_DISPLAY_LENGTH
                ? definition.selmaho
                : undefined
                ">
              {{ displayedSelmaho }}
            </RouterLink>
          </div>
          <!-- Definition Content -->
          <div v-if="showExpandedFrontContent"
            class="mt-2 text-sm font-semibold text-gray-800 whitespace-pre-wrap break-words">
            {{ displayedFreeContent }}
          </div>
          <div class="mt-3">
            <LazyMathJax :content="definition.definition" :enable-markdown="true" />
          </div>
          <!-- Notes -->
          <div v-if="definition.notes" class="mt-3 pt-2 border-t">

            <div class="text-sm text-gray-600 bg-gray-50 rounded p-2">
              <LazyMathJax :content="definition.notes" :enable-markdown="true" />
            </div>

          </div>
          <!-- Gloss Keywords -->
          <div v-if="definition.gloss_keywords && definition.gloss_keywords.length > 0" class="mt-3 pt-2 border-t">

            <div class="flex flex-wrap gap-1">
              <span v-for="keyword in definition.gloss_keywords" :key="keyword.word"
                class="px-2 py-1 text-xs bg-blue-50 text-blue-700 rounded"> {{ keyword.word }} </span>
            </div>

          </div>
          <!-- Metadata Row -->
          <div class="flex flex-wrap items-center gap-2 mt-3 text-sm text-gray-500">
            <span v-if="definition.definitionid && definition.langid" class="italic text-gray-600"> {{
              getLanguageName(definition.langid ?? definition.lang_id) }} </span> <span
              v-if="definition.definitionid && definition.username">·</span> <span v-if="definition.username"> {{
                t('components.definitionCard.by') }} <RouterLink :to="`/user/${definition.username}`"
                class="text-blue-600 hover:text-blue-800 hover:underline"> {{ definition.username }} </RouterLink>
            </span> <span v-if="definition.created_at && definition.username">·</span> <span
              v-if="definition.created_at"> {{ formatDate(definition.created_at) }} </span> <span
              v-if="props.showScore && definition.similarity" class="italic"> · {{
                t('components.definitionCard.similarity', {
                  percent: Math.round(definition.similarity * 100),
                })
              }} </span>
          </div>

        </div>

      </div>

    </div>

  </div>
  <ModalComponent :show="showValsiModal" :title="t('components.definitionCard.fullWordModalTitle')"
    @close="showValsiModal = false">
    <p class="text-sm text-gray-600 mb-3">{{ t('components.definitionCard.fullWordModalHint') }}</p>
    <RouterLink :to="valsiDefinitionLink"
      class="text-blue-700 hover:text-blue-800 hover:underline break-words font-medium" @click="showValsiModal = false">
      {{ definition.valsiword ?? definition.word }} </RouterLink>
  </ModalComponent>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { getTypeClass } from '@/utils/wordTypeUtils'
import LazyMathJax from './LazyMathJax.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import type { PropType } from 'vue'

const { t } = useI18n()

const MAX_VALSI_DISPLAY_LENGTH = 30
const showValsiModal = ref(false)

type LanguageRow = { langid: number; realname: string }

const props = defineProps({
  definition: {
    type: Object,
    required: true,
  },
  languages: {
    type: Array as PropType<LanguageRow[]>,
    required: true,
  },
  showWordType: {
    type: Boolean,
    default: true,
  },
  showScore: {
    type: Boolean,
    default: false,
  },
})

const valsiWord = computed(() => props.definition.valsiword ?? props.definition.word)
const displayedValsi = computed(() =>
  valsiWord.value.length > MAX_VALSI_DISPLAY_LENGTH
    ? valsiWord.value.slice(0, MAX_VALSI_DISPLAY_LENGTH) + '…'
    : valsiWord.value
)
const isValsiTruncated = computed(() => valsiWord.value.length > MAX_VALSI_DISPLAY_LENGTH)
const valsiDefinitionLink = computed(() =>
  props.definition.definitionid
    ? `/valsi/${encodeURIComponent(valsiWord.value.replace(/ /g, '_'))}?highlight_definition_id=${props.definition.definitionid}`
    : '#'
)
const displayedFreeContent = computed(() => {
  const raw = props.definition.free_content_front || props.definition.word || ''
  return raw
})
const showExpandedFrontContent = computed(
  () => !props.definition.definitionid && displayedFreeContent.value.trim().length > 0
)
const displayedSelmaho = computed(() => {
  const s = props.definition.selmaho || ''
  return s.length > MAX_VALSI_DISPLAY_LENGTH ? s.slice(0, MAX_VALSI_DISPLAY_LENGTH) + '…' : s
})

const selmahoLinkQuery = computed(() => ({
  selmaho: props.definition.selmaho,
}))

const getLanguageName = (langId: number) => {
  const lang = props.languages.find((l) => l.langid === langId)
  return lang ? lang.realname : ''
}

const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  }).format(date)
}
</script>
