<template>
  <Dropdown>
    <template #trigger="{ open }">
      <button
        :id="id"
        type="button"
        :aria-label="ariaLabel"
        class="input-field inline-flex h-8 w-auto max-w-full min-w-0 items-center justify-between gap-1.5 px-3 text-left text-sm"
        :class="variant === 'role' ? 'max-w-[min(100vw-4rem,14rem)]' : ''"
      >
        <span :class="truncateLabel ? 'min-w-0 truncate whitespace-nowrap' : 'whitespace-nowrap'">
          <slot name="label" />
        </span>
        <ChevronDown
          class="h-4 w-4 shrink-0 opacity-60 transition-transform duration-200"
          :class="{ 'rotate-180': open }"
          :stroke-width="2"
        />
      </button>
    </template>
    <slot />
  </Dropdown>
</template>

<script setup lang="ts">
import { ChevronDown } from 'lucide-vue-next'
import Dropdown from './Dropdown.vue'

defineOptions({ inheritAttrs: false })

withDefaults(
  defineProps<{
    id?: string
    ariaLabel?: string
    /** Wider labels (e.g. role names): ellipsis when overflow. */
    truncateLabel?: boolean
    /** `role` caps trigger width on small viewports. */
    variant?: 'default' | 'role'
  }>(),
  {
    variant: 'default',
    truncateLabel: false,
  }
)
</script>
