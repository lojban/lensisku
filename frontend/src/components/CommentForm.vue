<template>
  <div
    :class="[
      'surface-comment-form comment-item',
      disableBorder ? 'surface-comment-form--borderless' : null,
    ]"
  >
    <div v-if="optionalTitle && !showSubjectField" class="mb-2 flex items-center text-sm">
      <button
        type="button"
        class="text-blue-700 hover:underline"
        @click="revealTitle"
      >
        {{ t('creation.addTitle') }}
      </button>
    </div>
    <div class="border-b border-gray-100 last:border-0">
      <form :id="formId" @submit.prevent="handleSubmit">
        <div v-if="showSubjectField || (!isReply && !optionalTitle)" class="mb-2">
          <div class="flex justify-between items-center">
            <Input
              ref="subjectInputRef"
              v-model="form.subject"
              type="text"
              :placeholder="t('components.commentForm.subjectPlaceholder')"
              class="input-field w-full text-lg bg-transparent placeholder-gray-500 focus:outline-none"
            />
            <Button
              v-if="isReply"
              variant="neutral"
              type="button"
              class="ml-2 text-sm text-gray-500 hover:text-gray-700 focus:outline-none"
              @click="showSubjectField = false"
            >
              {{ t('components.commentForm.hideSubject') }}
            </Button>
          </div>
        </div>

        <div ref="editor" class="milkdown-editor z-index-1" />

        <div v-if="!hideSubmit" class="flex items-center justify-end mt-1">
          <div class="flex items-center space-x-3">
            <!-- add subject control removed in favor of subject Input field -->
            <Button
              variant="insert"
              type="submit"
              :disabled="isSubmitting || characterCount > 10280 || (requireBody && !hasBody)"
              class="inline-flex items-center ui-btn--insert text-sm"
            >
              <div class="flex items-center">
                <Loader v-if="isSubmitting" class="animate-spin -ml-1 mr-2 h-4 w-4" />
                {{ submitButtonText }}
              </div>
            </Button>
          </div>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button, Input } from '@packages/ui'
import { Crepe } from '@milkdown/crepe'
import { editorViewCtx } from '@milkdown/core'
import { Loader } from '@lucide/vue'
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { insert } from '@milkdown/utils'
import '@milkdown/crepe/theme/common/style.css'
import '@milkdown/crepe/theme/frame.css'
import { useI18n } from 'vue-i18n'

import { useError } from '@/composables/useError'

const { t } = useI18n()
const { showError } = useError()

// Payload size limit (5MB)
const MAX_PAYLOAD_SIZE = 5 * 1024 * 1024

const editor = ref(null)
let crepe = null
/** Cleanup for Space-exits-link keydown listener (remove on unmount). */
let spaceExitLinkCleanup = null

onMounted(async () => {
  crepe = new Crepe({
    root: editor.value,
    defaultValue: props.initialValues.content,
    featureConfigs: {
      [Crepe.Feature.Placeholder]: {
        text: 'Type / to show menu',
      },
      [Crepe.Feature.ImageBlock]: {
        onUpload: async (file: File) => {
          // Convert file to base64
          const reader = new FileReader()
          reader.readAsDataURL(file)
          const dataUrl = await new Promise<string>((resolve, reject) => {
            reader.onload = () => resolve(reader.result as string)
            reader.onerror = () => reject(reader.error)
          })
          return dataUrl
        },
      },
    },
  })

  await crepe.create()

  // When cursor is inside a link, Space or Tab should end the link and insert the character (not extend the anchor).
  crepe.editor.action((ctx) => {
    const view = ctx.get(editorViewCtx)
    if (!view?.dom) return
    const linkMarkType = view.state.schema.marks.link
    if (!linkMarkType) return
    const handler = (e) => {
      const key = e.key === ' ' ? ' ' : e.key === 'Tab' ? '\t' : null
      if (key === null) return
      const { state } = view
      const { $from, empty } = state.selection
      const inLink = empty && $from.marks().some((m) => m.type === linkMarkType)
      if (!inLink) return
      e.preventDefault()
      e.stopPropagation()
      const tr = state.tr.removeStoredMark(linkMarkType).insertText(key, state.selection.from)
      view.dispatch(tr)
    }
    view.dom.addEventListener('keydown', handler, true)
    spaceExitLinkCleanup = () => {
      view.dom.removeEventListener('keydown', handler, true)
      spaceExitLinkCleanup = null
    }
  })

  const updateFormContent = () => {
    if (!crepe) return
    markdown.value = crepe.getMarkdown()
  }

  // Update content ref on change
  crepe.on((listener) => {
    listener.markdownUpdated(updateFormContent)
  })
})

onUnmounted(() => {
  if (typeof spaceExitLinkCleanup === 'function') {
    spaceExitLinkCleanup()
  }
  if (crepe) {
    crepe.destroy()
  }
})

const props = defineProps({
  formId: { type: String, default: undefined },
  hideSubmit: { type: Boolean, default: false },
  optionalTitle: { type: Boolean, default: false },
  requireBody: { type: Boolean, default: false },
  submitLabel: { type: String, default: '' },
  isSubmitting: {
    type: Boolean,
    default: false,
  },
  isReply: {
    type: Boolean,
    default: false,
  },
  /** Drop outer border/shadow (e.g. when nested in a sidebar chrome). */
  disableBorder: {
    type: Boolean,
    default: false,
  },
  initialValues: {
    type: Object,
    default: () => ({
      subject: '',
      content: '',
    }),
  },
})

const emit = defineEmits(['submit', 'cancel', 'draft-change'])

const textareaRef = ref(null)
const subjectInputRef = ref(null)
const showSubjectField = ref(
  (!props.isReply && !props.optionalTitle) || !!props.initialValues.subject
)
const form = ref({
  subject: props.initialValues.subject,
  content: props.initialValues.content,
})

const markdown = ref(
  typeof props.initialValues.content === 'string' ? props.initialValues.content : ''
)
const hasBody = computed(() => markdown.value.trim().length > 0)
const characterCount = computed(() => markdown.value.length)
watch([markdown, () => form.value.subject], () =>
  emit('draft-change', { subject: form.value.subject, markdown: markdown.value })
)

const submitButtonText = computed(() =>
  props.isSubmitting
    ? t('components.commentForm.posting')
    : props.submitLabel || t('components.commentForm.sendButton')
)

const autoResize = async () => {
  await nextTick()
  const textarea = textareaRef.value
  if (textarea) {
    const lineHeight = parseInt(getComputedStyle(textarea).lineHeight)
    const maxHeight = lineHeight * 10 // 10 lines max

    textarea.style.height = 'auto'
    const contentHeight = textarea.scrollHeight

    textarea.style.height = `${Math.min(contentHeight, maxHeight)}px`
  }
}

watch(
  () => props.initialValues,
  (newValues) => {
    form.value = {
      subject: newValues.subject || '',
      content: newValues.content || '',
    }

    if (crepe && newValues.content) {
      crepe.editor.action(insert('\n\n' + newValues.content))
    }
  },
  { deep: true }
)

watch(() => form.value.content, autoResize)

const handleSubmit = () => {
  if (props.isSubmitting) return
  const draftMarkdown = crepe ? crepe.getMarkdown() : markdown.value
  markdown.value = draftMarkdown
  if (draftMarkdown.length > 10280 || (props.requireBody && !draftMarkdown.trim())) return
  const normalized = draftMarkdown
    .replace(/(https?:)(\\_){2}/g, '$1//')
    .replace(/(https?)__(?=[^\s\]])/g, '$1://')
    .replace(/<(https?:\/\/[^\s>]+)>/g, '[$1]($1)')
  const content = normalized
    .split(/(^>.*$)/gm)
    .filter((line) => line.trim())
    .map((line) => {
      line = line.trim()
      return line.startsWith('# ')
        ? { type: 'header', data: line.substring(2).trim() }
        : { type: 'text', data: line }
    })
  if (!content.length && !form.value.subject.trim()) return
  const encoder = new TextEncoder()
  if (
    encoder.encode(form.value.subject).length + encoder.encode(JSON.stringify(content)).length >
    MAX_PAYLOAD_SIZE
  ) {
    showError('components.commentForm.errorTooLarge')
    return
  }
  emit('submit', { subject: form.value.subject.trim(), content })
}

async function revealTitle() {
  showSubjectField.value = true
  await nextTick()
  focusSubject()
}

const focusSubject = () => {
  subjectInputRef.value?.focus()
}

defineExpose({
  getDraft: () => ({
    subject: form.value.subject,
    markdown: crepe ? crepe.getMarkdown() : markdown.value,
  }),
  focusSubject,
})
</script>

<style>
milkdown-slash-menu {
  z-index: 100;
}

.milkdown .ProseMirror {
  @apply py-2 px-0 md:pl-20;
}

milkdown-slash-menu {
  position: fixed !important;
  top: 50% !important;
  left: 50% !important;
  transform: translate(-50%, -50%) !important;
  z-index: 100;
  width: auto !important;
  max-width: 80vw !important;
  /* Prevent it from overflowing horizontally */
  overflow-y: auto !important;
  /* Allow vertical scrolling if needed */
  max-height: none !important;
  /* Remove any max-height limitations */
}

/* Hide block handle on mobile */
@media (max-width: 640px) {
  milkdown-block-handle {
    display: none !important;
  }
}
</style>
