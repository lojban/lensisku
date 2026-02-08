<template>
  <div class="w-full min-w-0 max-w-full overflow-x-hidden bg-white border rounded-lg hover:border-blue-300 transition-colors shadow hover:shadow-none p-4">
    <!-- Header Section -->
    <div class="flex flex-col sm:flex-row justify-between items-start gap-4 min-w-0">
      <!-- Word and Type Info -->
      <div class="flex-1 w-full min-w-0 space-y-3">
        <div class="w-full min-w-0">
          <div class="flex flex-wrap items-center justify-between gap-2 min-w-0">
            <div class="min-w-0 flex-1 flex items-center gap-2 flex-wrap max-w-full">
              <div class="min-w-0 max-w-full flex items-baseline gap-1">
                <h2
                  v-if="definition.definitionid"
                  :title="valsiDisplayText"
                  class="text-base font-semibold truncate min-w-0 text-blue-700 hover:text-blue-800 hover:underline"
                >
                  <RouterLink :to="`/valsi/${definition.valsiword ?? definition.word}?highlight_definition_id=${definition.definitionid}`">
                    {{ definition.valsiword ?? definition.word }}
                  </RouterLink>
                </h2>
                <h2
                  v-else
                  :title="valsiDisplayText"
                  class="text-base font-semibold truncate min-w-0 text-gray-800"
                >
                  {{ definition.free_content_front || definition.word }}
                </h2>
                <button
                  v-if="definition.definitionid && isValsiLong"
                  type="button"
                  class="shrink-0 p-0.5 text-gray-500 hover:text-blue-600 rounded"
                  :title="t('components.definitionCard.showFullWord')"
                  @click.stop="showFullWordPopover = true"
                >
                  <span class="sr-only">{{ t('components.definitionCard.showFullWord') }}</span>
                  <Expand class="h-4 w-4" aria-hidden="true" />
                </button>
              </div>
              <span
                v-if="definition.type_name && showWordType"
                class="px-2 py-1 text-xs font-medium rounded-full"
                :class="getTypeClass(definition.type_name)"
              >
                {{ t(`wordTypes.${definition.type_name.replace(/'/g, 'h').replace(/ /g, '-')}`) }}
              </span>
              <RouterLink
                v-if="definition.selmaho"
                :to="{ path: '/', query: selmahoLinkQuery }"
                class="px-2 py-1 text-xs font-medium rounded-full bg-purple-100 text-purple-700 hover:bg-purple-200"
              >
                {{ definition.selmaho }}
              </RouterLink>
            </div>
          </div>

          <!-- Definition Content -->
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
              <span
                v-for="keyword in definition.gloss_keywords"
                :key="keyword.word"
                class="px-2 py-1 text-xs bg-blue-50 text-blue-700 rounded"
              >
                {{ keyword.word }}
              </span>
            </div>
          </div>

          <!-- Metadata Row -->
          <div class="flex flex-wrap items-center gap-2 mt-3 text-sm text-gray-500">
            <span v-if="definition.definitionid && definition.langid" class="italic text-gray-600">
              {{ getLanguageName(definition.langid ?? definition.lang_id) }}
            </span>
            <span v-if="definition.definitionid && definition.username">·</span>
            <span v-if="definition.username">
              {{ t('components.definitionCard.by') }}
              <RouterLink
                :to="`/user/${definition.username}`"
                class="text-blue-600 hover:text-blue-800 hover:underline"
              >
                {{ definition.username }}
              </RouterLink>
            </span>
            <span v-if="definition.created_at && definition.username">·</span>
            <span v-if="definition.created_at">
              {{ formatDate(definition.created_at) }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
  <!-- Full word popover for long valsi -->
  <Teleport to="body">
    <div
      v-if="showFullWordPopover"
      class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/30"
      role="dialog"
      aria-modal="true"
      :aria-label="t('components.definitionCard.showFullWord')"
      @click.self="showFullWordPopover = false"
    >
      <div
        ref="fullWordPopoverRef"
        tabindex="-1"
        class="bg-white rounded-lg shadow-xl border border-gray-200 max-w-[min(90vw,28rem)] max-h-[80vh] flex flex-col p-4 outline-none"
        @keydown.escape="showFullWordPopover = false"
      >
        <div class="flex justify-between items-start gap-2 mb-2">
          <h3 class="text-sm font-medium text-gray-700 shrink-0">{{ t('components.definitionCard.showFullWord') }}</h3>
          <button
            type="button"
            class="shrink-0 p-1 text-gray-500 hover:text-gray-700 rounded"
            :aria-label="t('components.definitionCard.closeFullWord')"
            @click="showFullWordPopover = false"
          >
            <span class="sr-only">{{ t('components.definitionCard.closeFullWord') }}</span>
            <span aria-hidden="true">&times;</span>
          </button>
        </div>
        <p class="text-base break-all overflow-y-auto text-gray-900">{{ valsiDisplayText }}</p>
        <RouterLink
          v-if="definition.definitionid"
          :to="`/valsi/${definition.valsiword ?? definition.word}?highlight_definition_id=${definition.definitionid}`"
          class="mt-3 text-sm text-blue-600 hover:text-blue-800 underline"
          @click="showFullWordPopover = false"
        >
          {{ t('components.definitionCard.goToValsiPage') }}
        </RouterLink>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
import { computed, ref, watch, nextTick } from 'vue';
import { RouterLink } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { Expand } from 'lucide-vue-next';
import { getTypeClass } from '@/utils/wordTypeUtils';
import LazyMathJax from './LazyMathJax.vue';

const { t } = useI18n();

const props = defineProps({
  definition: {
    type: Object,
    required: true,
  },
  languages: {
    type: Array,
    required: true,
  },
  showWordType: {
    type: Boolean,
    default: true,
  },
});

const showFullWordPopover = ref(false);
const fullWordPopoverRef = ref(null);

watch(showFullWordPopover, async (open) => {
  if (open) {
    await nextTick();
    fullWordPopoverRef.value?.focus?.();
  }
});

const valsiDisplayText = computed(() =>
  props.definition.valsiword ?? props.definition.word ?? props.definition.free_content_front ?? ''
);
const isValsiLong = computed(() => {
  const text = valsiDisplayText.value;
  return typeof text === 'string' && text.length > 80;
});

const selmahoLinkQuery = computed(() => ({
  selmaho: props.definition.selmaho,
}));

const getLanguageName = (langId) => {
  const lang = props.languages.find((l) => l.langid === langId);
  return lang ? lang.realname : '';
};

const formatDate = (dateString) => {
  const date = new Date(dateString);
  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  }).format(date);
};
</script>
