<template>
  <div
    v-if="!locked"
    class="lingo-lesson-button group flex flex-row items-center justify-start gap-2"
    :style="nodeStyle"
  >
    <!-- Level button (clickable link) - no menu inside so it never blocks the click -->
    <RouterLink
      :to="studyUrl"
      class="flex shrink-0 flex-col items-center"
    >
      <!-- Current: "Start" badge + circular progress + button (clone: lesson-button.tsx + button secondary/rounded border-b-8) -->
      <div v-if="current" class="relative h-[102px] w-[102px]">
        <!-- Start label: exact clone - no shadow, border-2, animate-bounce, triangle pointer -->
        <div
          class="absolute -top-6 left-2.5 z-10 animate-bounce rounded-xl border-2 border-slate-200 bg-white px-3 py-2.5 font-bold uppercase tracking-wide text-green-500"
        >
          {{ startLabel }}
          <div
            class="absolute -bottom-2 left-1/2 h-0 w-0 -translate-x-1/2 border-x-8 border-t-8 border-x-transparent border-t-white"
            aria-hidden
          />
        </div>
        <!-- Progress ring (clone: path #4ade80, trail #e5e7eb) + button centered inside -->
        <div class="relative h-[102px] w-[102px]">
          <svg class="absolute inset-0 h-[102px] w-[102px] -rotate-90" viewBox="0 0 102 102">
            <circle
              cx="51"
              cy="51"
              r="43"
              fill="none"
              stroke="#e5e7eb"
              stroke-width="8"
            />
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
            <div
              class="lingo-lesson-button-circle flex h-[70px] w-[70px] flex-shrink-0 items-center justify-center rounded-full border-2 border-b-8 active:border-b-0"
              :class="circleClass"
            >
              <component :is="iconComponent" class="h-10 w-10" :class="iconClass" />
            </div>
          </div>
        </div>
      </div>

      <!-- Not current: just the circular button (clone: same secondary/locked, border-b-8) -->
      <div
        v-else
        class="lingo-lesson-button-circle flex h-[70px] w-[70px] flex-shrink-0 items-center justify-center rounded-full border-2 border-b-8 active:border-b-0"
        :class="circleClass"
      >
        <component :is="iconComponent" class="h-10 w-10" :class="iconClass" />
      </div>
    </RouterLink>

    <!-- Owner menu to the side: trigger + dropdown (does not overlap the level button) -->
    <div
      v-if="showOwnerMenu"
      class="relative z-20 flex shrink-0 items-center"
      @click.stop
    >
      <button
        type="button"
        class="flex h-8 w-8 items-center justify-center rounded-full text-slate-500 transition hover:bg-slate-200 hover:text-slate-700"
        :class="{ 'bg-slate-100 text-slate-700': menuOpen }"
        aria-haspopup="true"
        :aria-expanded="menuOpen"
        aria-label="Level options"
        @click.prevent="menuOpen = !menuOpen"
      >
        <Settings class="h-4 w-4" />
      </button>
      <!-- Dropdown to the right of the gear so the level button stays clickable -->
      <div
        v-if="menuOpen"
        class="absolute left-full top-1/2 z-[50] ml-1 min-w-[180px] -translate-y-1/2 rounded-lg border border-slate-200 bg-white py-1 shadow-lg"
      >
        <slot name="menu" />
      </div>
      <!-- Click outside to close (above page, below dropdown) -->
      <div
        v-if="menuOpen"
        class="fixed inset-0 z-[45]"
        aria-hidden
        @click="menuOpen = false"
      />
    </div>
  </div>
  <!-- Locked with owner menu: show locked circle + settings to the right -->
  <div
    v-else-if="locked && showOwnerMenu"
    class="lingo-lesson-button group flex flex-row items-center justify-start gap-2"
    :style="nodeStyle"
    aria-disabled="true"
  >
    <div
      class="lingo-lesson-button-circle flex h-[70px] w-[70px] flex-shrink-0 items-center justify-center rounded-full border-2 border-neutral-400 border-b-8 bg-neutral-200 pointer-events-none active:border-b-0"
    >
      <component :is="iconComponent" class="h-10 w-10 fill-neutral-400 stroke-neutral-400 text-neutral-400" />
    </div>
    <div
      class="relative z-20 flex shrink-0 items-center"
      @click.stop
    >
      <button
        type="button"
        class="flex h-8 w-8 items-center justify-center rounded-full text-slate-500 transition hover:bg-slate-200 hover:text-slate-700"
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
        class="absolute left-full top-1/2 z-[50] ml-1 min-w-[180px] -translate-y-1/2 rounded-lg border border-slate-200 bg-white py-1 shadow-lg"
      >
        <slot name="menu" />
      </div>
      <div
        v-if="menuOpen"
        class="fixed inset-0 z-[45]"
        aria-hidden
        @click="menuOpen = false"
      />
    </div>
  </div>
  <!-- Locked, no owner menu: circle only -->
  <div
    v-else
    class="lingo-lesson-button flex pointer-events-none"
    :style="nodeStyle"
    aria-disabled="true"
  >
    <div class="lingo-lesson-button-circle flex h-[70px] w-[70px] flex-shrink-0 items-center justify-center rounded-full border-2 border-neutral-400 border-b-8 bg-neutral-200 active:border-b-0">
      <component :is="iconComponent" class="h-10 w-10 fill-neutral-400 stroke-neutral-400 text-neutral-400" />
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { RouterLink } from 'vue-router'
import { Check, Crown, Star, Settings } from 'lucide-vue-next'

const props = defineProps({
  index: { type: Number, required: true },
  totalCount: { type: Number, required: true },
  locked: { type: Boolean, default: false },
  current: { type: Boolean, default: false },
  completed: { type: Boolean, default: false },
  percentage: { type: Number, default: 0 },
  studyUrl: { type: String, default: '' },
  startLabel: { type: String, default: 'Start' },
  showOwnerMenu: { type: Boolean, default: false },
})

const menuOpen = ref(false)

// Zigzag positioning (clone: cycleLength 8, indentationLevel, rightPosition = indentationLevel * 40)
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
  marginRight: `-${rightPositionPx.value}px`,
  marginTop: isFirst.value && !isCompleted.value ? '60px' : '24px',
}))

const iconComponent = computed(() => {
  if (props.completed) return Check
  if (props.index === props.totalCount) return Crown
  return Star
})

// Clone button variants: secondary = green-500/green-600 border-b-8; locked = neutral-200/neutral-400 border-b-8
const circleClass = computed(() => {
  if (props.locked) return 'border-neutral-400 bg-neutral-200'
  return 'border-green-600 bg-green-500 hover:bg-green-500/90 text-white'
})

const iconClass = computed(() => {
  if (props.locked) return 'fill-neutral-400 stroke-neutral-400 text-neutral-400'
  if (props.completed) return 'fill-none stroke-[4] text-white'
  return 'fill-white stroke-white text-white'
})

</script>

<style scoped>
.lingo-lesson-button {
  position: relative;
  display: flex;
}
.lingo-lesson-button-circle {
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}
.lingo-lesson-button:not(.pointer-events-none):hover .lingo-lesson-button-circle {
  transform: scale(1.05);
}
</style>
