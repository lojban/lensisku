<template>
  <div class="text-gray-500 text-xs italic">
    <div
      v-if="step.assistant_reasoning && String(step.assistant_reasoning).trim()"
      class="mb-2 not-italic rounded border border-indigo-100 bg-indigo-50/70 px-2 py-1.5 text-gray-800"
    >
      <span class="mb-1 block text-[10px] font-semibold uppercase tracking-wide text-indigo-700/90">
        {{ $t('assistantChat.reasoningLabel') }}
      </span>
      <LazyMathJax
        :content="String(step.assistant_reasoning)"
        :enable-markdown="true"
        :enable-curly-links="false"
        :lang-id="langId"
        class="block text-xs text-gray-800 leading-snug"
      />
    </div>
    <span>{{ step.action }}</span>
    <span v-if="!batchSearches.length && !semanticPayload?.results?.length" class="block mt-0.5"
      >— {{ step.result }}</span
    >
    <!-- Batched semantic search: one block per query string -->
    <div v-if="batchSearches.length" class="mt-1.5 not-italic space-y-2">
      <details v-for="(block, bi) in batchSearches" :key="bi" class="assistant-fold-details">
        <summary class="assistant-fold-summary">
          <span class="font-mono text-[11px]">{{ block.query }}</span> —
          <template v-if="block.error">{{ $t('assistantChat.toolErrorSummary') }}</template>
          <template v-else>{{
            $t('assistantChat.returnedDefinitions', {
              n: (block.results && block.results.length) || 0,
            })
          }}</template>
        </summary>

        <ul v-if="block.results?.length" class="mt-2 space-y-1 pl-0 list-none">
          <li v-for="(row, i) in block.results" :key="i">
            <details class="assistant-fold-details-nested">
              <summary class="assistant-fold-summary-row">
                <span class="font-mono font-semibold">{{ row.valsi }}</span>
                <span v-if="row.lang" class="text-gray-500 font-normal">· {{ row.lang }}</span>
                <span v-if="row.similarity != null" class="text-gray-400 font-normal text-[10px]"
                  >sim {{ formatSimilarity(row.similarity) }}</span
                >
              </summary>

              <div
                v-if="row.definition"
                class="mt-1.5 pl-1 border-l-2 border-gray-200 text-gray-700 break-words"
              >
                <LazyMathJax
                  :content="row.definition"
                  :lang-id="langId"
                  class="block text-xs text-gray-700"
                />
              </div>

              <div v-if="row.notes" class="mt-1 text-[11px] text-gray-500 break-words">
                <span class="font-medium text-gray-600">{{ $t('assistantChat.notesLabel') }}</span>
                <LazyMathJax
                  :content="row.notes"
                  :lang-id="langId"
                  class="inline text-[11px] text-gray-500"
                />
              </div>
            </details>
          </li>
        </ul>

        <p
          v-if="block.error"
          class="mt-2 text-[11px] text-amber-900 whitespace-pre-wrap break-words"
        >
          {{ block.error }}
        </p>
      </details>
    </div>
    <!-- Structured semantic search results (legacy single-query JSON): nested foldables -->
    <div v-else-if="semanticPayload?.results?.length" class="mt-1.5 not-italic space-y-1">
      <details class="assistant-fold-details">
        <summary class="assistant-fold-summary">
          {{ $t('assistantChat.returnedDefinitions', { n: semanticPayload.results.length }) }}
        </summary>

        <ul class="mt-2 space-y-1 pl-0 list-none">
          <li v-for="(row, i) in semanticPayload.results" :key="i">
            <details class="assistant-fold-details-nested">
              <summary class="assistant-fold-summary-row">
                <span class="font-mono font-semibold">{{ row.valsi }}</span>
                <span v-if="row.lang" class="text-gray-500 font-normal">· {{ row.lang }}</span>
                <span v-if="row.similarity != null" class="text-gray-400 font-normal text-[10px]"
                  >sim {{ formatSimilarity(row.similarity) }}</span
                >
              </summary>

              <div
                v-if="row.definition"
                class="mt-1.5 pl-1 border-l-2 border-gray-200 text-gray-700 break-words"
              >
                <LazyMathJax
                  :content="row.definition"
                  :lang-id="langId"
                  class="block text-xs text-gray-700"
                />
              </div>

              <div v-if="row.notes" class="mt-1 text-[11px] text-gray-500 break-words">
                <span class="font-medium text-gray-600">{{ $t('assistantChat.notesLabel') }}</span>
                <LazyMathJax
                  :content="row.notes"
                  :lang-id="langId"
                  class="inline text-[11px] text-gray-500"
                />
              </div>
            </details>
          </li>
        </ul>

        <p
          v-if="
            semanticPayload.total != null && semanticPayload.total > semanticPayload.results.length
          "
          class="text-[11px] text-gray-500 mt-2 mb-0"
        >
          {{ $t('assistantChat.totalMatchingHint', { total: semanticPayload.total }) }}
        </p>
      </details>
    </div>
    <!-- Tool returned an error object -->
    <div v-else-if="semanticPayload?.error" class="mt-1.5 not-italic">
      <details class="assistant-fold-details-warning">
        <summary class="assistant-fold-summary-warning">
          {{ $t('assistantChat.toolErrorSummary') }}
        </summary>

        <p class="mt-1.5 text-amber-900 whitespace-pre-wrap break-words">
          {{ semanticPayload.error }}
        </p>
      </details>
    </div>
    <!-- Unparseable or non-semantic payload: raw fold -->
    <div v-else-if="step.tool_output" class="mt-1 text-gray-400 not-italic">
      <button
        type="button"
        class="text-left underline hover:no-underline focus:outline-none text-xs"
        :aria-expanded="showRawOutput"
        @click="$emit('toggleRaw')"
      >
        {{ showRawOutput ? $t('assistantChat.hideRawOutput') : $t('assistantChat.showRawOutput') }}
      </button>
      <pre
        v-show="showRawOutput"
        class="mt-1 p-1.5 rounded bg-gray-200 text-[10px] overflow-x-auto max-h-48 overflow-y-auto whitespace-pre-wrap break-all"
        >{{ step.tool_output }}</pre
      >
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import LazyMathJax from '@/components/LazyMathJax.vue'

const props = defineProps({
  step: {
    type: Object,
    required: true,
  },
  showRawOutput: {
    type: Boolean,
    default: false,
  },
  langId: {
    type: String,
    default: '',
  },
})

defineEmits(['toggleRaw'])

const semanticPayload = computed(() => {
  const raw = props.step?.tool_output
  if (!raw || typeof raw !== 'string') return null
  try {
    const o = JSON.parse(raw)
    if (o && typeof o === 'object') return o
  } catch (_) {
    /* ignore */
  }
  return null
})

/** New batched tool shape: { searches: [{ query, results?, error? }] } */
const batchSearches = computed(() => {
  const p = semanticPayload.value
  if (!p || !Array.isArray(p.searches)) return []
  return p.searches.filter((s) => s && typeof s === 'object')
})

function formatSimilarity(s) {
  if (typeof s !== 'number' || Number.isNaN(s)) return String(s)
  return s.toFixed(3)
}
</script>
