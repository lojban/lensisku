<template>
  <aside class="lingo-sidebar-rail" :class="railClass">
    <nav class="lingo-sidebar-nav">
      <LingoSidebarItem :label="t('logoText')" href="/" icon="home" @navigate="$emit('navigate')" />
      <template v-if="showCourseList">
        <LingoSidebarItem
          :label="t('lingo.allCourses')"
          href="/lingo/courses"
          icon="courses"
          @navigate="$emit('navigate')"
        />
        <div class="lingo-sidebar-nav__separator" role="separator" />
        <LingoSidebarItem
          v-for="course in sidebarCourses"
          :key="course.id"
          :label="course.name"
          :description="course.description"
          :collection-id="course.id"
          :has-cover-image="course.hasCoverImage"
          :href="courseHref(course.id)"
          @navigate="$emit('navigate')"
        />
        <p
          v-if="!isLoading && sidebarCourses.length === 0"
          class="px-2 py-2 text-sm text-slate-500"
        >
          {{ t('lingo.noCourses') }}
        </p>
      </template>
      <template v-else>
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
      </template>
      <div class="lingo-sidebar-nav__spacer" aria-hidden="true" />
      <LingoSidebarItem
        :label="t('lingo.classicCollections')"
        :href="classicThemeHref"
        icon="library"
        @navigate="$emit('navigate')"
      />
    </nav>
  </aside>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import LingoSidebarItem from './LingoSidebarItem.vue'
import { getCollection, searchCollectionsAndUsers } from '@/api'
import { useLingoProgress } from '@/composables/useLingoProgress'
import { useLocalePath } from '@/composables/useLocalePath'
import { classicThemePathForRoute } from '@/utils/lingoClassicTheme'

defineProps({
  /** Extra classes for the rail (e.g. drawer vs desktop). */
  railClass: { type: String, default: '' },
})

defineEmits(['navigate'])

type SidebarCourse = {
  id: number
  name: string
  description: string
  hasCoverImage: boolean
  enriched?: boolean
}

const { t } = useI18n()
const route = useRoute()
const localePath = useLocalePath()
const { recentCourses, activeCourseId, activeCourseName } = useLingoProgress()

const popularIds = ref<number[]>([])
const courseMeta = ref<Record<number, SidebarCourse>>({})
const isLoading = ref(false)
const coursesLoaded = ref(false)

const SIDEBAR_COURSE_LIMIT = 10
const POPULAR_FETCH_PER_KIND = 10

/**
 * Classic theme exit — always in the rail. Maps the current Lingo surface to the
 * matching collections / flashcards route (catalog, levels, or study).
 */
const classicThemeHref = computed(() => localePath(classicThemePathForRoute(route)))

/** Recent/popular course list only on the levels ladder (names are `LingoLevels` / `LingoLevels-en`). */
const showCourseList = computed(() => {
  const name = route.name?.toString() || ''
  return name === 'LingoLevels' || name.startsWith('LingoLevels-')
})

function courseHref(collectionId: number) {
  return `/collections/${collectionId}/lingo/levels`
}

function orderedIds(): number[] {
  const seen = new Set<number>()
  const ids: number[] = []

  const recent = recentCourses.value.map((c) => c.id)
  if (activeCourseId.value != null && !recent.includes(activeCourseId.value)) {
    recent.unshift(activeCourseId.value)
  }

  for (const id of recent) {
    if (seen.has(id)) continue
    seen.add(id)
    ids.push(id)
    if (ids.length >= SIDEBAR_COURSE_LIMIT) return ids
  }

  for (const id of popularIds.value) {
    if (seen.has(id)) continue
    seen.add(id)
    ids.push(id)
    if (ids.length >= SIDEBAR_COURSE_LIMIT) break
  }

  return ids
}

const sidebarCourses = computed(() => {
  return orderedIds().map((id) => {
    const meta = courseMeta.value[id]
    const recent = recentCourses.value.find((c) => c.id === id)
    const name =
      meta?.name ||
      recent?.name ||
      (id === activeCourseId.value ? activeCourseName.value : '') ||
      t('lingo.courses')
    return {
      id,
      name,
      description: meta?.description || '',
      hasCoverImage: !!meta?.hasCoverImage,
    }
  })
})

async function enrichCourse(id: number) {
  try {
    const response = await getCollection(id)
    const data = response.data || {}
    courseMeta.value = {
      ...courseMeta.value,
      [id]: {
        id,
        name: String(data.name || '').trim() || t('lingo.courses'),
        description: String(data.description || '').trim(),
        hasCoverImage: !!data.has_cover_image,
        enriched: true,
      },
    }
  } catch (e) {
    console.error(e)
    const recent = recentCourses.value.find((c) => c.id === id)
    courseMeta.value = {
      ...courseMeta.value,
      [id]: {
        id,
        name: courseMeta.value[id]?.name || recent?.name || t('lingo.courses'),
        description: courseMeta.value[id]?.description || '',
        hasCoverImage: courseMeta.value[id]?.hasCoverImage || false,
        enriched: true,
      },
    }
  }
}

async function enrichMissing(ids: number[]) {
  const needFetch = ids.filter((id) => !courseMeta.value[id]?.enriched)
  if (needFetch.length === 0) return
  await Promise.all(needFetch.map((id) => enrichCourse(id)))
}

async function loadPopularCourses() {
  if (coursesLoaded.value) {
    await enrichMissing(orderedIds())
    return
  }
  isLoading.value = true
  try {
    const response = await searchCollectionsAndUsers({ per_kind: POPULAR_FETCH_PER_KIND })
    const items = (response.data.items ?? []) as Array<{
      kind: string
      collection_id?: number
      name?: string
    }>
    popularIds.value = items
      .filter((item) => item.kind === 'collection' && typeof item.collection_id === 'number')
      .map((item) => item.collection_id as number)

    const seeded: Record<number, SidebarCourse> = { ...courseMeta.value }
    for (const item of items) {
      if (item.kind !== 'collection' || typeof item.collection_id !== 'number') continue
      if (seeded[item.collection_id]) continue
      seeded[item.collection_id] = {
        id: item.collection_id,
        name: item.name?.trim() || t('lingo.courses'),
        description: '',
        hasCoverImage: false,
      }
    }
    courseMeta.value = seeded
    coursesLoaded.value = true
    await enrichMissing(orderedIds())
  } catch (e) {
    console.error(e)
    popularIds.value = []
  } finally {
    isLoading.value = false
  }
}

watch(
  showCourseList,
  (show) => {
    if (show) loadPopularCourses()
  },
  { immediate: true }
)

watch(
  () => (showCourseList.value ? orderedIds().join(',') : ''),
  (key) => {
    if (!key) return
    enrichMissing(orderedIds())
  }
)
</script>
