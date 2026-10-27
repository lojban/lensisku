<template>
  <div
    v-if="!locked"
    class="lingo-lesson-button group relative flex flex-row items-center justify-start gap-2"
    :data-level-id="levelId || undefined"
    :style="nodeStyle"
  >
    <a
      :href="studyUrl"
      class="flex shrink-0 flex-col items-center no-underline"
      :aria-label="accessibleName"
      :title="strengthText || undefined"
      :aria-busy="showSpinner || undefined"
      @click="onStudyClick"
    >
      <div v-if="current" class="relative h-[102px] w-[102px]">
        <div
          v-if="!showSpinner"
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
              class="lingo-lesson-progress transition-all duration-300"
              stroke-width="8"
              stroke-linecap="round"
              stroke-dasharray="270.2"
              :stroke-dashoffset="270.2 - (270.2 * Math.min(100, percentage)) / 100"
            />
          </svg>
          <div class="absolute inset-0 flex items-center justify-center">
            <div class="lingo-lesson-node lingo-lesson-node--open">
              <Loader
                v-if="showSpinner"
                class="h-10 w-10 animate-spin text-white"
                aria-hidden="true"
              />
              <component :is="iconComponent" v-else class="h-10 w-10" :class="iconClass" />
            </div>
          </div>
        </div>
      </div>
      <div v-else class="lingo-lesson-node lingo-lesson-node--open">
        <Loader v-if="showSpinner" class="h-10 w-10 animate-spin text-white" aria-hidden="true" />
        <component :is="iconComponent" v-else class="h-10 w-10" :class="iconClass" />
      </div>
      <span v-if="label" class="lingo-lesson-label" :title="label">{{ label }}</span>
    </a>
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
    :data-level-id="levelId || undefined"
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
    :data-level-id="levelId || undefined"
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
import { Check, Crown, Droplet, Star, Settings, Loader } from '@lucide/vue'

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
  /** Stable id for scroll-into-view from study finish (`?focusLevelId=`). */
  levelId: { type: [Number, String], default: '' },
  showOwnerMenu: { type: Boolean, default: false },
  /** Parent is navigating into this lesson (header Continue / sibling click). */
  pending: { type: Boolean, default: false },
  /** Level asks for practice (cards due or strength below threshold). */
  needsWatering: { type: Boolean, default: false },
  /** Localized "Strength 70% · 5 cards due" for tooltip and screen readers. */
  strengthText: { type: String, default: '' },
})

const emit = defineEmits<{ start: [] }>()

const menuOpen = ref(false)
/** Sync paint on click before parent `pending` / route freeze. */
const starting = ref(false)
const showSpinner = computed(() => props.pending || starting.value)

function onStudyClick(event: MouseEvent) {
  if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || event.button !== 0) {
    return
  }
  event.preventDefault()
  if (showSpinner.value || !props.studyUrl) return
  starting.value = true
  emit('start')
}

const accessibleName = computed(() => {
  if (props.label) return [props.label, props.strengthText].filter(Boolean).join(' · ')
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

/**
 * Hue for this node. A hash of the ladder index.
 * Orange (~30°), amber (~45°), and deep yellow (~55°) are in range.
 * Deep blue (215°–254°) is skipped; sky blue and indigo stay.
 */
function hueFromIndex(index: number): number {
  let h = (index + 1) >>> 0
  h = Math.imul(h ^ (h >>> 16), 0x7feb352d)
  h = Math.imul(h ^ (h >>> 15), 0x846ca68b)
  h = (h ^ (h >>> 16)) >>> 0
  const deepBlueStart = 215
  const deepBlueGap = 40
  const span = h % (360 - deepBlueGap)
  return span < deepBlueStart ? span : span + deepBlueGap
}

const nodeStyle = computed(() => ({
  right: `${rightPositionPx.value}px`,
  // Extra vertical room so `.lingo-lesson-label` (up to 3 lines) never crowds the next node.
  marginTop: isFirst.value && !isCompleted.value ? '60px' : '36px',
  '--lingo-node-h': String(hueFromIndex(props.index)),
}))

/** Completed but memory decayed: the node shows a droplet instead of the checkmark (Duolingo
 *  "cracked skill" idea). Completion and unlocking are unchanged; this is display only. */
const wilted = computed(() => props.completed && props.needsWatering)

const iconComponent = computed(() => {
  if (wilted.value) return Droplet
  if (props.completed) return Check
  if (props.index === props.totalCount) return Crown
  return Star
})

const iconClass = computed(() => {
  if (props.locked) return 'fill-neutral-400 stroke-neutral-400 text-neutral-400'
  if (wilted.value) return 'fill-white/30 stroke-[2.5] text-white'
  if (props.completed) return 'fill-none stroke-[4] text-white'
  return 'fill-white stroke-white text-white'
})
</script>

<style scoped>
.lingo-lesson-button:not(.pointer-events-none):hover .lingo-lesson-node {
  transform: scale(1.05);
}

/* Hue is `--lingo-node-h` on the button (hash of the ladder index). */
.lingo-lesson-node--open {
  background-color: hsl(var(--lingo-node-h, 142) 71% 45%);
  border-color: hsl(var(--lingo-node-h, 142) 76% 36%);
}

.lingo-lesson-button:not(.pointer-events-none):hover .lingo-lesson-node--open {
  background-color: hsl(var(--lingo-node-h, 142) 71% 40%);
}

/* Strength meter under the node: droplet when the level needs watering, then a 0–100 bar. */

.lingo-lesson-progress {
  stroke: hsl(var(--lingo-node-h, 142) 71% 60%);
}
</style>
