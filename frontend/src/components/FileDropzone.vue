<template>
  <div
    ref="dropZoneRef"
    class="rounded-lg transition-colors"
    :class="[
      disabled
        ? 'border-2 border-dashed border-gray-200 bg-gray-50'
        : isOverDropZone
          ? 'border-2 border-dashed border-blue-400 bg-blue-50'
          : 'border-2 border-dashed border-gray-300 bg-gray-50/80',
    ]"
    role="region"
    :aria-label="dropzoneAriaLabel"
  >
    <label
      class="flex min-h-[9rem] cursor-pointer flex-col items-center justify-center gap-2 px-4 py-6 text-center"
      :class="disabled ? 'pointer-events-none cursor-not-allowed opacity-50' : ''"
    >
      <input
        type="file"
        class="sr-only"
        :accept="accept"
        :disabled="disabled"
        :aria-label="inputAriaLabel"
        @change="onNativeFileChange"
        @click.stop
      />
      <Upload class="h-10 w-10 text-gray-300" :stroke-width="1" aria-hidden="true" />
      <div class="flex flex-wrap items-center justify-center gap-x-1 text-sm text-gray-600">
        <span class="font-medium text-blue-600">{{ chooseFileText }}</span>
        <span>{{ orDragDropText }}</span>
      </div>

      <p v-if="typesNoteText" class="text-xs text-gray-500">{{ typesNoteText }}</p>
    </label>
  </div>
</template>

<script setup lang="ts">
import { Upload } from 'lucide-vue-next'
import { useDropZone } from '@vueuse/core'
import { ref } from 'vue'

const props = withDefaults(
  defineProps<{
    /** `accept` attribute for the file input (e.g. `.csv,.tsv,text/csv`). */
    accept: string
    chooseFileText: string
    orDragDropText: string
    dropzoneAriaLabel: string
    inputAriaLabel: string
    typesNoteText?: string
    disabled?: boolean
    /** When set, file is only emitted if this returns true; otherwise `reject` is emitted. */
    validateFile?: (file: File) => boolean
  }>(),
  {
    typesNoteText: '',
    disabled: false,
    validateFile: undefined,
  }
)

const emit = defineEmits<{
  select: [file: File]
  reject: []
}>()

const dropZoneRef = ref<HTMLElement | null>(null)

function forwardFile(file: File | undefined) {
  if (props.disabled || !file) return
  if (props.validateFile && !props.validateFile(file)) {
    emit('reject')
    return
  }
  emit('select', file)
}

const { isOverDropZone } = useDropZone(dropZoneRef, (files) => {
  forwardFile(files?.[0])
})

function onNativeFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  forwardFile(file)
  input.value = ''
}
</script>
