<template>
  <aside class="lingo-sidebar-rail" :class="railClass">
    <RouterLink
      to="/lingo/courses"
      class="lingo-sidebar-brand no-underline"
      @click="$emit('navigate')"
    >
      <GraduationCap class="h-10 w-10 shrink-0 text-green-600" />
      <span>{{ t('lingo.appName') }}</span>
    </RouterLink>

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
      <LingoSidebarItem
        :label="t('lingo.quests')"
        href="/lingo/quests"
        icon="quests"
        @navigate="$emit('navigate')"
      />
      <LingoSidebarItem
        :label="t('lingo.shop')"
        href="/lingo/shop"
        icon="shop"
        @navigate="$emit('navigate')"
      />
    </nav>
  </aside>
</template>

<script setup lang="ts">
import { GraduationCap } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import LingoSidebarItem from './LingoSidebarItem.vue'

defineProps({
  /** Extra classes for the rail (e.g. drawer vs desktop). */
  railClass: { type: String, default: '' },
})

defineEmits(['navigate'])

const { t } = useI18n()
</script>
