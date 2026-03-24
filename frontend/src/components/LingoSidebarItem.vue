<template>
  <RouterLink
    :to="href"
    class="lingo-sidebar-item flex items-center font-medium transition-colors"
    :class="[
      horizontal
        ? 'h-9 shrink-0 gap-2 rounded-lg px-3 text-slate-600 hover:bg-slate-100'
        : 'h-[52px] w-full gap-5 rounded-xl px-4 text-left',
      isActive
        ? 'bg-green-100 text-green-700'
        : horizontal
          ? ''
          : 'text-slate-600 hover:bg-slate-100',
    ]"
  >
    <component :is="iconComponent" :class="horizontal ? 'h-5 w-5 shrink-0' : 'h-8 w-8 shrink-0'" />
    <span>{{ label }}</span>
  </RouterLink>
</template>

<script setup>
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { BookOpen, Trophy, Target, ShoppingBag, Dumbbell } from 'lucide-vue-next'

const props = defineProps({
  label: { type: String, required: true },
  href: { type: String, required: true },
  icon: {
    type: String,
    default: 'learn',
    validator: (v) => ['learn', 'courses', 'leaderboard', 'quests', 'shop'].includes(v),
  },
  horizontal: { type: Boolean, default: false },
})

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
  return map[props.icon] || BookOpen
})
</script>
