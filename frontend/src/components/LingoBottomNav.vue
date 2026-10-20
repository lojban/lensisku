<template>
  <nav class="lingo-bottom-nav" aria-label="Lingo">
    <RouterLink
      v-for="item in items"
      :key="item.href"
      :to="item.href"
      class="lingo-bottom-nav-item"
      :class="{ 'lingo-bottom-nav-item--active': isActive(item.href) }"
    >
      <component :is="item.icon" class="h-6 w-6 shrink-0" />
      <span>{{ item.label }}</span>
    </RouterLink>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { BookOpen, Trophy, Dumbbell } from '@lucide/vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const route = useRoute()

/** Quests / shop nav hidden for now (routes still exist). */
const items = computed(() => [
  { label: t('lingo.learn'), href: '/lingo/learn', icon: Dumbbell },
  { label: t('lingo.courses'), href: '/lingo/courses', icon: BookOpen },
  { label: t('lingo.leaderboard'), href: '/lingo/leaderboard', icon: Trophy },
])

function isActive(href: string) {
  if (route.path === href || route.path.startsWith(href + '/')) return true
  if (href === '/lingo/learn' && /\/lingo\/levels\/?$/.test(route.path)) return true
  return false
}
</script>
