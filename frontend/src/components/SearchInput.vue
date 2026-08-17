<script setup lang="ts">
import { Loader2, X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { computed, ref, useAttrs, type PropType } from 'vue'
import { IconButtonGhost, Input, Select } from '@packages/ui'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'

const { t } = useI18n()
const attrs = useAttrs()

const props = defineProps({
  modelValue: {
    type: String,
    default: '',
  },
  isLoading: {
    type: Boolean,
    default: false,
  },
  placeholder: {
    type: String,
    default: '',
  },
  showSearchIcon: {
    type: Boolean,
    default: false,
  },
  /** Optional mode select value (v-model) */
  modeValue: {
    type: String,
    default: '',
  },
  /** Options for the mode selector: { value, label, icon? } */
  modeOptions: {
    type: Array as PropType<{ value: string; label: string; icon?: unknown }[]>,
    default: () => [],
  },
  /** Placeholder shown when no mode is selected */
  selectPlaceholder: {
    type: String,
    default: '',
  },
  /** Whether an addon is joined to the right of the input (removes right radius) */
  joinedRight: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['update:modelValue', 'update:modeValue', 'clear', 'search'])

const hasModes = computed(() => props.modeOptions.length > 0)
const joinedLeft = computed(() => props.modeOptions.length > 0)
const joinedRight = computed(() => props.joinedRight)

const selectOptions = computed(() =>
  props.modeOptions.map((m) => ({ value: m.value, label: m.label, icon: m.icon }))
)

const query = computed({
  get: () => props.modelValue,
  set: (value: string) => emit('update:modelValue', normalizeSearchQuery(value) as string),
})

const mode = computed({
  get: () => props.modeValue,
  set: (value: string | number) => emit('update:modeValue', String(value)),
})

function onKeyup(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    emit('search', { query: query.value, mode: mode.value })
  }
}

function onClear() {
  query.value = ''
  emit('clear')
}

const inputRef = ref<InstanceType<typeof Input> | null>(null)

function focus() {
  inputRef.value?.focus()
}

defineExpose({ focus })
</script>

<template>
  <div class="relative flex w-full min-w-0" :class="attrs.class">
    <div v-if="hasModes" class="shrink-0">
      <Select
        id="search-mode"
        v-model="mode"
        :options="selectOptions"
        :placeholder="selectPlaceholder"
        select-class="input-field rounded-r-none h-10 py-0 pr-8 text-sm bg-gray-50"
      />
    </div>
    <div class="relative flex-1 min-w-0">
      <Input
        ref="inputRef"
        v-model="query"
        type="text"
        :placeholder="placeholder"
        :search-icon="showSearchIcon"
        input-class="input-field w-full min-w-0 sm:min-w-[200px] h-10 transition-colors"
        :class="{
          'rounded-l-none': joinedLeft,
          'rounded-r-none': joinedRight,
          'pr-10': query.length > 0,
        }"
        @keyup.enter="onKeyup"
      />
      <div class="absolute right-3 top-1/2 z-10 -translate-y-1/2 flex items-center gap-1">
        <Loader2
          v-if="isLoading"
          class="h-4 w-4 text-blue-500 animate-spin shrink-0"
          aria-hidden="true"
        />
        <IconButtonGhost
          v-else-if="query"
          compact
          :aria-label="t('components.searchInput.clearAria')"
          @click="onClear"
        >
          <X class="h-4 w-4" />
        </IconButtonGhost>
      </div>
    </div>
  </div>
</template>
