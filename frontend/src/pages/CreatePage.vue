<template>
  <div class="max-w-2xl mx-auto py-6 space-y-5">
    <h1 class="text-2xl font-bold text-gray-800">{{ t('creation.create') }}</h1>
    <p class="text-gray-600">{{ t('creation.choose') }}</p>
    <RouterLink
      v-for="choice in choices"
      :key="choice.path"
      :to="localePath(choice.path)"
      class="flex items-center gap-4 bg-white border rounded-xl p-5 hover:border-blue-500 focus-visible:outline-blue-600"
    >
      <component :is="choice.icon" class="h-6 w-6 text-blue-600 shrink-0" />
      <div>
        <h2 class="font-semibold text-gray-900">{{ choice.label }}</h2>
        <p class="text-sm text-gray-600 mt-1">{{ choice.description }}</p>
      </div>
      <ChevronRight class="h-5 w-5 ml-auto text-gray-400" />
    </RouterLink>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Waves, BookOpen, FileText, ChevronRight } from '@lucide/vue'
import { useLocalePath } from '@/composables/useLocalePath'
import { useSeoHead } from '@/composables/useSeoHead'
const { t } = useI18n()
const localePath = useLocalePath()
useSeoHead({ title: () => t('creation.create') })
const choices = computed(() => [
  {
    path: '/comments/new-thread',
    icon: Waves,
    label: t('creation.newDiscussion'),
    description: t('creation.discussionDescription'),
  },
  {
    path: '/valsi/add',
    icon: BookOpen,
    label: t('creation.newDefinition'),
    description: t('creation.definitionDescription'),
  },
  {
    path: '/wiki/add',
    icon: FileText,
    label: t('creation.newWikiPage'),
    description: t('creation.wikiDescription'),
  },
])
</script>
