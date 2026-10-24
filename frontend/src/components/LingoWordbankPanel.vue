<template>
  <!-- Assembled answer line (reply bubble). -->
  <div v-if="mode === 'line'" class="lingo-study-wordbank-line" :aria-label="ariaLabel">
    <template v-if="lineChips.length">
      <button
        v-for="chip in lineChips"
        :key="chip.id"
        type="button"
        class="lingo-study-wordbank-chip lingo-study-wordbank-chip--selected"
        :disabled="disabled"
        @click="emit('unpick', chip.id)"
      >
        {{ chip.text }}
      </button>
    </template>
    <span v-else class="lingo-study-wordbank-line-placeholder">{{ placeholder }}</span>
  </div>

  <!-- Source chip bank (below dialogue). -->
  <div v-else class="lingo-study-wordbank-bank" role="list" :aria-label="ariaLabel">
    <button
      v-for="chip in bank"
      :key="chip.id"
      type="button"
      role="listitem"
      class="lingo-study-wordbank-chip"
      :class="{
        'lingo-study-wordbank-chip--used': usedIdSet.has(chip.id),
      }"
      :disabled="disabled || usedIdSet.has(chip.id)"
      :aria-hidden="usedIdSet.has(chip.id) ? true : undefined"
      @click="emit('pick', chip.id)"
    >
      <span class="lingo-study-wordbank-chip__label">{{ chip.text }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { WordbankChip } from '@/utils/wordbank'

const props = withDefaults(
  defineProps<{
    mode: 'line' | 'bank'
    /** Full shuffled bank (always passed). */
    bank: WordbankChip[]
    /** Ordered ids currently on the answer line. */
    selectedIds: string[]
    disabled?: boolean
    placeholder?: string
    ariaLabel?: string
  }>(),
  {
    disabled: false,
    placeholder: '',
    ariaLabel: undefined,
  }
)

const emit = defineEmits<{
  pick: [id: string]
  unpick: [id: string]
}>()

const usedIdSet = computed(() => new Set(props.selectedIds))

const lineChips = computed(() => {
  const byId = new Map(props.bank.map((c) => [c.id, c]))
  return props.selectedIds.map((id) => byId.get(id)).filter((c): c is WordbankChip => !!c)
})
</script>
