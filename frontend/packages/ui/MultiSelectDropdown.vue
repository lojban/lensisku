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

    <div
      v-show="open"
      class="absolute left-0 right-0 z-50 mt-1 flex min-h-0 flex-col rounded-lg border border-gray-200 bg-white py-2 shadow-lg"
      role="presentation"
      :style="panelViewportStyle"
    >
       <!-- Search: see `searchFieldKeys` prop, else values-only deep match -->
      <div class="border-b border-gray-100 px-2 pb-2">
         <div class="relative">
           <Search
            class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400"
            aria-hidden="true"
          /> <input
            v-model="searchQuery"
            type="search"
            :placeholder="searchPlaceholder"
            class="input-field w-full !h-9 pl-9 pr-3 text-sm"
            autocomplete="off"
            @keydown.escape.stop="open = false"
          />
        </div>
      </div>

       <!-- Select all (applies to the filtered list) -->
      <div class="border-b border-gray-100 px-2 py-1.5">
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
          /> <span class="select-none">{{ selectAllLabel }}</span>
        </label>
      </div>

       <ul class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-1 py-0.5" role="listbox" aria-multiselectable="true">
         <li v-for="(opt, idx) in filteredOptions" :key="optionKey(opt, idx)" role="option" :aria-selected="isSelected(opt)">
           <label
            class="flex cursor-pointer items-center gap-2 rounded-md px-2 py-2 text-sm text-gray-700 hover:bg-gray-50"
          >
             <input
              type="checkbox"
              class="checkmark-aqua shrink-0"
              :checked="isSelected(opt)"
              @change="toggleOption(opt)"
            /> <span class="min-w-0 flex-1">{{ optionLabel(opt) }}</span>
          </label>
        </li>
         <li v-if="!filteredOptions.length" class="px-3 py-4 text-center text-sm text-gray-500">
           {{ emptyFilterLabel }}
        </li>
      </ul>
    </div>

  </div>

</template>

<script setup lang="ts">
import { ChevronDown, Search } from 'lucide-vue-next'
import {
  ref,
  computed,
  watch,
  watchEffect,
  nextTick,
  onMounted,
  onUnmounted,
  useAttrs,
} from 'vue'
import type { PropType } from 'vue'

defineOptions({ inheritAttrs: false })

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
  emptyFilterLabel: {
    type: String,
    default: 'No matches',
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
})

const emit = defineEmits<{
  'update:modelValue': [value: unknown[]]
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
const selectAllInputRef = ref<HTMLInputElement | null>(null)
const panelViewportStyle = ref<Record<string, string>>({})

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
  panelViewportStyle.value = {
    maxHeight: `${maxPx}px`,
    minHeight: `${minPx}px`,
  }
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
function listedFieldValuesMatch(
  item: unknown,
  needleLower: string,
  keys: string[]
): boolean {
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
function deepFieldSubstringMatch(value: unknown, needleLower: string, seen: WeakSet<object>): boolean {
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

const allFilteredSelected = computed(() => {
  const list = filteredOptions.value
  if (!list.length) return false
  return list.every((o) => isSelected(o))
})

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
  if (rootRef.value && el instanceof Node && !rootRef.value.contains(el)) {
    open.value = false
  }
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
  await nextTick()
  requestAnimationFrame(() => {
    updatePanelViewportStyle()
  })
})

function onViewportChange() {
  if (open.value) {
    updatePanelViewportStyle()
  }
}

onMounted(() => {
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
