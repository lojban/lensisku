<template>
  <aside
    class="lingo-sidebar flex w-full flex-row flex-wrap items-center gap-x-1 gap-y-2 border-b-2 border-slate-200 bg-white px-4"
  >
    <nav class="flex flex-row flex-wrap items-center gap-x-1">
      <span
        v-if="!hideLogo"
        class="lingo-sidebar-item flex h-9 shrink-0 items-center gap-2 px-3 font-medium italic text-slate-400"
      >
        <GraduationCap class="h-5 w-5 shrink-0 text-slate-400" />
        <span>{{ t('lingo.appName') }}</span>
      </span>
      <LingoSidebarItem
        label="Learn"
        href="/lingo/learn"
        icon="learn"
        :horizontal="true"
      />
      <LingoSidebarItem
        label="Courses"
        href="/lingo/courses"
        icon="courses"
        :horizontal="true"
      />
      <LingoSidebarItem
        label="Leaderboard"
        href="/lingo/leaderboard"
        icon="leaderboard"
        :horizontal="true"
      />
      <LingoSidebarItem
        label="Quests"
        href="/lingo/quests"
        icon="quests"
        :horizontal="true"
      />
      <LingoSidebarItem
        label="Shop"
        href="/lingo/shop"
        icon="shop"
        :horizontal="true"
      />
    </nav>
  </aside>
</template>

<script setup>
import { GraduationCap } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import LingoSidebarItem from './LingoSidebarItem.vue'

defineProps({
  hideLogo: { type: Boolean, default: false },
})

const { t } = useI18n()
</script>
