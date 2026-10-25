<template>
  <details
    class="levels-panel rounded-lg border border-gray-200 bg-white"
    :open="open"
    @toggle="onToggle"
  >
    <summary
      class="flex min-h-[2.75rem] cursor-pointer select-none items-center gap-2 px-3 text-sm font-medium text-gray-700"
    >
      <LayoutPanelTop class="h-4 w-4 shrink-0" aria-hidden="true" />
      <span class="flex-1">{{ t('levelsPanel.title') }}</span>
      <span class="min-w-[1.5rem] text-right text-xs text-gray-500">{{
        course.isLoaded.value ? levels.length : ''
      }}</span>
    </summary>
    <div class="flex flex-col gap-2 border-t border-gray-100 p-3">
      <p v-if="course.isLoaded.value && !levels.length" class="text-sm text-gray-500">
        {{ t('levelsPanel.empty') }}
      </p>
      <LevelHeading
        v-for="(level, i) in levels"
        :key="level.level_id"
        :level="level"
        :anchor="false"
        :index="i"
        :count="level.card_count"
        :is-owner="isOwner"
        :is-last="i === levels.length - 1"
        :show-reorder="isOwner && levels.length > 1"
        @rename="(name) => renameLevel(level, name)"
        @edit="openEdit(level)"
        @move-up="moveLevel(i, -1)"
        @move-down="moveLevel(i, 1)"
      >
        <RouterLink
          :to="{
            path: `/collections/${collectionId}/flashcards`,
            hash: `#level-${level.level_id}`,
          }"
          class="ui-btn--toolbar inline-flex shrink-0 items-center gap-1 text-xs"
        >
          <ListOrdered class="h-4 w-4" aria-hidden="true" />
          {{
            isOwner
              ? t('flashcardLevels.actions.manageCards')
              : t('flashcardLevels.actions.viewCards')
          }}
        </RouterLink>
      </LevelHeading>
      <div v-if="isOwner" class="flex">
        <Button variant="create" @click="openCreate">
          <template #icon><PlusCircle class="h-4 w-4" /></template>
          {{ t('levelHeading.addLevel') }}
        </Button>
      </div>
    </div>
    <LevelEditModal
      v-if="isOwner"
      :show="showModal"
      :collection-id="collectionId"
      :level="editingLevel"
      :levels="levels"
      @close="showModal = false"
      @saved="onChanged"
      @deleted="onChanged"
    />
  </details>
</template>

<script setup lang="ts">
import { LayoutPanelTop, ListOrdered, PlusCircle } from '@lucide/vue'
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'

import { saveCourseOrder, updateLevel } from '@/api'
import LevelEditModal from '@/components/levels/LevelEditModal.vue'
import LevelHeading from '@/components/levels/LevelHeading.vue'
import { useCourseLevels, type CourseLevel } from '@/composables/useCourseLevels'
import { useError } from '@/composables/useError'
import { getApiErrorMessage } from '@/utils/apiError'
import { Button } from '@packages/ui'

/**
 * Collapsible list of a collection's levels with owner-only create / rename / edit / delete /
 * reorder (shared LevelHeading + LevelEditModal). Collapsed by default so it never shifts layout.
 */
const props = defineProps<{ collectionId: number | string; isOwner: boolean }>()
const emit = defineEmits<{ changed: [] }>()

const { t } = useI18n()
const { showError } = useError()
const course = useCourseLevels(() => props.collectionId)
const levels = course.levels
const open = ref(false)
const showModal = ref(false)
const editingLevel = ref<CourseLevel | null>(null)

function onToggle(e: Event) {
  open.value = (e.target as HTMLDetailsElement).open
  if (open.value && !course.isLoaded.value) void course.load()
}

function openCreate() {
  editingLevel.value = null
  showModal.value = true
}

function openEdit(level: CourseLevel) {
  editingLevel.value = level
  showModal.value = true
}

async function onChanged() {
  showModal.value = false
  await course.load()
  emit('changed')
}

async function renameLevel(level: CourseLevel, name: string) {
  try {
    await updateLevel(level.level_id, { name })
    await onChanged()
  } catch (err) {
    showError(getApiErrorMessage(err) || t('flashcardLevels.saveError'))
  }
}

async function moveLevel(i: number, dir: -1 | 1) {
  const ids = levels.value.map((l) => l.level_id)
  const j = i + dir
  if (j < 0 || j >= ids.length) return
  ;[ids[i], ids[j]] = [ids[j], ids[i]]
  try {
    await saveCourseOrder(props.collectionId, { levels: [], level_order: ids })
    await onChanged()
  } catch (err) {
    showError(getApiErrorMessage(err) || t('courseOrder.saveError'))
  }
}

onMounted(() => void course.load())
</script>
