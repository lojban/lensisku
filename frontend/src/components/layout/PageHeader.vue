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
 * Card-style page header (`.page-header-shell` + gap tokens in brandbook §8 / `tailwind.config.js`).
 *
 * **Layouts**
 * - `stack` (default): title + optional `trailing` on `.page-header-title-row`; description next;
 *   `#toolbar` as a major stack block. Optional `leading` uses `.page-header-leading-row`.
 * - `split`: one row — left block is `leading` + text column; right block is `trailing`.
 *
 * **Gaps:** `stackGap="compact"` (default, 0.5rem) or `"comfortable"` (1rem) between major blocks.
 * Do not sprinkle ad-hoc `gap-*` in slot markup — use `.page-header-actions` / `.page-header-description`.
 *
 * **Slots:** `leading`, `eyebrow`, `icon`, `title`, `title-after`, `trailing`, `description`, `meta`, `meta-trailing`, `toolbar`.
 */
defineOptions({ inheritAttrs: false })

withDefaults(
  defineProps<{
    titleAs?: 'h1' | 'h2' | 'h3'
    /** e.g. SEO: `{ itemprop: 'name' }` */
    titleAttrs?: Record<string, string | number | boolean | undefined | null>
    margin?: 'none' | 'section'
    layout?: 'stack' | 'split'
    /** Vertical gap between major blocks (body, meta, toolbar). See brandbook §8. */
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

const passthroughAttrs = computed(() => {
  const { class: _c, ...rest } = attrs as Record<string, unknown>
  return rest
})
</script>

<template>
  <header
    class="page-header-shell"
    :class="attrs.class"
    :data-margin="margin === 'section' ? 'section' : undefined"
    v-bind="passthroughAttrs"
  >
    <!-- Split: leading + title block | trailing -->
    <template v-if="layout === 'split'">
      <div class="page-header-stack" :data-stack-gap="stackGap">
        <div class="page-header-split-row">
          <div
            class="flex min-w-0 flex-1 flex-col gap-3 sm:flex-row sm:items-center sm:gap-3 md:gap-4"
          >
            <div v-if="hasLeading" class="mx-auto shrink-0 sm:mx-0"><slot name="leading" /></div>

            <div class="page-header-body min-w-0 flex-1 text-center sm:text-left">
              <div v-if="hasEyebrow" class="min-w-0"><slot name="eyebrow" /></div>
              <component
                :is="titleAs"
                v-bind="titleAttrs"
                class="page-header-title"
                :data-tone="titleTone === 'secondary' ? 'secondary' : undefined"
              >
                <slot name="title" />
              </component>
              <div v-if="hasDescription" class="min-w-0"><slot name="description" /></div>
            </div>
          </div>

          <div
            v-if="hasTrailing"
            class="page-header-trailing w-full justify-center sm:w-auto sm:justify-end"
          >
            <slot name="trailing" />
          </div>
        </div>

        <div v-if="hasMeta || hasMetaTrailing" class="page-header-meta">
          <div v-if="hasMeta" data-slot="meta" class="page-header-meta-cluster">
            <slot name="meta" />
          </div>

          <div v-if="hasMetaTrailing" data-slot="meta-trailing" class="page-header-meta-trailing">
            <slot name="meta-trailing" />
          </div>
        </div>

        <div v-if="hasToolbar" class="page-header-toolbar"><slot name="toolbar" /></div>
      </div>
    </template>
    <!-- Stack (default) -->
    <template v-else>
      <div class="page-header-stack" :data-stack-gap="stackGap">
        <template v-if="hasLeading">
          <div class="page-header-leading-row">
            <div class="shrink-0"><slot name="leading" /></div>

            <div class="page-header-body min-w-0 flex-1">
              <div v-if="hasEyebrow" class="min-w-0"><slot name="eyebrow" /></div>

              <div class="page-header-title-row">
                <div class="page-header-title-cluster">
                  <span v-if="hasIcon" class="inline-flex shrink-0 items-center text-gray-500">
                    <slot name="icon" />
                  </span>
                  <component
                    :is="titleAs"
                    v-bind="titleAttrs"
                    class="page-header-title"
                    :data-tone="titleTone === 'secondary' ? 'secondary' : undefined"
                  >
                    <slot name="title" />
                  </component>
                  <span class="inline-flex shrink-0 items-center empty:hidden">
                    <slot name="title-after" />
                  </span>
                </div>

                <div v-if="hasTrailing" class="page-header-trailing">
                  <slot name="trailing" />
                </div>
              </div>

              <div v-if="hasDescription" class="min-w-0"><slot name="description" /></div>
            </div>
          </div>
        </template>
        <template v-else>
          <div class="page-header-body">
            <div v-if="hasEyebrow" class="min-w-0"><slot name="eyebrow" /></div>

            <div class="page-header-title-row">
              <div class="page-header-title-cluster">
                <span v-if="hasIcon" class="inline-flex shrink-0 items-center text-gray-500">
                  <slot name="icon" />
                </span>
                <component
                  :is="titleAs"
                  v-bind="titleAttrs"
                  class="page-header-title"
                  :data-tone="titleTone === 'secondary' ? 'secondary' : undefined"
                >
                  <slot name="title" />
                </component>
                <span class="inline-flex shrink-0 items-center empty:hidden">
                  <slot name="title-after" />
                </span>
              </div>

              <div v-if="hasTrailing" class="page-header-trailing">
                <slot name="trailing" />
              </div>
            </div>

            <div v-if="hasDescription" class="min-w-0"><slot name="description" /></div>
          </div>
        </template>
        <div v-if="hasMeta || hasMetaTrailing" class="page-header-meta">
          <div v-if="hasMeta" data-slot="meta" class="page-header-meta-cluster">
            <slot name="meta" />
          </div>

          <div v-if="hasMetaTrailing" data-slot="meta-trailing" class="page-header-meta-trailing">
            <slot name="meta-trailing" />
          </div>
        </div>

        <div v-if="hasToolbar" class="page-header-toolbar"><slot name="toolbar" /></div>
      </div>
    </template>
  </header>
</template>
