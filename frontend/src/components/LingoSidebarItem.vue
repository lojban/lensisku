<template>
  <RouterLink
    :to="href"
    class="lingo-btn"
    :class="isActive ? 'lingo-btn--sidebar-active' : 'lingo-btn--sidebar'"
    @click="$emit('navigate')"
  >
    <component :is="iconComponent" class="h-8 w-8 shrink-0" />
    <span>{{ label }}</span>
  </RouterLink>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { BookOpen, Trophy, Target, ShoppingBag, Dumbbell } from '@lucide/vue'

const props = defineProps({
  label: { type: String, required: true },
  href: { type: String, required: true },
  icon: {
    type: String,
    default: 'learn',
    validator: (v: unknown) =>
      typeof v === 'string' && ['learn', 'courses', 'leaderboard', 'quests', 'shop'].includes(v),
  },
})

defineEmits(['navigate'])

const route = useRoute()
const isActive = computed(
  () => route.path === props.href || route.path.startsWith(props.href + '/')
)

const iconComponent = computed(() => {
  const map = {
    learn: Dumbbell,
    courses: BookOpen,
    leaderboard: Trophy,
    quests: Target,
    shop: ShoppingBag,
  }
  return map[props.icon as keyof typeof map] || BookOpen
})
</script>
