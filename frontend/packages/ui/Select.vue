<template>
  <div class="relative" :class="searchable ? $attrs.class : undefined">
    <MultiSelectDropdown
      v-if="searchable"
      :id="resolvedId"
      ref="searchableSelect"
      :model-value="dropdownSelection"
      :options="searchableOptions"
      :option-value="dropdownOptionValue"
      :option-label="dropdownOptionLabel"
      :search-field-keys="['label', 'value', 'searchText']"
      :single-select="!isMultiple"
      :disabled="disabled"
      :aria-label="label ?? ($attrs['aria-label'] as string | undefined)"
      :aria-required="$attrs.required != null && $attrs.required !== false"
      :trigger-class="hostClass"
      :placeholder="placeholder"
      :search-placeholder="searchPlaceholder"
      :empty-filter-label="emptyFilterLabel"
      teleport-panel
      class="w-full min-w-0"
      @update:model-value="onDropdownChange"
    />
    <Component
      :is="selectedIcon"
      v-if="selectedIcon && !searchable"
      class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-500 max-sm:left-1/2 max-sm:-translate-x-1/2"
      aria-hidden="true"
    />
    <select
      :id="searchable ? `${resolvedId}-native` : resolvedId"
      ref="nativeSelect"
      :disabled="disabled"
      :name="name"
      :aria-label="label"
      :aria-hidden="searchable || undefined"
      :tabindex="searchable ? -1 : undefined"
      :class="searchable ? 'sr-only' : hostClass"
      :multiple="isMultiple"
      v-bind="nativeAttrs"
      @change="onChange"
      @invalid="onInvalid"
    >
      <option
        v-for="opt in normalizedOptions"
        :key="opt.value"
        :value="opt.value"
        :disabled="opt.disabled"
        :selected="isOptionSelected(opt.value)"
      >
        {{ opt.label }}
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, useAttrs } from 'vue'
import MultiSelectDropdown from './MultiSelectDropdown.vue'
import type { PropType } from 'vue'

interface Option {
  [key: string]: unknown
  value: string | number | null
  label: string
  disabled?: boolean
  icon?: unknown
}

type OptionInput = Option | string | number

type ModelValue = string | number | null | unknown[]

type ModelModifiers = {
  number?: boolean
}

const props = defineProps({
  modelValue: { type: [String, Number, Array] as PropType<ModelValue>, default: '' },
  options: { type: Array as PropType<OptionInput[]>, default: () => [] as OptionInput[] },
  optionValue: { type: String, default: 'value' },
  optionLabel: { type: String, default: 'label' },
  optionDisabled: { type: String, default: 'disabled' },
  optionIcon: { type: String, default: 'icon' },
  label: { type: String, default: undefined },
  id: { type: String, default: undefined },
  placeholder: { type: String, default: undefined },
  disabled: { type: Boolean, default: false },
  name: { type: String, default: undefined },
  size: { type: String, default: 'md', validator: (v: string) => ['md', 'lg'].includes(v) },
  selectClass: { type: [String, Array, Object], default: '' },
  multiple: { type: Boolean, default: false },
  /** Use the shared searchable dropdown while retaining scalar values and native form validation. */
  searchable: { type: Boolean, default: false },
  searchPlaceholder: { type: String, default: '' },
  emptyFilterLabel: { type: String, default: 'No matches' },
  modelModifiers: { type: Object as () => ModelModifiers, default: () => ({}) },
})

const emit = defineEmits<{ 'update:modelValue': [value: ModelValue]; change: [event: Event] }>()

defineOptions({ name: 'UiSelect', inheritAttrs: false })

const attrs = useAttrs()
const nativeAttrs = computed(() => {
  if (!props.searchable) return attrs
  const { class: _class, style: _style, ...rest } = attrs
  return rest
})
const searchableSelect = ref<InstanceType<typeof MultiSelectDropdown> | null>(null)
const nativeSelect = ref<HTMLSelectElement | null>(null)
const generatedId = `ui-select-${Math.random().toString(36).slice(2, 9)}`
const resolvedId = computed(() => props.id ?? generatedId)

const hostClass = computed(() => {
  const extra = Array.isArray(props.selectClass) ? props.selectClass.join(' ') : props.selectClass
  return [
    'w-full',
    selectedIcon.value && !props.searchable
      ? 'pl-9 max-sm:pl-0 max-sm:pr-2 max-sm:w-11 max-sm:text-transparent'
      : '',
    extra,
  ]
    .filter(Boolean)
    .join(' ')
})

const normalizedOptions = computed<Option[]>(() => {
  return props.options.map((opt) => {
    if (typeof opt === 'string' || typeof opt === 'number') {
      return { value: String(opt), label: String(opt) }
    }
    return {
      ...opt,
      value: opt[props.optionValue] as string | number | null,
      label: String(opt[props.optionLabel]),
      disabled: Boolean(opt[props.optionDisabled]),
      icon: opt[props.optionIcon],
    }
  })
})

const isMultiple = computed(() => props.multiple || Array.isArray(props.modelValue))

function isOptionSelected(value: string | number | null): boolean {
  if (isMultiple.value) {
    const selected = props.modelValue as unknown[]
    return selected.some((v) => String(v) === String(value))
  }
  return String(props.modelValue) === String(value)
}

const selectedIcon = computed(() => {
  const selected = normalizedOptions.value.find((opt) => isOptionSelected(opt.value))
  return selected?.icon
})

const searchableOptions = computed(() =>
  normalizedOptions.value.filter((option) => !option.disabled)
)
const dropdownSelection = computed(() =>
  normalizedOptions.value.filter((option) => isOptionSelected(option.value))
)
const dropdownOptionValue = (option: unknown) => (option as Option).value
const dropdownOptionLabel = (option: unknown) => (option as Option).label

function onDropdownChange(selection: unknown[]) {
  const selected = selection as Option[]
  const values = selected.map((option) =>
    option.value === null ? null : normalizeValue(String(option.value))
  )
  emit('update:modelValue', isMultiple.value ? values : (values[0] ?? null))

  // Dispatch from the backing select so existing @change handlers still receive a DOM event.
  const native = nativeSelect.value
  if (native) {
    Array.from(native.options).forEach((option, index) => {
      option.selected = selected.some((item) => item.value === normalizedOptions.value[index].value)
    })
    native.dispatchEvent(new Event('change', { bubbles: true }))
  }
}

function onInvalid(event: Event) {
  if (!props.searchable) return
  event.preventDefault()
  focus()
}

function normalizeValue(raw: string): string | number {
  if (props.modelModifiers.number) {
    const n = Number(raw)
    return Number.isNaN(n) ? raw : n
  }
  return raw
}

function onChange(e: Event) {
  const target = e.target as HTMLSelectElement | null
  if (!target) return
  if (props.searchable) {
    emit('change', e)
    return
  }

  if (isMultiple.value) {
    const selected = Array.from(target.selectedOptions).map((o) => normalizeValue(o.value))
    emit('update:modelValue', selected)
    emit('change', e)
    return
  }

  const raw = target.value ?? ''
  emit('update:modelValue', normalizeValue(raw))
  emit('change', e)
}

function focus() {
  if (props.searchable) searchableSelect.value?.focus()
  else nativeSelect.value?.focus()
}

defineExpose({ select: nativeSelect, focus })
</script>
