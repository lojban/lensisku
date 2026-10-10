<template>
  <dialog
    ref="dialog"
    class="rounded-xl p-5 max-w-sm w-[calc(100%-2rem)] border shadow-xl backdrop:bg-black/40"
    aria-labelledby="unsaved-title"
    aria-describedby="unsaved-description"
    @cancel.prevent="resolveUnsavedPrompt(false)"
  >
    <h2 id="unsaved-title" class="text-lg font-semibold text-gray-900">
      {{ t('creation.unsavedTitle') }}
    </h2>
    <p id="unsaved-description" class="mt-3 text-sm text-gray-700">{{ t('creation.unsaved') }}</p>
    <div class="flex justify-end gap-3 mt-5">
      <Button ref="stayButton" variant="neutral" @click="resolveUnsavedPrompt(false)">{{
        t('creation.stay')
      }}</Button>
      <Button variant="cancel" @click="resolveUnsavedPrompt(true)">{{
        t('creation.leave')
      }}</Button>
    </div>
  </dialog>
</template>
<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button } from '@packages/ui'
import { unsavedPromptOpen, resolveUnsavedPrompt } from '@/composables/useUnsavedChanges'
const { t } = useI18n()
const dialog = ref<HTMLDialogElement | null>(null)
const stayButton = ref<{ $el: HTMLButtonElement } | null>(null)
watch(unsavedPromptOpen, async (open) => {
  await nextTick()
  if (open && !dialog.value?.open) {
    dialog.value?.showModal()
    stayButton.value?.$el.focus()
  } else if (!open) dialog.value?.close()
})
</script>
