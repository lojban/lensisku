<template>
  <div class="text-gray-500 text-xs italic">
    <span>{{ step.action }}</span>
    <span class="block mt-0.5">— {{ step.result }}</span>

    <!-- Structured semantic search results: nested foldables -->
    <div
      v-if="semanticPayload?.results?.length"
      class="mt-1.5 not-italic space-y-1"
    >
      <details class="rounded border border-gray-200 bg-gray-50/80 px-2 py-1">
        <summary
          class="cursor-pointer text-gray-700 text-xs font-medium hover:underline select-none list-none [&::-webkit-details-marker]:hidden"
        >
          {{ $t('assistantChat.returnedDefinitions', { n: semanticPayload.results.length }) }}
        </summary>
        <ul class="mt-2 space-y-1 pl-0 list-none">
          <li
            v-for="(row, i) in semanticPayload.results"
            :key="i"
          >
            <details class="rounded border border-gray-100 bg-white px-2 py-1 text-xs">
              <summary
                class="cursor-pointer text-gray-800 hover:underline select-none list-none flex flex-wrap items-baseline gap-x-1 gap-y-0 [&::-webkit-details-marker]:hidden"
              >
                <span class="font-mono font-semibold">{{ row.valsi }}</span>
                <span
                  v-if="row.lang"
                  class="text-gray-500 font-normal"
                >· {{ row.lang }}</span>
                <span
                  v-if="row.similarity != null"
                  class="text-gray-400 font-normal text-[10px]"
                >sim {{ formatSimilarity(row.similarity) }}</span>
              </summary>
              <div
                v-if="row.definition"
                class="mt-1.5 pl-1 border-l-2 border-gray-200 text-gray-700 whitespace-pre-wrap break-words"
              >
                {{ row.definition }}
              </div>
              <div
                v-if="row.notes"
                class="mt-1 text-[11px] text-gray-500 whitespace-pre-wrap break-words"
              >
                <span class="font-medium text-gray-600">{{ $t('assistantChat.notesLabel') }}</span>
                {{ row.notes }}
              </div>
            </details>
          </li>
        </ul>
        <p
          v-if="semanticPayload.total != null && semanticPayload.total > semanticPayload.results.length"
          class="text-[11px] text-gray-500 mt-2 mb-0"
        >
          {{ $t('assistantChat.totalMatchingHint', { total: semanticPayload.total }) }}
        </p>
      </details>
    </div>

    <!-- Tool returned an error object -->
    <div
      v-else-if="semanticPayload?.error"
      class="mt-1.5 not-italic"
    >
      <details class="rounded border border-amber-100 bg-amber-50/60 px-2 py-1 text-xs">
        <summary class="cursor-pointer text-amber-900 font-medium hover:underline select-none list-none [&::-webkit-details-marker]:hidden">
          {{ $t('assistantChat.toolErrorSummary') }}
        </summary>
        <p class="mt-1.5 text-amber-900 whitespace-pre-wrap break-words">
          {{ semanticPayload.error }}
        </p>
      </details>
    </div>

    <!-- Unparseable or non-semantic payload: raw fold -->
    <div
      v-else-if="step.tool_output"
      class="mt-1 text-gray-400 not-italic"
    >
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
      >{{ step.tool_output }}</pre>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  step: {
    type: Object,
    required: true,
  },
  showRawOutput: {
    type: Boolean,
    default: false,
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

function formatSimilarity(s) {
  if (typeof s !== 'number' || Number.isNaN(s)) return String(s)
  return s.toFixed(3)
}
</script>
