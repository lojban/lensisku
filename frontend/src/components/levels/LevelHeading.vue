<template>
  <div
    :id="anchor ? (level ? `level-${level.level_id}` : 'level-unassigned') : undefined"
    class="level-heading flex flex-col rounded-md border"
    :class="[
      highlighted ? 'level-heading--highlight' : '',
      level ? 'border-indigo-200 bg-indigo-50' : 'border-gray-200 bg-gray-50',
    ]"
  >
    <div class="level-heading__row flex items-center gap-2 px-3 py-2">
      <div class="flex min-w-0 flex-1 items-center gap-2">
        <span v-if="level" class="shrink-0 text-xs font-semibold uppercase text-indigo-700">
          {{ t('levelHeading.levelN', { n: index + 1 }) }}
        </span>
        <form
          v-if="level && isRenaming"
          class="flex min-w-0 flex-1 items-center gap-2"
          @submit.prevent="commitRename"
        >
          <Input
            ref="renameInput"
            v-model="draftName"
            type="text"
            required
            class="input-field h-8 min-w-0 flex-1"
            :aria-label="t('flashcardLevels.nameLabel')"
            @keydown.esc.prevent="isRenaming = false"
          />
          <Button variant="save" type="submit">{{ t('levelHeading.save') }}</Button>
          <Button variant="cancel" type="button" @click="isRenaming = false">
            {{ t('flashcardLevels.cancelButton') }}
          </Button>
        </form>
        <div v-else class="flex min-w-0 flex-1 flex-col">
          <h3 class="min-w-0 break-words text-sm font-semibold text-gray-800">
            {{ level ? level.name : t('levelHeading.unassigned') }}
            <span v-if="continued" class="font-normal text-gray-500">{{
              t('levelHeading.continued')
            }}</span>
          </h3>
        </div>
        <span
          v-if="level && level.prerequisites?.length && !isRenaming"
          class="inline-flex shrink-0 text-gray-500"
          role="img"
          :title="t('levelHeading.requires', { names: prereqNames })"
          :aria-label="t('levelHeading.requires', { names: prereqNames })"
        >
          <Lock class="h-3.5 w-3.5" />
        </span>
      </div>
      <div v-if="level && isOwner && !isRenaming" class="flex shrink-0 items-center gap-1">
        <IconButton
          v-if="showReorder"
          button-classes="ui-btn--toolbar"
          :aria-label="t('levelHeading.moveLevelUp')"
          :disabled="index === 0"
          @click="emit('move-up')"
        >
          <template #icon><ArrowUp class="h-4 w-4" /></template>
        </IconButton>
        <IconButton
          v-if="showReorder"
          button-classes="ui-btn--toolbar"
          :aria-label="t('levelHeading.moveLevelDown')"
          :disabled="isLast"
          @click="emit('move-down')"
        >
          <template #icon><ArrowDown class="h-4 w-4" /></template>
        </IconButton>
        <IconButton
          button-classes="ui-btn--toolbar"
          :aria-label="t('levelHeading.rename')"
          @click="startRename"
        >
          <template #icon><Pencil class="h-4 w-4" /></template>
        </IconButton>
        <IconButton
          button-classes="ui-btn--toolbar"
          :aria-label="t('levelHeading.settings')"
          @click="emit('edit')"
        >
          <template #icon><Settings class="h-4 w-4" /></template>
        </IconButton>
      </div>
    </div>
    <!-- Prerequisites: own block under the heading row (same in Learn and Edit mode: no CLS). -->
    <div
      v-if="level && prereqs.length && !continued"
      class="level-heading__requires border-t border-indigo-100 bg-white/60 px-3 py-2"
    >
      <p class="mb-1 text-xs font-medium text-gray-500">{{ t('levelHeading.requiresLabel') }}</p>
      <ul class="flex flex-wrap gap-1.5" role="list">
        <li
          v-for="p in prereqs"
          :key="p.level_id"
          class="inline-flex max-w-full items-center gap-1 rounded-full px-2 py-0.5 text-xs ring-1"
          :class="
            p.is_completed
              ? 'bg-green-50 text-green-800 ring-green-200'
              : 'bg-white text-gray-700 ring-gray-200'
          "
        >
          <CheckCircle2 v-if="p.is_completed" class="h-3 w-3 shrink-0" aria-hidden="true" />
          <Lock v-else class="h-3 w-3 shrink-0 text-gray-400" aria-hidden="true" />
          <span class="min-w-0 break-words">{{ p.name }}</span>
        </li>
      </ul>
    </div>
    <slot />
  </div>
</template>

<script setup lang="ts">
import { ArrowDown, ArrowUp, CheckCircle2, Lock, Pencil, Settings } from '@lucide/vue'
import { computed, nextTick, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import type { CourseLevel } from '@/composables/useCourseLevels'
import { Button, IconButton, Input } from '@packages/ui'

/** One level heading row (or the "Unassigned" pseudo-heading when `level` is null). */
const props = withDefaults(
  defineProps<{
    level: CourseLevel | null
    index?: number
    count: number
    isOwner?: boolean
    highlighted?: boolean
    isLast?: boolean
    showReorder?: boolean
    /** Repeated heading at the top of a page that continues this level. */
    continued?: boolean
    /** Render the `level-<id>` anchor id (only on the first heading of a level). */
    anchor?: boolean
  }>(),
  {
    index: 0,
    isOwner: false,
    highlighted: false,
    isLast: false,
    showReorder: false,
    continued: false,
    anchor: true,
  }
)

const emit = defineEmits<{
  rename: [name: string]
  edit: []
  'move-up': []
  'move-down': []
}>()

const { t } = useI18n()
const isRenaming = ref(false)
const draftName = ref('')
const renameInput = ref<{ $el?: HTMLElement } | null>(null)

const prereqs = computed(() => props.level?.prerequisites ?? [])
const prereqNames = computed(() => (props.level?.prerequisites ?? []).map((p) => p.name).join(', '))

async function startRename() {
  draftName.value = props.level?.name ?? ''
  isRenaming.value = true
  await nextTick()
  const el = renameInput.value?.$el
  const input = el instanceof HTMLInputElement ? el : el?.querySelector?.('input')
  input?.focus()
}

function commitRename() {
  const name = draftName.value.trim()
  isRenaming.value = false
  if (name && name !== props.level?.name) emit('rename', name)
}
</script>

<style scoped>
.level-heading {
  scroll-margin-top: 5rem;
  transition: box-shadow 0.3s;
}

.level-heading__row {
  min-height: 2.75rem;
}

.level-heading--highlight {
  box-shadow: 0 0 0 3px rgb(99 102 241 / 0.6);
}
</style>
