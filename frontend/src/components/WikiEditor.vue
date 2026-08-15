<template>
  <div class="wiki-editor space-y-2">
    <div class="flex flex-wrap items-center gap-2">
      <Button
        type="button"
        variant="empty"
        class="text-sm"
        :disabled="disabled"
        @click="insertWikiLink"
      >
        {{ t('upsertWiki.insertWikiLink') }}
      </Button>
    </div>
    <div
      ref="editorRoot"
      class="milkdown-editor -mx-3 border-y border-gray-300 sm:-mx-4 sm:border"
      :class="{ 'opacity-60 pointer-events-none': disabled }"
    />
  </div>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { Crepe } from '@milkdown/crepe'
import { insert } from '@milkdown/utils'
import { onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import '@milkdown/crepe/theme/common/style.css'
import '@milkdown/crepe/theme/frame.css'

const props = withDefaults(
  defineProps<{
    modelValue: string
    disabled?: boolean
    placeholder?: string
  }>(),
  {
    disabled: false,
    placeholder: '',
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const { t } = useI18n()
const editorRoot = ref<HTMLElement | null>(null)
let crepe: Crepe | null = null

function normalizeMarkdown(markdown: string): string {
  return markdown.replace(/<(https?:\/\/[^\s>]+)>/g, '[$1]($1)')
}

function syncFromEditor() {
  if (!crepe) return
  emit('update:modelValue', normalizeMarkdown(crepe.getMarkdown()))
}

function insertWikiLink() {
  const title = window.prompt(t('upsertWiki.wikiLinkPrompt'))
  if (!title?.trim() || !crepe) return
  const page = title.trim()
  const slug = page.replace(/ /g, '_')
  const snippet = ` [[${page}]](/wiki/${slug}) `
  crepe.editor.action(insert(snippet))
  syncFromEditor()
}

onMounted(async () => {
  crepe = new Crepe({
    root: editorRoot.value,
    defaultValue: props.modelValue || '',
    featureConfigs: {
      [Crepe.Feature.Placeholder]: {
        text: props.placeholder || t('upsertWiki.editorPlaceholder') || 'Type / to show menu',
      },
    },
  })
  await crepe.create()
  crepe.on((listener) => {
    listener.markdownUpdated(syncFromEditor)
  })
})

onUnmounted(() => {
  if (crepe) {
    crepe.destroy()
    crepe = null
  }
})

defineExpose({
  getMarkdown: () => {
    if (!crepe) return props.modelValue
    return normalizeMarkdown(crepe.getMarkdown())
  },
})
</script>

<style scoped>
.milkdown-editor {
  @apply min-h-60 flex flex-col;
}

.milkdown-editor :deep(.milkdown) {
  @apply flex min-h-full flex-1 flex-col;
}

.milkdown-editor :deep(.milkdown .ProseMirror) {
  @apply min-h-full flex-1 py-3 px-3 sm:py-4 sm:pl-14 sm:pr-4;
}

@media (max-width: 640px) {
  .milkdown-editor :deep(.milkdown-block-handle),
  .milkdown-editor :deep(milkdown-block-handle) {
    display: none !important;
  }
}
</style>
