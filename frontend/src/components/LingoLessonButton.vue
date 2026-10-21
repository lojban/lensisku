<template>
  <div
    v-if="!locked"
    class="lingo-lesson-button group relative flex flex-row items-center justify-start gap-2"
    :style="nodeStyle"
  >
    <RouterLink
      :to="studyUrl"
      class="flex shrink-0 flex-col items-center no-underline"
      :aria-label="accessibleName"
    >
      <div v-if="current" class="relative h-[102px] w-[102px]">
        <div
          class="absolute -top-6 left-2.5 z-10 animate-bounce rounded-xl border-2 border-slate-200 bg-white px-3 py-2.5 font-bold uppercase tracking-wide text-green-500"
        >
          {{ startLabel }}
          <div
            class="absolute -bottom-2 left-1/2 h-0 w-0 -translate-x-1/2 border-x-8 border-t-8 border-x-transparent border-t-white"
            aria-hidden
          />
        </div>
        <div class="relative h-[102px] w-[102px]">
          <svg class="absolute inset-0 h-[102px] w-[102px] -rotate-90" viewBox="0 0 102 102">
            <circle cx="51" cy="51" r="43" fill="none" stroke="#e5e7eb" stroke-width="8" />
            <circle
              cx="51"
              cy="51"
              r="43"
              fill="none"
              stroke="#4ade80"
              stroke-width="8"
              stroke-linecap="round"
              stroke-dasharray="270.2"
              :stroke-dashoffset="270.2 - (270.2 * Math.min(100, percentage)) / 100"
              class="transition-all duration-300"
            />
          </svg>
          <div class="absolute inset-0 flex items-center justify-center">
            <div class="lingo-lesson-node lingo-lesson-node--open">
              <component :is="iconComponent" class="h-10 w-10" :class="iconClass" />
            </div>
          </div>
        </div>
      </div>
      <div v-else class="lingo-lesson-node lingo-lesson-node--open">
        <component :is="iconComponent" class="h-10 w-10" :class="iconClass" />
      </div>
      <span v-if="label" class="lingo-lesson-label" :title="label">{{ label }}</span>
    </RouterLink>
    <div v-if="showOwnerMenu" class="relative z-20 flex shrink-0 items-center" @click.stop>
      <button
        type="button"
        class="lingo-lesson-menu-trigger"
        :class="{ 'bg-slate-100 text-slate-700': menuOpen }"
        aria-haspopup="true"
        :aria-expanded="menuOpen"
        aria-label="Level options"
        @click.prevent="menuOpen = !menuOpen"
      >
        <Settings class="h-4 w-4" />
      </button>
      <div
        v-if="menuOpen"
        class="absolute left-full top-1/2 z-[50] ml-1 min-w-[180px] -translate-y-1/2 rounded-xl border-2 border-slate-200 bg-white py-1 shadow-lg"
      >
        <slot name="menu" />
      </div>
      <div v-if="menuOpen" class="fixed inset-0 z-[45]" aria-hidden @click="menuOpen = false" />
    </div>
  </div>
  <div
    v-else-if="locked && showOwnerMenu"
    class="lingo-lesson-button group relative flex flex-row items-center justify-start gap-2"
    :style="nodeStyle"
    aria-disabled="true"
  >
    <div class="flex shrink-0 flex-col items-center" :aria-label="accessibleName">
      <div class="lingo-lesson-node lingo-lesson-node--locked pointer-events-none">
        <component
          :is="iconComponent"
          class="h-10 w-10 fill-neutral-400 stroke-neutral-400 text-neutral-400"
        />
      </div>
      <span v-if="label" class="lingo-lesson-label lingo-lesson-label--locked" :title="label">{{
        label
      }}</span>
    </div>
    <div class="relative z-20 flex shrink-0 items-center" @click.stop>
      <button
        type="button"
        class="lingo-lesson-menu-trigger"
        :class="{ 'bg-slate-100 text-slate-700': menuOpen }"
        aria-haspopup="true"
        :aria-expanded="menuOpen"
        aria-label="Level options"
        @click.prevent="menuOpen = !menuOpen"
      >
        <Settings class="h-4 w-4" />
      </button>
      <div
        v-if="menuOpen"
        class="absolute left-full top-1/2 z-[50] ml-1 min-w-[180px] -translate-y-1/2 rounded-xl border-2 border-slate-200 bg-white py-1 shadow-lg"
      >
        <slot name="menu" />
      </div>
      <div v-if="menuOpen" class="fixed inset-0 z-[45]" aria-hidden @click="menuOpen = false" />
    </div>
  </div>
  <div
    v-else
    class="lingo-lesson-button relative flex pointer-events-none"
    :style="nodeStyle"
    aria-disabled="true"
    :aria-label="accessibleName"
  >
    <div class="flex flex-col items-center">
      <div class="lingo-lesson-node lingo-lesson-node--locked">
        <component
          :is="iconComponent"
          class="h-10 w-10 fill-neutral-400 stroke-neutral-400 text-neutral-400"
        />
      </div>
      <span v-if="label" class="lingo-lesson-label lingo-lesson-label--locked" :title="label">{{
        label
      }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { RouterLink } from 'vue-router'
import { Check, Crown, Star, Settings } from '@lucide/vue'

const props = defineProps({
  index: { type: Number, required: true },
  totalCount: { type: Number, required: true },
  locked: { type: Boolean, default: false },
  current: { type: Boolean, default: false },
  completed: { type: Boolean, default: false },
  percentage: { type: Number, default: 0 },
  studyUrl: { type: String, default: '' },
  startLabel: { type: String, default: 'Start' },
  /** Visible level name under the node (e.g. "01 Basic"). */
  label: { type: String, default: '' },
  showOwnerMenu: { type: Boolean, default: false },
})

const menuOpen = ref(false)

const accessibleName = computed(() => {
  if (props.label) return props.label
  if (props.current) return props.startLabel
  return props.locked ? 'Locked lesson' : 'Lesson'
})

const cycleLength = 8
const cycleIndex = computed(() => props.index % cycleLength)

const indentationLevel = computed(() => {
  const i = cycleIndex.value
  if (i <= 2) return i
  if (i <= 4) return 4 - i
  if (i <= 6) return 4 - i
  return i - 8
})

const rightPositionPx = computed(() => indentationLevel.value * 40)

const isFirst = computed(() => props.index === 0)
const isCompleted = computed(() => props.completed)

const nodeStyle = computed(() => ({
  right: `${rightPositionPx.value}px`,
  // Extra vertical room so `.lingo-lesson-label` (up to 2 lines) never crowds the next node.
  marginTop: isFirst.value && !isCompleted.value ? '60px' : '36px',
}))

const iconComponent = computed(() => {
  if (props.completed) return Check
  if (props.index === props.totalCount) return Crown
  return Star
})

const iconClass = computed(() => {
  if (props.locked) return 'fill-neutral-400 stroke-neutral-400 text-neutral-400'
  if (props.completed) return 'fill-none stroke-[4] text-white'
  return 'fill-white stroke-white text-white'
})
</script>

<style scoped>
.lingo-lesson-button:not(.pointer-events-none):hover .lingo-lesson-node {
  transform: scale(1.05);
}
</style>
