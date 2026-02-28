<template>
  <div class="mt-3 mb-6 comment-item bg-white border rounded-lg p-3 my-2 hover:border-blue-300 transition-colors relative">
    <ToastFloat :show="showToast" :message="toastMessage" :type="toastType" @close="showToast = false" />
    <div class="border-b border-gray-100 last:border-0">
      <form @submit.prevent="handleSubmit">
        <div v-if="showSubjectField || !isReply" class="mb-2">
          <div class="flex justify-between items-center">
            <input ref="subjectInputRef" v-model="form.subject" type="text" :placeholder="t('components.commentForm.subjectPlaceholder')"
              class="input-field w-full text-lg bg-transparent placeholder-gray-500 focus:outline-none">
            <button v-if="isReply" type="button"
              class="ml-2 text-sm text-gray-500 hover:text-gray-700 focus:outline-none"
              @click="showSubjectField = false">
              {{ t('components.commentForm.hideSubject') }}
            </button>
          </div>
        </div>

        <div ref="editor" class="milkdown-editor z-index-1" />

        <div class="flex items-center justify-end mt-1">
          <div class="flex items-center space-x-3">
            <!-- <button v-if="isReply && !showSubjectField" type="button"
              @click="showSubjectField = true; nextTick(() => subjectInputRef?.focus())"
              class="text-sm text-gray-500 hover:text-gray-700 focus:outline-none inline-flex items-center gap-1 mr-2">
              <Plus class="w-4 h-4" />
              {{ t('components.commentForm.addSubject') }}
            </button> -->

            <button type="submit" :disabled="isSubmitting || characterCount > 10280"
              class="inline-flex items-center btn-insert text-sm">
              <div class="flex items-center">
                <Loader v-if="isSubmitting" class="animate-spin -ml-1 mr-2 h-4 w-4" />
                {{ submitButtonText }}
              </div>
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup>
import { Crepe } from '@milkdown/crepe';
import { Loader } from 'lucide-vue-next';
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { insert } from '@milkdown/utils';
import TurndownService from 'turndown';
import ToastFloat from './ToastFloat.vue'; // Import ToastFloat
import '@milkdown/crepe/theme/common/style.css';
import '@milkdown/crepe/theme/frame.css';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

// Payload size limit (5MB)
const MAX_PAYLOAD_SIZE = 5 * 1024 * 1024;

// Toast notification state
const showToast = ref(false);
const toastMessage = ref('');
const toastType = ref('info');

const editor = ref(null);
let crepe = null;
const turndownService = new TurndownService()
let handlePaste = null

onMounted(async () => {
  crepe = new Crepe({

    root: editor.value,
    defaultValue: props.initialValues.content,
    featureConfigs: {
      [Crepe.Feature.Placeholder]: {
        text: 'Type / to show menu',
      },
      [Crepe.Feature.ImageBlock]: {
        onUpload: async (file) => {
          // Convert file to base64
          const reader = new FileReader()
          reader.readAsDataURL(file)
          const dataUrl = await new Promise((resolve) => {
            reader.onload = () => resolve(reader.result)
          })
          return dataUrl
        },
      },
    },
  })

  await crepe.create()

  const updateFormContent = () => {
    if (!crepe) return;
    let markdown = crepe.getMarkdown()
    
    // Convert problematic autolinks <https://...> to [https://...](https://...) 
    // to prevent backend from stripping them as HTML tags.
    markdown = markdown.replace(/<(https?:\/\/[^\s>]+)>/g, '[$1]($1)')
    
    // Parse markdown into content parts array
    const contentParts = markdown.split(/(^>.*$)/gm).filter(line => line.trim()).map(line => {
      line = line.trim();
      if (line.startsWith('# ')) {
        return { type: 'header', data: line.substring(2).trim() }
      }
      return { type: 'text', data: line }
    })
    form.value.content = contentParts
  }

  // Update content ref on change
  crepe.on((listener) => {
    listener.markdownUpdated(updateFormContent)
  })
})

onUnmounted(() => {
  if (crepe) {
    crepe.destroy()
  }
})

const props = defineProps({
  isSubmitting: {
    type: Boolean,
    default: false,
  },
  isReply: {
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

const emit = defineEmits(['submit', 'cancel'])

const textareaRef = ref(null)
const subjectInputRef = ref(null)
const showSubjectField = ref(!props.isReply)
const form = ref({
  subject: props.initialValues.subject,
  content: props.initialValues.content,
})

const characterCount = computed(() => form.value.content.length);

const submitButtonText = computed(() => (props.isSubmitting ? t('components.commentForm.posting') : t('components.commentForm.sendButton')));

const autoResize = async () => {
  await nextTick();
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
  // Final sync to ensure content is captured even if debounced update hasn't fired
  if (crepe) {
    let markdown = crepe.getMarkdown()
    // Fix link anchor text
    markdown = markdown.replace(/(https?:)(\\_){2}/g, '$1//')
    markdown = markdown.replace(/(https?)__(?=[^\s\]])/g, '$1://')
    
    // Convert problematic autolinks <https://...> to [https://...](https://...)
    // to prevent backend from stripping them as HTML tags.
    markdown = markdown.replace(/<(https?:\/\/[^\s>]+)>/g, '[$1]($1)')
    
    form.value.content = markdown.split(/(^>.*$)/gm).filter(line => line.trim()).map(line => {
      line = line.trim();
      if (line.startsWith('# ')) return { type: 'header', data: line.substring(2).trim() }
      return { type: 'text', data: line }
    })
  }

  if (form.value.content.length > 0 || form.value.subject.length > 0) {
    // Estimate payload size
    const encoder = new TextEncoder();
    const subjectSize = encoder.encode(form.value.subject || '').length;
    const contentString = JSON.stringify(form.value.content || []);
    const contentSize = encoder.encode(contentString).length;
    const totalSize = subjectSize + contentSize;

    // Check against the limit
    if (totalSize > MAX_PAYLOAD_SIZE) {
      toastMessage.value = t('components.commentForm.errorTooLarge');
      toastType.value = 'error';
      showToast.value = true;
      console.error(`Comment payload size (${totalSize} bytes) exceeds limit (${MAX_PAYLOAD_SIZE} bytes).`);
      return; // Prevent submission
    }

    // Proceed with submission if size is okay
    emit('submit', {
      subject: form.value.subject.trim(),
      content: form.value.content == '' ? [] : form.value.content
    })
  }
}

const focusSubject = () => {
  subjectInputRef.value?.focus()
}

defineExpose({
  focusSubject
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
