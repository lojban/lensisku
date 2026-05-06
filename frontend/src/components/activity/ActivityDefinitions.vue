<template>
  <div class="space-y-4">
    <div v-if="definitions.length === 0" class="text-center py-8 bg-gray-50 rounded-lg">
      <Book class="mx-auto h-12 w-12 text-blue-400" />
      <p class="text-gray-600">{{ t('activity.noDefinitions') }}</p>
    </div>
    <DefinitionCard
      v-for="def in definitions"
      v-else
      :key="def.definitionid"
      :definition="def"
      :languages="languages"
      :disable-toolbar="true"
      :disable-discussion-button="false"
      :show-word-type="true"
      :show-audio="true"
    />
  </div>
</template>

<script setup lang="ts">
import { Book } from 'lucide-vue-next'
import { ref, onMounted } from 'vue'

import { getLanguages } from '@/api' // Import getLanguages
import DefinitionCard from '@/components/DefinitionCard.vue' // Import DefinitionCard

import { useI18n } from 'vue-i18n'
import type { PropType } from 'vue'

const { t } = useI18n()

defineProps({
  definitions: {
    type: Array as PropType<Array<{ definitionid: number } & Record<string, unknown>>>,
    required: true,
  },
  formatDate: {
    type: Function,
    required: true,
  },
})

const languages = ref([])

onMounted(async () => {
  try {
    const response = await getLanguages()
    languages.value = response.data
  } catch (error) {
    console.error('Error fetching languages:', error)
    // Handle error appropriately, maybe show a message
  }
})
</script>
