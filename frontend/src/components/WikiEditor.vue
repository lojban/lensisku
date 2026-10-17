<template>
  <div class="wiki-editor flex flex-col space-y-2">
    <div class="flex shrink-0 flex-wrap items-center gap-2">
      <Button
        type="button"
        variant="empty"
        class="text-sm"
        :disabled="disabled"
        @click="insertWikiLink"
      >
        {{ t('upsertWiki.insertWikiLink') }}
      </Button>
      <Button
        type="button"
        variant="empty"
        class="text-sm"
        :disabled="disabled"
        @click="toggleMode"
      >
        {{ rawMode ? t('upsertWiki.visualEditor') : t('upsertWiki.rawSource') }}
      </Button>
    </div>
    <textarea
      v-if="rawMode"
      v-model="rawValue"
      class="wiki-editor-surface wiki-editor-min-h w-full rounded border border-gray-300 p-3 font-mono text-sm max-sm:rounded-none max-sm:border-x-0"
      :disabled="disabled"
      :placeholder="placeholder"
      @input="syncFromRaw"
    />
    <div
      v-else
      ref="editorRoot"
      class="milkdown-editor wiki-editor-surface"
      :class="{ 'opacity-60 pointer-events-none': disabled }"
    />
  </div>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { Crepe } from '@milkdown/crepe'
import { insert } from '@milkdown/utils'
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
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
const rawMode = ref(false)
const rawValue = ref(props.modelValue || '')
let crepe: Crepe | null = null
let mountGeneration = 0

function normalizeMarkdown(markdown: string): string {
  return markdown.replace(/<(https?:\/\/[^\s>]+)>/g, '[$1]($1)')
}

function syncFromEditor() {
  if (!crepe) return
  const markdown = normalizeMarkdown(crepe.getMarkdown())
  rawValue.value = markdown
  emit('update:modelValue', markdown)
}

function syncFromRaw() {
  const markdown = normalizeMarkdown(rawValue.value)
  rawValue.value = markdown
  emit('update:modelValue', markdown)
}

function wikiLinkSnippet(title: string): string {
  const page = title.trim()
  const slug = encodeURIComponent(page.replace(/ /g, '_'))
  return ` [${page}](/wiki/${slug}) `
}

function insertWikiLink() {
  const title = window.prompt(t('upsertWiki.wikiLinkPrompt'))
  if (!title?.trim()) return
  const snippet = wikiLinkSnippet(title)
  if (rawMode.value) {
    rawValue.value = `${rawValue.value}${snippet}`
    syncFromRaw()
    return
  }
  if (!crepe) return
  crepe.editor.action(insert(snippet))
  syncFromEditor()
}

async function destroyEditor() {
  mountGeneration += 1
  const instance = crepe
  crepe = null
  if (!instance) return
  await instance.destroy()
}

async function mountEditor() {
  const generation = ++mountGeneration
  await nextTick()
  if (generation !== mountGeneration || rawMode.value || !editorRoot.value) return
  const instance = new Crepe({
    root: editorRoot.value,
    defaultValue: rawValue.value || '',
    featureConfigs: {
      [Crepe.Feature.Placeholder]: {
        text: props.placeholder || t('upsertWiki.editorPlaceholder') || 'Type / to show menu',
      },
    },
  })
  await instance.create()
  if (generation !== mountGeneration || rawMode.value) {
    await instance.destroy()
    return
  }
  crepe = instance
  crepe.on((listener) => {
    listener.markdownUpdated(syncFromEditor)
  })
}

async function toggleMode() {
  if (rawMode.value) {
    syncFromRaw()
    rawMode.value = false
    await mountEditor()
    return
  }
  if (crepe) {
    rawValue.value = normalizeMarkdown(crepe.getMarkdown())
    emit('update:modelValue', rawValue.value)
  }
  await destroyEditor()
  rawMode.value = true
}

onMounted(() => {
  void mountEditor()
})

watch(
  () => props.modelValue,
  (value) => {
    const next = value || ''
    if (next !== rawValue.value) rawValue.value = next
  }
)

onUnmounted(() => {
  void destroyEditor()
})

defineExpose({
  getMarkdown: () => {
    if (crepe && !rawMode.value) return normalizeMarkdown(crepe.getMarkdown())
    return normalizeMarkdown(rawValue.value || props.modelValue)
  },
})
</script>

<style scoped>
.wiki-editor-surface {
  @apply max-sm:-mx-3 max-sm:!w-[calc(100%+1.5rem)];
}

.wiki-editor-min-h {
  min-height: calc(100dvh + 16rem);
}

.milkdown-editor {
  @apply flex w-full flex-col;
}

.milkdown-editor :deep(.milkdown) {
  @apply flex flex-col overflow-visible;
}

.milkdown-editor :deep(.milkdown .ProseMirror) {
  @apply w-full py-3 px-3 sm:py-4 sm:pl-14 sm:pr-4;
  min-height: calc(100dvh + 16rem);
}

@media (max-width: 640px) {
  .milkdown-editor :deep(.milkdown-block-handle),
  .milkdown-editor :deep(milkdown-block-handle) {
    display: none !important;
  }
}
</style>
