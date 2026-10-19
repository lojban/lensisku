<template>
  <Dropdown :columns="columns">
    <template #trigger="{ open }">
      <slot name="trigger" :open="open">
        <button
          :id="id"
          type="button"
          :aria-label="ariaLabel"
          aria-haspopup="menu"
          :aria-expanded="open"
          class="w-auto max-w-full min-w-0 shrink overflow-hidden flex-nowrap"
          :class="[
            triggerIcon === 'ellipsis' ? 'dropdown-action-trigger' : 'dropdown-trigger',
            variant === 'role' ? 'max-w-[min(100vw-4rem,14rem)]' : '',
            triggerClass,
          ]"
        >
          <span v-if="$slots.icon" class="shrink-0 inline-flex items-center" aria-hidden="true">
            <slot name="icon" />
          </span>
          <span class="dropdown-trigger-label min-w-0 max-w-full truncate">
            <slot name="label" />
          </span>
          <ChevronDown
            v-if="triggerIcon === 'chevron'"
            class="h-4 w-4 shrink-0 opacity-60 transition-transform duration-200"
            :class="{ 'rotate-180': open }"
            :stroke-width="2"
          />
          <EllipsisVertical v-else class="h-4 w-4 shrink-0 opacity-60" :stroke-width="2" />
        </button>
      </slot>
    </template>
    <slot />
  </Dropdown>
</template>

<script setup lang="ts">
import { ChevronDown, EllipsisVertical } from '@lucide/vue'
import Dropdown from './Dropdown.vue'

defineOptions({ inheritAttrs: false })

withDefaults(
  defineProps<{
    id?: string
    ariaLabel?: string
    /** `role` caps trigger width on small viewports. */
    variant?: 'default' | 'role'
    /** Trailing icon for the trigger: `chevron` for selects, `ellipsis` for action menus. */
    triggerIcon?: 'chevron' | 'ellipsis'
    /** Extra classes to override the trigger button styling. */
    triggerClass?: string
    /** Lay menu items in this many columns. `2` pairs items on the same row. */
    columns?: number
  }>(),
  {
    id: undefined,
    ariaLabel: undefined,
    variant: 'default',
    triggerIcon: 'chevron',
    triggerClass: '',
    columns: 1,
  }
)
</script>
