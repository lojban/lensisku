<template>
  <ToolbarSelectDropdown>
    <template v-if="floating" #trigger="{ open }">
      <span class="fab-elevation-shell">
        <FabButton :aria-label="t('creation.create')" aria-haspopup="menu" :aria-expanded="open">
          <Plus
            class="h-8 w-8 shrink-0 transition-transform duration-200"
            stroke-width="2.75"
            :class="{ 'rotate-45': open }"
          />
        </FabButton>
      </span>
    </template>
    <template #icon>
      <Plus class="h-4 w-4" />
    </template>
    <template #label>{{ t('creation.create') }}</template>
    <ToolbarSelectDropdownItem
      v-for="choice in choices"
      :key="choice.path"
      class="!w-80 !max-w-[calc(100vw-2rem)] !whitespace-normal !items-center !gap-4 !rounded-xl !border !border-transparent !px-5 !py-4 text-left hover:!border-blue-500 hover:!bg-white focus-visible:!outline focus-visible:!outline-2 focus-visible:!outline-blue-600"
      @click="router.push(localePath(choice.path))"
    >
      <component
        :is="choice.icon"
        class="fab-menu-icon h-6 w-6 shrink-0"
        :class="choice.iconClass"
        stroke-width="2"
      />
      <span class="flex flex-col items-start">
        <span>{{ choice.label }}</span>
        <span class="text-xs text-gray-500">{{ choice.description }}</span>
      </span>
    </ToolbarSelectDropdownItem>
  </ToolbarSelectDropdown>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Plus, Waves, BookPlus } from '@lucide/vue'
import { FabButton, ToolbarSelectDropdown, ToolbarSelectDropdownItem, WikiIcon } from '@packages/ui'
import { useLocalePath } from '@/composables/useLocalePath'
defineProps({ floating: { type: Boolean, default: false } })
const router = useRouter()
const localePath = useLocalePath()
const { t } = useI18n()
const choices = computed(() => [
  {
    path: '/comments/new-thread',
    icon: Waves,
    iconClass: 'fab-menu-icon--discussion',
    label: t('creation.newDiscussion'),
    description: t('creation.discussionDescription'),
  },
  {
    path: '/valsi/add',
    icon: BookPlus,
    iconClass: 'fab-menu-icon--definition',
    label: t('creation.newDefinition'),
    description: t('creation.definitionDescription'),
  },
  {
    path: '/wiki/add',
    icon: WikiIcon,
    iconClass: 'fab-menu-icon--definition',
    label: t('creation.newWikiPage'),
    description: t('creation.wikiDescription'),
  },
])
</script>
