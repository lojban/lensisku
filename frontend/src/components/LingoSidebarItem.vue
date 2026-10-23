<template>
  <RouterLink :to="href" class="lingo-btn" :class="linkClass" @click="$emit('navigate')">
    <template v-if="isCourseRow">
      <div class="lingo-sidebar-course__media">
        <img
          v-if="resolvedImageUrl && !imageFailed"
          :src="resolvedImageUrl"
          alt=""
          class="h-full w-full object-cover"
          @error="imageFailed = true"
        />
        <PlayingCardsFanIcon v-else class="h-5 w-5 text-green-600" aria-hidden="true" />
      </div>
      <div
        class="lingo-sidebar-course__copy"
        :class="{ 'lingo-sidebar-course__copy--solo': !description }"
      >
        <span class="lingo-sidebar-course__title">{{ label }}</span>
        <span v-if="description" class="lingo-sidebar-course__desc">{{ description }}</span>
      </div>
    </template>
    <template v-else>
      <component :is="iconComponent" class="h-8 w-8 shrink-0" />
      <span>{{ label }}</span>
    </template>
  </RouterLink>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { BookOpen, Trophy, Target, ShoppingBag, Dumbbell, Library, Home } from '@lucide/vue'
import { PlayingCardsFanIcon } from '@packages/ui'
import { getCollectionImage } from '@/api'
import { supportedLocales } from '@/config/locales'

const localeRootPathRegex = new RegExp(`^/(?:${supportedLocales.join('|')})/?$`)

const props = defineProps({
  label: { type: String, required: true },
  href: { type: String, required: true },
  description: { type: String, default: '' },
  collectionId: { type: Number, default: null },
  hasCoverImage: { type: Boolean, default: false },
  icon: {
    type: String,
    default: 'learn',
    validator: (v: unknown) =>
      typeof v === 'string' &&
      ['home', 'learn', 'courses', 'leaderboard', 'quests', 'shop', 'library'].includes(v),
  },
})

defineEmits(['navigate'])

const route = useRoute()
const imageFailed = ref(false)

const isCourseRow = computed(() => props.collectionId != null)

const resolvedImageUrl = computed(() => {
  if (!props.hasCoverImage || props.collectionId == null) return null
  return getCollectionImage(props.collectionId, { cached: true })
})

const isActive = computed(() => {
  const hrefPath = props.href.split('?')[0]
  // Home / locale root: exact match only (do not treat every page as a child of `/`).
  if (hrefPath === '/' || localeRootPathRegex.test(hrefPath)) {
    const normalized = route.path.replace(/\/$/, '') || '/'
    const target = hrefPath.replace(/\/$/, '') || '/'
    return normalized === target
  }
  // Classic theme exit: never mark active while still inside Lingo chrome.
  if (/\/lingo\//.test(route.path) || /\/lingo\/?$/.test(route.path)) {
    if (
      /\/collections\/?$/.test(hrefPath) ||
      /\/collections\/[^/]+\/levels\/?$/.test(hrefPath) ||
      /\/collections\/[^/]+\/flashcards(\/study)?\/?$/.test(hrefPath)
    ) {
      return false
    }
  }
  // Classic collections catalog: exact list path only (not /collections/:id/...).
  if (/\/collections\/?$/.test(hrefPath)) {
    return /\/collections\/?$/.test(route.path)
  }
  if (route.path === hrefPath || route.path.startsWith(hrefPath + '/')) return true
  // Levels ladder is the Learn surface (clone /learn).
  if (hrefPath === '/lingo/learn' && /\/lingo\/levels\/?$/.test(route.path)) return true
  if (hrefPath.includes('/collections/') && route.path.startsWith(hrefPath)) return true
  return false
})

const linkClass = computed(() => {
  if (isCourseRow.value) {
    return isActive.value ? 'lingo-btn--sidebar-course-active' : 'lingo-btn--sidebar-course'
  }
  return isActive.value ? 'lingo-btn--sidebar-active' : 'lingo-btn--sidebar'
})

const iconComponent = computed(() => {
  const map = {
    home: Home,
    learn: Dumbbell,
    courses: BookOpen,
    leaderboard: Trophy,
    quests: Target,
    shop: ShoppingBag,
    library: Library,
  }
  return map[props.icon as keyof typeof map] || BookOpen
})

watch(
  () => props.collectionId,
  () => {
    imageFailed.value = false
  }
)
</script>
