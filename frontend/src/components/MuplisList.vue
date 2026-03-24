<template>

  <div
    class="muplis-list bg-white border border-blue-200 rounded-lg hover:border-blue-300 transition-colors cursor-pointer shadow-sm divide-y divide-gray-200 p-4"
  >

    <div v-for="entry in entries" :key="entry.id" class="muplis-item py-2">

      <h3 class="text-lg font-semibold text-gray-800">
         <span v-html="highlightText(entry.lojban)" />
      </h3>

      <p class="text-gray-600" v-html="highlightText(entry.english)" />

    </div>

  </div>

</template>

<script setup lang="ts">
import type { PropType } from 'vue'

const props = defineProps({
  entries: {
    type: Array as PropType<Array<{ id: number | string; lojban: string; english: string }>>,
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
</script>

<style scoped>
mark {
  background-color: yellow;
  padding: 0.2em 0;
}
</style>

