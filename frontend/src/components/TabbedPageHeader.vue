<template>

  <div class="mb-2">

    <nav class="ui-tab-strip" aria-label="Tabs">
       <NotebookTab
        v-for="tab in tabs"
        :key="tab.key"
        :active="activeTab === tab.key"
        :label="tab.label"
        @click="$emit('tab-click', tab.key)"
        > <template #icon> <component :is="tab.icon" v-if="tab.icon" /> </template> </NotebookTab
      >
    </nav>

    <h2 class="text-2xl font-bold text-gray-800 my-2 block md:hidden"> {{ pageTitle }} </h2>

  </div>

</template>

<script setup lang="ts">
import type { Component, PropType } from 'vue'
import NotebookTab from '@/components/NotebookTab.vue'

defineProps({
  tabs: {
    type: Array as PropType<Array<{ key: string; label: string; icon?: Component }>>,
    required: true,
  },
  activeTab: {
    type: String,
    required: true,
  },
  pageTitle: {
    type: String,
    required: true,
  },
})

defineEmits(['tab-click'])
</script>

