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
    <template #label
      ><Plus class="h-4 w-4" /><span>{{ t('creation.create') }}</span></template
    >
    <ToolbarSelectDropdownItem
      v-for="choice in choices"
      :key="choice.path"
      @click="router.push(localePath(choice.path))"
    >
      <component
        :is="choice.icon"
        class="fab-menu-icon"
        :class="choice.iconClass"
        stroke-width="2"
      />
      {{ choice.label }}
    </ToolbarSelectDropdownItem>
  </ToolbarSelectDropdown>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Plus, Waves, BookOpen, FileText } from '@lucide/vue'
import { FabButton, ToolbarSelectDropdown, ToolbarSelectDropdownItem } from '@packages/ui'
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
  },
  {
    path: '/valsi/add',
    icon: BookOpen,
    iconClass: 'fab-menu-icon--definition',
    label: t('creation.newDefinition'),
  },
  {
    path: '/wiki/add',
    icon: FileText,
    iconClass: 'fab-menu-icon--definition',
    label: t('creation.newWikiPage'),
  },
])
</script>
