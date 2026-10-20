<template>
  <aside class="lingo-sidebar-rail" :class="railClass">
    <nav class="lingo-sidebar-nav">
      <LingoSidebarItem
        :label="t('lingo.learn')"
        href="/lingo/learn"
        icon="learn"
        @navigate="$emit('navigate')"
      />
      <LingoSidebarItem
        :label="t('lingo.courses')"
        href="/lingo/courses"
        icon="courses"
        @navigate="$emit('navigate')"
      />
      <LingoSidebarItem
        :label="t('lingo.leaderboard')"
        href="/lingo/leaderboard"
        icon="leaderboard"
        @navigate="$emit('navigate')"
      />
      <!-- Quests / shop nav hidden for now (routes still exist). -->
    </nav>
  </aside>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import LingoSidebarItem from './LingoSidebarItem.vue'

defineProps({
  /** Extra classes for the rail (e.g. drawer vs desktop). */
  railClass: { type: String, default: '' },
})

defineEmits(['navigate'])

const { t } = useI18n()
</script>
