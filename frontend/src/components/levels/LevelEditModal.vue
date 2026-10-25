<template>
  <ModalComponent :show="show" :title="title" @close="emit('close')">
    <form id="level-edit-form" class="flex flex-col gap-4" @submit.prevent="submit">
      <div class="flex flex-col gap-1">
        <label for="level-name" class="level-form-label">{{
          t('flashcardLevels.nameLabel')
        }}</label>
        <Input
          id="level-name"
          v-model="form.name"
          type="text"
          required
          class="input-field w-full"
        />
      </div>
      <div class="flex flex-col gap-1">
        <label for="level-description" class="level-form-label">{{
          t('flashcardLevels.descriptionLabel')
        }}</label>
        <Textarea
          id="level-description"
          v-model="form.description"
          rows="3"
          class="textarea-field"
        />
      </div>
      <div class="flex flex-col gap-1">
        <label for="level-prerequisites" class="level-form-label">{{
          t('flashcardLevels.prerequisitesLabel')
        }}</label>
        <MultiSelectDropdown
          id="level-prerequisites"
          v-model="selectedPrerequisites"
          :options="prerequisiteOptions"
          :option-value="optionValue"
          :option-label="optionLabel"
          :search-field-keys="['name']"
          :placeholder="t('flashcardLevels.prerequisitesLabel')"
          :search-placeholder="t('filters.searchOptions')"
          :select-all-label="t('filters.selectAll')"
          :deselect-all-label="t('filters.deselectAll')"
          :empty-filter-label="t('filters.noMatches')"
          teleport-panel
          class="w-full"
          trigger-class="input-field h-10"
        />
        <p class="text-xs text-gray-500">{{ t('flashcardLevels.lockedHint') }}</p>
      </div>
      <div class="level-form-row">
        <label for="level-min-cards" class="level-form-label">{{
          t('flashcardLevels.minCardsLabel')
        }}</label>
        <Input
          id="level-min-cards"
          v-model.number="form.min_cards"
          type="number"
          min="1"
          class="input-field level-form-input"
        />
      </div>
      <div class="level-form-row">
        <label for="level-min-rate" class="level-form-label">{{
          t('flashcardLevels.minSuccessRateLabel')
        }}</label>
        <Input
          id="level-min-rate"
          v-model.number="form.min_success_rate"
          type="number"
          min="0"
          max="100"
          class="input-field level-form-input"
        />
      </div>
      <div
        v-if="level"
        class="flex flex-wrap items-center justify-between gap-2 rounded-md border border-gray-200 p-3"
      >
        <p class="min-w-0 flex-1 text-xs text-gray-600">{{ t('flashcardLevels.levelMixHint') }}</p>
        <Button
          variant="toolbar"
          type="button"
          class="shrink-0"
          :disabled="isApplyingMix"
          @click="applyMix"
        >
          {{ t('flashcardLevels.actions.applyChallengeMix') }}
        </Button>
      </div>
    </form>
    <template #footer>
      <div class="flex flex-wrap items-center justify-end gap-3">
        <Button
          v-if="level"
          variant="delete"
          type="button"
          class="mr-auto"
          @click="showDeleteConfirm = true"
        >
          {{ t('flashcardLevels.deleteLevelButton') }}
        </Button>
        <Button variant="cancel" type="button" @click="emit('close')">
          {{ t('flashcardLevels.cancelButton') }}
        </Button>
        <Button
          :variant="level ? 'edit' : 'create'"
          type="submit"
          form="level-edit-form"
          :disabled="isSubmitting"
        >
          {{
            isSubmitting
              ? t('flashcardLevels.savingButton')
              : level
                ? t('flashcardLevels.saveChangesButton')
                : t('flashcardLevels.createLevelButton')
          }}
        </Button>
      </div>
    </template>
  </ModalComponent>
  <DeleteConfirmationModal
    :show="showDeleteConfirm"
    :title="t('flashcardLevels.deleteLevelConfirmTitle')"
    :message="t('flashcardLevels.deleteLevelKeepsCards', { levelName: level?.name ?? '' })"
    :is-deleting="isDeleting"
    @confirm="performDelete"
    @cancel="showDeleteConfirm = false"
  />
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { applyChallengeMixToLevel, createLevel, deleteLevel, updateLevel } from '@/api'
import DeleteConfirmationModal from '@/components/DeleteConfirmation.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import type { CourseLevel } from '@/composables/useCourseLevels'
import { useError } from '@/composables/useError'
import { useSuccessToast } from '@/composables/useSuccessToast'
import { getApiErrorMessage } from '@/utils/apiError'
import { Button, Input, MultiSelectDropdown, Textarea } from '@packages/ui'

/**
 * Create / edit / delete one level. Shared by the levels graph, the course-order view on the
 * collection page and the flashcards page. Deleting a level never deletes cards: its cards
 * simply become unassigned.
 */
const props = defineProps<{
  show: boolean
  collectionId: number | string
  /** Level being edited; null creates a new one. */
  level: CourseLevel | null
  /** All levels of the collection (prerequisite choices). */
  levels: CourseLevel[]
}>()

const emit = defineEmits<{
  close: []
  saved: [levelId: number | null]
  deleted: [levelId: number]
}>()

const { t } = useI18n()
const { showError } = useError()
const { showSuccess } = useSuccessToast()

const emptyForm = () => ({
  name: '',
  description: '',
  min_cards: 5,
  min_success_rate: 80,
  prerequisite_ids: [] as number[],
})
const form = ref(emptyForm())
const isSubmitting = ref(false)
const isApplyingMix = ref(false)
const isDeleting = ref(false)
const showDeleteConfirm = ref(false)

watch(
  () => [props.show, props.level] as const,
  ([show, level]) => {
    if (!show) return
    form.value = level
      ? {
          name: level.name,
          description: level.description ?? '',
          min_cards: level.min_cards,
          min_success_rate: Math.round(level.min_success_rate * 100),
          prerequisite_ids: (level.prerequisites ?? []).map((p) => p.level_id),
        }
      : emptyForm()
  },
  { immediate: true }
)

const title = computed(() =>
  props.level ? t('flashcardLevels.editLevelTitle') : t('flashcardLevels.createLevelTitle')
)

type Option = { level_id: number; name: string }
const prerequisiteOptions = computed<Option[]>(() =>
  props.levels.filter((l) => l.level_id !== props.level?.level_id)
)
const optionValue = (o: unknown) => (o as Option).level_id
const optionLabel = (o: unknown) => (o as Option).name
const selectedPrerequisites = computed<unknown[]>({
  get: () =>
    prerequisiteOptions.value.filter((l) => form.value.prerequisite_ids.includes(l.level_id)),
  set: (sel) => {
    form.value.prerequisite_ids = sel.map((o) => (o as Option).level_id)
  },
})

async function submit() {
  if (isSubmitting.value) return
  isSubmitting.value = true
  try {
    const data = { ...form.value, min_success_rate: form.value.min_success_rate / 100 }
    if (props.level) {
      await updateLevel(props.level.level_id, data)
      emit('saved', props.level.level_id)
    } else {
      const res = await createLevel(props.collectionId, data)
      emit('saved', (res.data as { level_id?: number })?.level_id ?? null)
    }
  } catch (err) {
    showError(getApiErrorMessage(err) || t('flashcardLevels.saveError'))
  } finally {
    isSubmitting.value = false
  }
}

async function applyMix() {
  if (!props.level || isApplyingMix.value) return
  isApplyingMix.value = true
  try {
    await applyChallengeMixToLevel(props.level.level_id)
    showSuccess(t('flashcardLevels.levelMixDone'))
  } catch (err) {
    showError(getApiErrorMessage(err) || t('collectionDetail.applyMixError'))
  } finally {
    isApplyingMix.value = false
  }
}

async function performDelete() {
  if (!props.level) return
  isDeleting.value = true
  try {
    await deleteLevel(props.level.level_id)
    showDeleteConfirm.value = false
    emit('deleted', props.level.level_id)
  } catch (err) {
    showError(getApiErrorMessage(err) || t('flashcardLevels.deleteError'))
  } finally {
    isDeleting.value = false
  }
}
</script>

<style scoped>
.level-form-label {
  font-size: 0.875rem;
  font-weight: 500;
  color: #374151;
}

.level-form-row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.level-form-input {
  width: 100%;
}

@media (min-width: 480px) {
  .level-form-row {
    flex-direction: row;
    align-items: center;
    gap: 1rem;
  }

  .level-form-row .level-form-label {
    flex: 1 1 auto;
    min-width: 0;
  }

  .level-form-input {
    flex: none;
    width: 7rem;
  }
}
</style>
