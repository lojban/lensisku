<template>
  <div class="wiki-editor flex flex-col space-y-2">
    <div v-if="rawMode" class="milkdown">
      <div class="milkdown-top-bar">
        <div class="top-bar-inner">
          <button
            type="button"
            class="top-bar-item active"
            :aria-label="t('components.editorToolbar.visual')"
            :disabled="disabled"
            @pointerdown.prevent="toggleRawMode"
          >
            <span class="milkdown-icon" v-html="sourceToolbarIcon" />
          </button>
        </div>
      </div>
    </div>
    <textarea
      v-if="rawMode"
      v-model="rawText"
      class="wiki-editor-surface wiki-editor-min-h w-full rounded border border-gray-300 p-3 font-mono text-sm max-sm:rounded-none max-sm:border-x-0"
      :disabled="disabled"
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
import { Crepe } from '@milkdown/crepe'
import { editorViewCtx } from '@milkdown/core'
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import '@milkdown/crepe/theme/common/style.css'
import '@milkdown/crepe/theme/frame.css'

import {
  addSourceToolbarItem,
  attachTopBarToggle,
  crepeToolbarFeatures,
  placeHeadingLast,
  resolveEditorHref,
  sourceToolbarIcon,
} from '@/utils/crepeToolbar'

const props = withDefaults(
  defineProps<{
    modelValue: string
    disabled?: boolean
  }>(),
  {
    disabled: false,
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const { t } = useI18n()
const editorRoot = ref<HTMLElement | null>(null)
const rawMode = ref(false)
const rawText = ref(props.modelValue || '')
let crepe: Crepe | null = null
let mountGeneration = 0
let lastMarkdown = props.modelValue || ''
let rewriting = false
let topBarToggleCleanup: (() => void) | null = null

function normalizeMarkdown(markdown: string): string {
  return markdown.replace(/<(https?:\/\/[^\s>]+)>/g, '[$1]($1)')
}

function rewriteLinkHrefs() {
  if (!crepe || rewriting) return
  crepe.editor.action((ctx) => {
    const view = ctx.get(editorViewCtx)
    const linkType = view.state.schema.marks.link
    if (!linkType) return
    let tr = view.state.tr
    let changed = false
    view.state.doc.descendants((node, pos) => {
      if (!node.isText) return
      const mark = node.marks.find((item) => item.type === linkType)
      if (!mark) return
      const href = String(mark.attrs.href || '')
      const next = resolveEditorHref(href)
      if (next === href) return
      const from = pos
      const to = pos + node.nodeSize
      tr = tr
        .removeMark(from, to, mark)
        .addMark(from, to, linkType.create({ ...mark.attrs, href: next }))
      changed = true
    })
    if (!changed) return
    rewriting = true
    view.dispatch(tr)
    rewriting = false
  })
}

function currentMarkdown(): string {
  if (!crepe) return normalizeMarkdown(lastMarkdown)
  rewriteLinkHrefs()
  return normalizeMarkdown(crepe.getMarkdown())
}

function syncFromEditor() {
  if (!crepe || rewriting) return
  const markdown = currentMarkdown()
  if (markdown === lastMarkdown) return
  lastMarkdown = markdown
  emit('update:modelValue', markdown)
}

async function destroyEditor() {
  mountGeneration += 1
  topBarToggleCleanup?.()
  topBarToggleCleanup = null
  const instance = crepe
  crepe = null
  if (!instance) return
  await instance.destroy()
}

function syncFromRaw() {
  lastMarkdown = rawText.value
  emit('update:modelValue', normalizeMarkdown(rawText.value))
}

async function toggleRawMode() {
  if (!rawMode.value) {
    rawText.value = currentMarkdown()
    lastMarkdown = rawText.value
    await destroyEditor()
    rawMode.value = true
    return
  }
  lastMarkdown = rawText.value
  rawMode.value = false
  await mountEditor()
}

async function mountEditor() {
  if (rawMode.value) return
  const generation = ++mountGeneration
  await nextTick()
  if (generation !== mountGeneration || !editorRoot.value) return
  const instance = new Crepe({
    root: editorRoot.value,
    defaultValue: lastMarkdown,
    features: crepeToolbarFeatures,
    featureConfigs: {
      [Crepe.Feature.TopBar]: {
        buildTopBar: (builder) => {
          placeHeadingLast(builder)
          addSourceToolbarItem(builder, () => {
            queueMicrotask(() => {
              void toggleRawMode()
            })
          })
        },
      },
      [Crepe.Feature.LinkTooltip]: {
        inputPlaceholder: t('upsertWiki.linkPlaceholder'),
      },
    },
  })
  await instance.create()
  if (generation !== mountGeneration) {
    await instance.destroy()
    return
  }
  crepe = instance
  if (editorRoot.value) {
    topBarToggleCleanup = attachTopBarToggle(editorRoot.value, {
      expand: t('components.editorToolbar.expand'),
      collapse: t('components.editorToolbar.collapse'),
      raw: t('components.editorToolbar.raw'),
    })
  }
  crepe.on((listener) => {
    listener.markdownUpdated(syncFromEditor)
  })
}

onMounted(() => {
  void mountEditor()
})

watch(
  () => props.modelValue,
  (value) => {
    const next = value || ''
    if (next === lastMarkdown) return
    lastMarkdown = next
    rawText.value = next
    if (rawMode.value) return
    void destroyEditor().then(mountEditor)
  }
)

onUnmounted(() => {
  void destroyEditor()
})

defineExpose({
  getMarkdown: () => (rawMode.value ? normalizeMarkdown(rawText.value) : currentMarkdown()),
})
</script>

<style scoped>
.wiki-editor-min-h {
  min-height: calc(100dvh + 16rem);
}

.wiki-editor-surface {
  @apply w-full;
}

.milkdown-editor {
  @apply flex w-full flex-col;
}

.milkdown-editor :deep(.milkdown) {
  @apply flex flex-col overflow-visible;
}

.milkdown-editor :deep(.milkdown .ProseMirror) {
  @apply w-full py-3 px-3 sm:px-4 sm:py-4;
  min-height: calc(100dvh + 16rem);
}
</style>
