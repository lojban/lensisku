<template>
  <ToolbarSelectDropdown
    :id="id"
    :aria-label="ariaLabel"
    :trigger-class="triggerClass"
    :columns="2"
  >
    <template #icon>
      <ArrowUpNarrowWide v-if="order === 'asc'" class="h-4 w-4 opacity-60" :stroke-width="2" />
      <ArrowDownWideNarrow v-else class="h-4 w-4 opacity-60" :stroke-width="2" />
    </template>
    <template #label>{{ selectedLabel }}</template>
    <template v-for="opt in options" :key="opt.value">
      <ToolbarSelectDropdownItem
        :class="{ 'bg-gray-100': field === opt.value && order === 'desc' }"
        :aria-label="itemAriaLabel(opt.label, 'desc')"
        @click="emit('select', { field: opt.value, order: 'desc' })"
      >
        <ArrowDownWideNarrow class="h-4 w-4 shrink-0 opacity-60" :stroke-width="2" />
        {{ opt.label }}
      </ToolbarSelectDropdownItem>
      <ToolbarSelectDropdownItem
        :class="{ 'bg-gray-100': field === opt.value && order === 'asc' }"
        :aria-label="itemAriaLabel(opt.label, 'asc')"
        @click="emit('select', { field: opt.value, order: 'asc' })"
      >
        <ArrowUpNarrowWide class="h-4 w-4 shrink-0 opacity-60" :stroke-width="2" />
        {{ opt.label }}
      </ToolbarSelectDropdownItem>
    </template>
  </ToolbarSelectDropdown>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ArrowDownWideNarrow, ArrowUpNarrowWide } from '@lucide/vue'
import ToolbarSelectDropdown from './ToolbarSelectDropdown.vue'
import ToolbarSelectDropdownItem from './ToolbarSelectDropdownItem.vue'

export type SortOrder = 'asc' | 'desc'
export type SortFieldOption = { value: string; label: string }

const props = withDefaults(
  defineProps<{
    field: string
    order: SortOrder | string
    options: SortFieldOption[]
    id?: string
    ariaLabel?: string
    triggerClass?: string
    /** Accessible names for the two direction columns, e.g. "Sort descending". */
    descendingLabel?: string
    ascendingLabel?: string
  }>(),
  {
    id: undefined,
    ariaLabel: undefined,
    triggerClass: '!w-full max-w-[min(100vw-4rem,18rem)]',
    descendingLabel: '',
    ascendingLabel: '',
  }
)

const emit = defineEmits<{
  select: [payload: { field: string; order: SortOrder }]
}>()

const selectedLabel = computed(
  () => props.options.find((o) => o.value === props.field)?.label ?? props.options[0]?.label ?? ''
)

function itemAriaLabel(label: string, direction: SortOrder) {
  const directionLabel = direction === 'desc' ? props.descendingLabel : props.ascendingLabel
  return directionLabel ? `${label}, ${directionLabel}` : label
}
</script>
