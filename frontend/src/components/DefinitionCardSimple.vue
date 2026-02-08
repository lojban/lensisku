<template>
  <div class="w-full bg-white border rounded-lg hover:border-blue-300 transition-colors shadow hover:shadow-none p-4">
    <!-- Header Section -->
    <div class="flex flex-col sm:flex-row justify-between items-start gap-4">
      <!-- Word and Type Info -->
      <div class="flex-1 w-full space-y-3">
        <div class="w-full">
          <div class="flex flex-wrap items-center justify-between gap-2 min-w-0">
            <div class="min-w-0 flex-1 flex items-center gap-2 flex-wrap">
              <h2
                v-if="definition.definitionid"
                class="text-base font-semibold min-w-0 max-w-full truncate flex-shrink-0 text-blue-700 hover:text-blue-800 hover:underline"
              >
                <template v-if="isLongValsi">
                  <button
                    type="button"
                    class="text-left align-baseline cursor-pointer bg-transparent border-none p-0 underline-offset-2 hover:underline"
                    :title="t('components.definitionCard.clickToGoToDefinition')"
                    @click.prevent="showValsiPopup = true"
                  >
                    {{ valsiWord }}
                  </button>
                </template>
                <RouterLink v-else :to="valsiHref">
                  {{ valsiWord }}
                </RouterLink>
              </h2>
              <h2
                v-else
                class="text-base font-semibold min-w-0 max-w-full truncate flex-shrink-0 text-gray-800"
              >
                {{ definition.free_content_front || definition.word }}
              </h2>
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
  <Teleport to="body">
    <div
      v-if="showValsiPopup"
      class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/50"
      @click.self="showValsiPopup = false"
    >
      <div
        class="bg-white rounded-lg shadow-xl max-w-[90vw] max-h-[80vh] overflow-auto p-6 cursor-pointer border-2 border-blue-200 hover:border-blue-400 transition-colors"
        role="button"
        tabindex="0"
        :title="t('components.definitionCard.clickToGoToDefinition')"
        @click="goToValsiPage"
        @keydown.enter="goToValsiPage"
        @keydown.space.prevent="goToValsiPage"
      >
        <p class="text-base font-semibold text-blue-700 break-all">{{ valsiWord }}</p>
        <p class="mt-2 text-sm text-gray-500">{{ t('components.definitionCard.clickToGoToDefinition') }}</p>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { RouterLink } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { getTypeClass } from '@/utils/wordTypeUtils';
import LazyMathJax from './LazyMathJax.vue';

const { t } = useI18n();
const router = useRouter();

const LONG_VALSI_THRESHOLD = 50;
const valsiWord = computed(() => props.definition.valsiword ?? props.definition.word ?? '');
const valsiHref = computed(() =>
  props.definition.definitionid
    ? `/valsi/${encodeURIComponent(valsiWord.value)}?highlight_definition_id=${props.definition.definitionid}`
    : ''
);
const isLongValsi = computed(() => (valsiWord.value || '').length > LONG_VALSI_THRESHOLD);
const showValsiPopup = ref(false);

function goToValsiPage() {
  if (valsiHref.value) {
    router.push(valsiHref.value);
    showValsiPopup.value = false;
  }
}

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
