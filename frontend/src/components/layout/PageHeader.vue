<script setup lang="ts">
import { Comment, Fragment, computed, useAttrs, useSlots } from 'vue'
import type { VNode } from 'vue'

/** `useSlots()` truthiness is wrong for `[]`; detect real output for optional rows. */
function slotHasContent(render?: () => VNode[]): boolean {
  if (!render) return false
  const list = render()
  if (!Array.isArray(list) || list.length === 0) return false
  return list.some((vn) => vnodeIsVisible(vn))
}

function vnodeIsVisible(vn: VNode): boolean {
  if (vn.type === Comment) return false
  if (vn.type === Fragment) {
    const ch = vn.children
    return Array.isArray(ch) && ch.some((c) => vnodeIsVisible(c as VNode))
  }
  if (typeof vn.type === 'symbol') {
    return String(vn.children ?? '').trim().length > 0
  }
  return true
}

/**
 * Card-style page header: bordered white panel, responsive padding (`mb-6` when `margin="section"`).
 *
 * **Layouts**
 * - `stack` (default): title row (`icon` + heading + `title-after`, optional `trailing` on the same row).
 *   Optional `leading` (e.g. collection cover) opens a two-column row; `eyebrow` / `description` sit in the text column.
 * - `split`: one row — left block is `leading` + text column; right block is `trailing` (e.g. study session actions).
 *
 * **Slots:** `leading`, `eyebrow`, `icon`, `title`, `title-after`, `trailing`, `description`, `meta`, `meta-trailing`, `toolbar`.
 */
defineOptions({ inheritAttrs: false })

const props = withDefaults(
  defineProps<{
    titleAs?: 'h1' | 'h2' | 'h3'
    /** e.g. SEO: `{ itemprop: 'name' }` */
    titleAttrs?: Record<string, string | number | boolean | undefined | null>
    margin?: 'none' | 'section'
    layout?: 'stack' | 'split'
    /** Vertical gap between major blocks (meta, toolbar, leading block). */
    stackGap?: 'compact' | 'comfortable'
    /** `primary`: entry-style title; `secondary`: collection-style slightly smaller heading. */
    titleTone?: 'primary' | 'secondary'
  }>(),
  {
    titleAs: 'h1',
    titleAttrs: () => ({}),
    margin: 'none',
    layout: 'stack',
    stackGap: 'compact',
    titleTone: 'primary',
  }
)

const attrs = useAttrs()
const slots = useSlots()

const hasLeading = computed(() => slotHasContent(slots.leading))
const hasEyebrow = computed(() => slotHasContent(slots.eyebrow))
const hasMeta = computed(() => slotHasContent(slots.meta))
const hasMetaTrailing = computed(() => slotHasContent(slots['meta-trailing']))
const hasTrailing = computed(() => slotHasContent(slots.trailing))
const hasIcon = computed(() => slotHasContent(slots.icon))
const hasDescription = computed(() => slotHasContent(slots.description))
const hasToolbar = computed(() => slotHasContent(slots.toolbar))

const headerClass = computed(() => {
  /** Shared shell: mobile bleed + square corners (see `tailwind.config.js` `.page-header-shell`). */
  const shell = 'page-header-shell'
  const margin = props.margin === 'section' ? 'mb-6' : ''
  const extra = attrs.class
  return [shell, margin, extra].filter(Boolean)
})

const passthroughAttrs = computed(() => {
  const { class: _c, ...rest } = attrs as Record<string, unknown>
  return rest
})

const stackGapClass = computed(() => (props.stackGap === 'comfortable' ? 'gap-4' : 'gap-2'))

const titleRowClass = computed(() =>
  hasTrailing.value
    ? 'flex min-w-0 flex-1 flex-wrap items-center gap-x-2 gap-y-1'
    : 'flex w-fit max-w-full min-w-0 flex-wrap items-center gap-x-2 gap-y-1'
)

const titleShellClass = computed(() =>
  hasTrailing.value
    ? 'flex w-full min-w-0 items-start justify-between gap-2'
    : 'flex w-full min-w-0 flex-col gap-2 sm:flex-row sm:items-center sm:gap-3 sm:gap-4'
)

const titleClass = computed(() => {
  const base =
    'my-0 inline-flex min-w-0 max-w-full items-center break-words font-bold text-gray-800'
  const tone = props.titleTone === 'secondary' ? 'text-xl sm:text-2xl' : 'text-2xl sm:text-3xl'
  return `${base} ${tone}`
})

const metaRowJustifyClass = computed(() => {
  if (hasMeta.value && hasMetaTrailing.value) return 'justify-between'
  if (hasMetaTrailing.value && !hasMeta.value) return 'justify-end'
  return ''
})
</script>

<template>
  <header :class="headerClass" v-bind="passthroughAttrs">
    <!-- Split: leading + title block | trailing -->
    <template v-if="layout === 'split'">
      <div class="flex w-full min-w-0 flex-col" :class="stackGapClass">
        <div class="flex w-full min-w-0 flex-wrap items-center justify-between gap-4">
          <div
            class="flex min-w-0 flex-1 flex-col gap-3 sm:flex-row sm:items-center sm:gap-3 md:gap-4"
          >
            <div v-if="hasLeading" class="mx-auto shrink-0 sm:mx-0"><slot name="leading" /></div>

            <div class="min-w-0 flex-1 text-center sm:text-left">
              <div v-if="hasEyebrow" class="min-w-0"><slot name="eyebrow" /></div>
              <component :is="titleAs" v-bind="titleAttrs" :class="titleClass">
                <slot name="title" />
              </component>
              <div v-if="hasDescription" class="min-w-0"><slot name="description" /></div>
            </div>
          </div>

          <div
            v-if="hasTrailing"
            class="flex w-full shrink-0 flex-wrap items-center justify-center gap-2 sm:w-auto sm:justify-end"
          >
            <slot name="trailing" />
          </div>
        </div>

        <div
          v-if="hasMeta || hasMetaTrailing"
          class="flex w-full min-w-0 flex-wrap items-center gap-x-2 gap-y-2"
          :class="metaRowJustifyClass"
        >
          <div v-if="hasMeta" class="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
            <slot name="meta" />
          </div>

          <div
            v-if="hasMetaTrailing"
            class="flex shrink-0 flex-wrap items-center justify-end gap-2"
          >
            <slot name="meta-trailing" />
          </div>
        </div>

        <div v-if="hasToolbar" class="w-full min-w-0"><slot name="toolbar" /></div>
      </div>
    </template>
    <!-- Stack (default) -->
    <template v-else>
      <div class="flex w-full min-w-0 flex-col" :class="stackGapClass">
        <template v-if="hasLeading">
          <div class="flex flex-row items-stretch gap-3 sm:gap-4">
            <div class="shrink-0"><slot name="leading" /></div>

            <div class="flex min-w-0 flex-1 flex-col gap-2 text-left">
              <div v-if="hasEyebrow" class="min-w-0"><slot name="eyebrow" /></div>

              <div
                class="flex w-full min-w-0 flex-col gap-2 sm:flex-row sm:items-center sm:gap-3 sm:gap-4"
              >
                <div :class="titleRowClass">
                  <span v-if="hasIcon" class="inline-flex shrink-0 items-center text-gray-500">
                    <slot name="icon" />
                  </span>
                  <component :is="titleAs" v-bind="titleAttrs" :class="titleClass">
                    <slot name="title" />
                  </component>
                  <span class="inline-flex shrink-0 items-center empty:hidden">
                    <slot name="title-after" />
                  </span>
                </div>

                <div
                  v-if="hasTrailing"
                  class="flex shrink-0 flex-wrap items-center justify-end gap-2"
                >
                  <slot name="trailing" />
                </div>
              </div>

              <div v-if="hasDescription" class="min-w-0"><slot name="description" /></div>
            </div>
          </div>
        </template>
        <template v-else>
          <div class="flex w-full min-w-0 flex-col gap-2 text-left">
            <div v-if="hasEyebrow" class="min-w-0"><slot name="eyebrow" /></div>

            <div :class="titleShellClass">
              <div :class="titleRowClass">
                <span v-if="hasIcon" class="inline-flex shrink-0 items-center text-gray-500">
                  <slot name="icon" />
                </span>
                <component :is="titleAs" v-bind="titleAttrs" :class="titleClass">
                  <slot name="title" />
                </component>
                <span class="inline-flex shrink-0 items-center empty:hidden">
                  <slot name="title-after" />
                </span>
              </div>

              <div
                v-if="hasTrailing"
                class="flex shrink-0 flex-wrap items-center justify-end gap-2"
              >
                <slot name="trailing" />
              </div>
            </div>

            <div v-if="hasDescription" class="min-w-0"><slot name="description" /></div>
          </div>
        </template>
        <div
          v-if="hasMeta || hasMetaTrailing"
          class="flex w-full min-w-0 flex-wrap items-center gap-x-2 gap-y-2"
          :class="metaRowJustifyClass"
        >
          <div v-if="hasMeta" class="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
            <slot name="meta" />
          </div>

          <div
            v-if="hasMetaTrailing"
            class="flex shrink-0 flex-wrap items-center justify-end gap-2"
          >
            <slot name="meta-trailing" />
          </div>
        </div>

        <div v-if="hasToolbar" class="w-full min-w-0"><slot name="toolbar" /></div>
      </div>
    </template>
  </header>
</template>
