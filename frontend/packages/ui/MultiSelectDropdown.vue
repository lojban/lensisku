<template>
  <div ref="rootRef" :class="rootClass" v-bind="wrapperAttrs">
    <button
      type="button"
      class="dropdown-trigger"
      :aria-expanded="open"
      aria-haspopup="listbox"
      @click="toggleOpen"
      @keydown.escape.prevent="open = false"
    >
      <span class="min-w-0 flex-1 truncate">{{ summaryText }}</span>
      <ChevronDown
        class="h-4 w-4 shrink-0 text-gray-500 transition-transform duration-200"
        :class="{ 'rotate-180': open }"
        aria-hidden="true"
      />
    </button>
    <Teleport to="body" :disabled="!useViewportPanel">
      <div
        v-show="open"
        ref="panelRef"
        class="dropdown-floating-panel"
        :class="{ 'dropdown-floating-panel--viewport': useViewportPanel }"
        role="presentation"
        :style="panelViewportStyle"
      >
        <div v-if="preface" class="dropdown-option-preface">
          {{ preface }}
        </div>
        <!-- Search: see `searchFieldKeys` prop, else values-only deep match -->
        <div class="border-b border-gray-100 px-2 pb-2">
          <Input
            v-model="searchQuery"
            type="search"
            search-icon
            :placeholder="searchPlaceholder"
            input-class="input-field w-full !h-9 pr-3 text-sm"
            autocomplete="off"
            @keydown.escape.stop="open = false"
          />
        </div>
        <!-- Select all (applies to the filtered list) -->
        <div v-if="showSelectAll" class="border-b border-gray-100 px-2 py-1.5">
          <label
            class="flex cursor-pointer items-center gap-2 rounded-md px-1 py-1 text-sm text-gray-700 hover:bg-gray-50"
            :class="{ 'pointer-events-none opacity-50': !filteredOptions.length }"
          >
            <input
              ref="selectAllInputRef"
              type="checkbox"
              class="checkmark-aqua shrink-0"
              :checked="allFilteredSelected"
              :disabled="!filteredOptions.length"
              @change="toggleSelectAll"
            />
            <span class="select-none">{{ selectAllRowLabel }}</span>
          </label>
        </div>

        <ul
          class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-1 py-0.5"
          role="listbox"
          aria-multiselectable="true"
        >
          <template v-if="visibleSuggested.length">
            <li class="px-2 pt-1.5 pb-0.5 text-[11px] font-semibold uppercase tracking-wide text-gray-400">
              {{ suggestedLabel }}
            </li>
            <li
              v-for="(opt, idx) in visibleSuggested"
              :key="`sug-${optionKey(opt, idx)}`"
              role="option"
              :aria-selected="isSelected(opt)"
            >
              <label
                class="flex cursor-pointer items-center gap-2 rounded-md px-2 py-2 text-sm text-gray-700 hover:bg-gray-50"
              >
                <input
                  type="checkbox"
                  class="checkmark-aqua shrink-0"
                  :checked="isSelected(opt)"
                  @change="toggleOption(opt)"
                />
                <slot name="option" :option="opt">
                  <span class="min-w-0 flex-1 truncate">{{ optionLabel(opt) }}</span>
                </slot>
              </label>
            </li>
            <li class="my-1 border-t border-gray-100" aria-hidden="true" />
          </template>

          <li
            v-for="(opt, idx) in filteredOptions"
            :key="optionKey(opt, idx)"
            role="option"
            :aria-selected="isSelected(opt)"
          >
            <label
              class="flex cursor-pointer items-center gap-2 rounded-md px-2 py-2 text-sm text-gray-700 hover:bg-gray-50"
            >
              <input
                type="checkbox"
                class="checkmark-aqua shrink-0"
                :checked="isSelected(opt)"
                @change="toggleOption(opt)"
              />
              <slot name="option" :option="opt">
                <span class="min-w-0 flex-1 truncate">{{ optionLabel(opt) }}</span>
              </slot>
            </label>
          </li>

          <li
            v-if="!filteredOptions.length && !visibleSuggested.length"
            class="px-3 py-4 text-center text-sm text-gray-500"
          >
            {{ emptyFilterLabel }}
          </li>
        </ul>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ChevronDown } from 'lucide-vue-next'
import { ref, computed, watch, watchEffect, nextTick, onMounted, onUnmounted, useAttrs } from 'vue'
import type { PropType } from 'vue'

import Input from './Input.vue'

defineOptions({ inheritAttrs: false })

const SM_BREAKPOINT_PX = 640

const props = defineProps({
  /** Selected option objects (same references / identity as in `options` after value compare) */
  modelValue: {
    type: Array as PropType<unknown[]>,
    required: true,
  },
  options: {
    type: Array as PropType<unknown[]>,
    required: true,
  },
  optionLabel: {
    type: Function as PropType<(item: unknown) => string>,
    required: true,
  },
  /** Stable identity for equality and list keys */
  optionValue: {
    type: Function as PropType<(item: unknown) => unknown>,
    required: true,
  },
  placeholder: {
    type: String,
    default: '',
  },
  searchPlaceholder: {
    type: String,
    default: '',
  },
  selectAllLabel: {
    type: String,
    default: 'Select all',
  },
  deselectAllLabel: {
    type: String,
    default: 'Deselect all',
  },
  emptyFilterLabel: {
    type: String,
    default: 'No matches',
  },
  /** When false, the “Select all” row is hidden (useful for async option lists). */
  showSelectAll: {
    type: Boolean,
    default: true,
  },
  /** Max primary labels before summarizing with “+N” */
  maxSelectedLabels: {
    type: Number,
    default: 3,
  },
  /**
   * If set, search is a case-insensitive substring match on **values** of these keys only (not key names).
   * If unset, search walks nested **values** only (still not property names).
   */
  searchFieldKeys: {
    type: Array as PropType<string[]>,
    default: undefined,
  },
  /**
   * Copies of recently selected items shown first in the list (still appear in `options` as well).
   */
  suggestedOptions: {
    type: Array as PropType<unknown[]>,
    default: () => [],
  },
  suggestedLabel: {
    type: String,
    default: 'Recent',
  },
  /** On viewports below `sm`, pin the panel to the viewport width instead of the trigger column. */
  fullBleedMobilePanel: {
    type: Boolean,
    default: false,
  },
  /** Short hint shown at the top of the open panel. */
  preface: {
    type: String,
    default: '',
  },
})

const emit = defineEmits<{
  'update:modelValue': [value: unknown[]]
  search: [query: string]
  open: []
}>()

const attrs = useAttrs()
const wrapperAttrs = computed(() => {
  const { class: _c, ...rest } = attrs as Record<string, unknown>
  return rest
})

const rootClass = computed(() => {
  const cls = attrs.class
  return ['relative', cls].filter(Boolean).join(' ')
})

/** `mt-1` gap between trigger and panel */
const PANEL_GAP_PX = 4
/** Padding from the bottom of the viewport */
const VIEWPORT_BOTTOM_PAD_PX = 8
/** Max panel height when there is plenty of vertical space (matches previous 24rem cap) */
const PANEL_MAX_REM = 24
/** Minimum panel height when the viewport allows (user request: not below 20rem) */
const PANEL_MIN_REM = 20

const open = ref(false)
const searchQuery = ref('')
const rootRef = ref<HTMLElement | null>(null)
const panelRef = ref<HTMLElement | null>(null)
const selectAllInputRef = ref<HTMLInputElement | null>(null)
const panelViewportStyle = ref<Record<string, string>>({})
const isNarrow = ref(false)

const useViewportPanel = computed(() => props.fullBleedMobilePanel && isNarrow.value)

function updateNarrow() {
  isNarrow.value = typeof window !== 'undefined' && window.innerWidth < SM_BREAKPOINT_PX
}

function getRootRemPx(): number {
  if (typeof document === 'undefined') return 16
  return parseFloat(getComputedStyle(document.documentElement).fontSize) || 16
}

function updatePanelViewportStyle() {
  if (!open.value || !rootRef.value) {
    panelViewportStyle.value = {}
    return
  }
  const btn = rootRef.value.querySelector('button')
  if (!btn) {
    panelViewportStyle.value = {}
    return
  }
  const rect = btn.getBoundingClientRect()
  const rem = getRootRemPx()
  const availableBelow = Math.max(
    0,
    window.innerHeight - rect.bottom - PANEL_GAP_PX - VIEWPORT_BOTTOM_PAD_PX
  )
  const maxPx = Math.min(availableBelow, PANEL_MAX_REM * rem)
  const minPx = Math.min(PANEL_MIN_REM * rem, maxPx)
  const style: Record<string, string> = {
    maxHeight: `${maxPx}px`,
    minHeight: `${minPx}px`,
  }
  if (useViewportPanel.value) {
    style.top = `${rect.bottom + PANEL_GAP_PX}px`
  }
  panelViewportStyle.value = style
}

function valuesEqual(a: unknown, b: unknown): boolean {
  return a === b
}

function getValue(item: unknown): unknown {
  return props.optionValue(item)
}

function isSelected(item: unknown): boolean {
  const v = getValue(item)
  return props.modelValue.some((x) => valuesEqual(getValue(x), v))
}

function optionKey(item: unknown, index: number): string | number {
  const v = getValue(item)
  if (v != null && (typeof v === 'string' || typeof v === 'number')) return v
  return index
}

function haystackIncludesNeedle(haystack: string, needleLower: string): boolean {
  return haystack.toLowerCase().includes(needleLower)
}

/** Value-only substring match (keys on the option object are not searched). */
function listedFieldValuesMatch(item: unknown, needleLower: string, keys: string[]): boolean {
  if (item === null || typeof item !== 'object') return false
  const rec = item as Record<string, unknown>
  for (const key of keys) {
    if (!Object.prototype.hasOwnProperty.call(rec, key)) continue
    if (valueMatchesNeedle(rec[key], needleLower)) return true
  }
  return false
}

/** Case-insensitive substring on a single value (objects/arrays serialized for search text). */
function valueMatchesNeedle(value: unknown, needleLower: string): boolean {
  if (value === null || value === undefined) return false
  const t = typeof value
  if (t === 'string') return haystackIncludesNeedle(value as string, needleLower)
  if (t === 'number' || t === 'boolean' || t === 'bigint') {
    return haystackIncludesNeedle(String(value), needleLower)
  }
  if (t === 'symbol') return haystackIncludesNeedle(String(value), needleLower)
  if (t === 'function') return false
  if (t === 'object') {
    try {
      return JSON.stringify(value).toLowerCase().includes(needleLower)
    } catch {
      return false
    }
  }
  return haystackIncludesNeedle(String(value), needleLower)
}

/** Nested values only; property names are not matched. */
function deepFieldSubstringMatch(
  value: unknown,
  needleLower: string,
  seen: WeakSet<object>
): boolean {
  if (value === null || value === undefined) return false
  const t = typeof value
  if (t === 'string') return haystackIncludesNeedle(value as string, needleLower)
  if (t === 'number' || t === 'boolean' || t === 'bigint') {
    return haystackIncludesNeedle(String(value), needleLower)
  }
  if (t === 'symbol') return haystackIncludesNeedle(String(value), needleLower)
  if (t === 'function') return false
  if (t !== 'object') return haystackIncludesNeedle(String(value), needleLower)

  const obj = value as object
  if (seen.has(obj)) return false
  seen.add(obj)

  if (Array.isArray(value)) {
    for (let i = 0; i < value.length; i++) {
      if (deepFieldSubstringMatch(value[i], needleLower, seen)) return true
    }
    return false
  }

  const rec = value as Record<string, unknown>
  for (const key of Object.keys(rec)) {
    if (deepFieldSubstringMatch(rec[key], needleLower, seen)) return true
  }
  return false
}

function itemMatchesQuery(item: unknown, q: string): boolean {
  const needleLower = q.trim().toLowerCase()
  if (!needleLower) return true
  const keys = props.searchFieldKeys
  if (keys && keys.length > 0) {
    return listedFieldValuesMatch(item, needleLower, keys)
  }
  return deepFieldSubstringMatch(item, needleLower, new WeakSet())
}

const filteredOptions = computed(() => {
  return props.options.filter((o) => itemMatchesQuery(o, searchQuery.value))
})

const visibleSuggested = computed(() => {
  const seen = new Set<unknown>()
  const out: unknown[] = []
  for (const opt of props.suggestedOptions) {
    const v = getValue(opt)
    if (seen.has(v)) continue
    seen.add(v)
    if (itemMatchesQuery(opt, searchQuery.value)) out.push(opt)
  }
  return out
})

const allFilteredSelected = computed(() => {
  const list = filteredOptions.value
  if (!list.length) return false
  return list.every((o) => isSelected(o))
})

const selectAllRowLabel = computed(() =>
  allFilteredSelected.value ? props.deselectAllLabel : props.selectAllLabel
)

watchEffect(() => {
  const el = selectAllInputRef.value
  if (!el) return
  const list = filteredOptions.value
  if (!list.length) {
    el.indeterminate = false
    return
  }
  let n = 0
  for (const o of list) {
    if (isSelected(o)) n++
  }
  el.indeterminate = n > 0 && n < list.length
})

function toggleOpen() {
  open.value = !open.value
}

function toggleOption(item: unknown) {
  const v = getValue(item)
  const next = [...props.modelValue]
  const i = next.findIndex((x) => valuesEqual(getValue(x), v))
  if (i >= 0) {
    next.splice(i, 1)
  } else {
    const fromOptions = props.options.find((o) => valuesEqual(getValue(o), v))
    next.push(fromOptions !== undefined ? fromOptions : item)
  }
  emit('update:modelValue', next)
}

function toggleSelectAll() {
  const list = filteredOptions.value
  if (!list.length) return
  if (allFilteredSelected.value) {
    const remove = new Set(list.map((o) => getValue(o)))
    emit(
      'update:modelValue',
      props.modelValue.filter((x) => !remove.has(getValue(x)))
    )
  } else {
    const selectedValues = new Set(props.modelValue.map((x) => getValue(x)))
    const next = [...props.modelValue]
    for (const o of list) {
      const v = getValue(o)
      if (!selectedValues.has(v)) {
        selectedValues.add(v)
        next.push(o)
      }
    }
    emit('update:modelValue', next)
  }
}

const summaryText = computed(() => {
  const sel = props.modelValue
  if (!sel.length) return props.placeholder
  const max = props.maxSelectedLabels
  const labels = sel.map((x) => props.optionLabel(x))
  if (labels.length <= max) return labels.join(', ')
  const shown = labels.slice(0, max).join(', ')
  const rest = labels.length - max
  return `${shown} (+${rest})`
})

function handleClickOutside(event: MouseEvent) {
  const el = event.target
  if (!(el instanceof Node)) return
  if (rootRef.value && rootRef.value.contains(el)) return
  if (panelRef.value && panelRef.value.contains(el)) return
  open.value = false
}

function handleEscapeGlobal(event: KeyboardEvent) {
  if (event.key === 'Escape' && open.value) {
    open.value = false
  }
}

watch(open, async (isOpen) => {
  if (!isOpen) {
    searchQuery.value = ''
    panelViewportStyle.value = {}
    return
  }
  emit('open')
  await nextTick()
  requestAnimationFrame(() => {
    updatePanelViewportStyle()
  })
})

watch(searchQuery, (q) => {
  if (open.value) {
    emit('search', q)
  }
})

watch(useViewportPanel, () => {
  if (open.value) {
    updatePanelViewportStyle()
  }
})

function onViewportChange() {
  updateNarrow()
  if (open.value) {
    updatePanelViewportStyle()
  }
}

onMounted(() => {
  updateNarrow()
  document.addEventListener('mousedown', handleClickOutside)
  document.addEventListener('keydown', handleEscapeGlobal)
  window.addEventListener('resize', onViewportChange)
  window.addEventListener('scroll', onViewportChange, true)
})

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside)
  document.removeEventListener('keydown', handleEscapeGlobal)
  window.removeEventListener('resize', onViewportChange)
  window.removeEventListener('scroll', onViewportChange, true)
})
</script>
