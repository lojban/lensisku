<template>
  <div class="mb-2">
    <nav class="flex flex-wrap gap-x-2 gap-y-0" aria-label="Tabs">
      <NotebookTab
        v-for="tab in tabs"
        :key="tab.key"
        :active="activeTab === tab.key"
        :label="tab.label"
        @click="$emit('tab-click', tab.key)"
      >
        <template #icon>
          <component :is="tab.icon" v-if="tab.icon" />
        </template>
      </NotebookTab>
    </nav>
    <h2 class="text-2xl font-bold text-gray-800 my-2 block md:hidden">
      {{ pageTitle }}
    </h2>
  </div>
</template>

<script setup>
import NotebookTab from '@/components/NotebookTab.vue'

defineProps({
  tabs: {
    type: Array,
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
