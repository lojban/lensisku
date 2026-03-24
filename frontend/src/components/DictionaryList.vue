<template>

  <div class="dictionary-list divide-y divide-gray-200 p-4">

    <div v-for="entry in entries" :key="entry.w" class="dictionary-item py-2">

      <h3 class="text-lg font-semibold text-gray-800">
         <span v-html="highlightText(entry.w)" />
      </h3>

      <p v-if="entry.d" class="text-gray-600 mathjax-content" v-html="entry.d" />

      <p v-if="entry.n" class="text-gray-500 text-sm mathjax-content" v-html="entry.n" />

      <p v-if="entry.t" class="text-gray-500 text-sm">Type: {{ entry.t }}</p>

      <p v-if="entry.s" class="text-gray-500 text-sm">Selmaho: {{ entry.s }}</p>

      <p v-if="entry.g" class="text-gray-500 text-sm">Gloss: {{ entry.g }}</p>

    </div>

  </div>

</template>

<script setup lang="ts">
import { watch, onMounted, onUpdated, nextTick } from 'vue'
import type { PropType } from 'vue'

type DictionaryListEntry = {
  w: string
  d?: string
  n?: string
  t?: string
  s?: string
  g?: string
}

const props = defineProps({
  entries: {
    type: Array as PropType<DictionaryListEntry[]>,
    default: () => [],
  },
  searchTerm: {
    type: String,
    default: '',
  },
})

const highlightText = (text: string) => {
  if (!props.searchTerm) return text
  const regex = new RegExp(`(${props.searchTerm})`, 'gi')
  return text.replace(regex, '<mark>$1</mark>')
}

const renderMathJax = () => {
  nextTick(() => {
    void window.MathJax?.typesetPromise?.()
  })
}

watch(() => props.entries, renderMathJax)

onMounted(renderMathJax)
onUpdated(renderMathJax)
</script>

<style scoped>
mark {
  background-color: yellow;
  padding: 0.2em 0;
}
.mathjax-content {
  overflow-x: auto;
}
</style>

