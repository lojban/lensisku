<script setup lang="ts">
/**
 * Several favorites (brand book §5c): a star, with the left half of that star beside it.
 * One favorite is `StarIcon`. Adding to a collection is `StarCreateIcon`.
 *
 * The two stars are scaled so both fit in the 24px Lucide box. Stroke width on each
 * path is divided by that scale, so the painted weight matches other Lucide icons.
 * `stroke-width`, `absolute-stroke-width`, and `size` work the same way as on Lucide icons.
 */
import { computed, useId } from 'vue'

defineOptions({ name: 'StarsIcon', inheritAttrs: false })

const props = defineProps<{
  strokeWidth?: number | string
  absoluteStrokeWidth?: boolean
  size?: number | string
}>()

const STAR =
  'M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z'

/**
 * Lucide star is ~20 wide. Both marks stay inside the 24 box, and the gap between
 * the half and the full star is wide enough that the strokes do not cross.
 * The half sits a little above the full star.
 */
const SCALE = 0.7
const HALF_W = 10.002 * SCALE
const PAD = 1.2
const LEFT_CENTER = PAD + HALF_W
const RIGHT_CENTER = 24 - PAD - HALF_W
const LEFT_Y = 10.85

const clipId = `stars-clip-${useId().replace(/[^a-zA-Z0-9_-]/g, '')}`

const stroke = computed(() => {
  const raw = props.strokeWidth == null || props.strokeWidth === '' ? 2 : Number(props.strokeWidth)
  const width = Number.isFinite(raw) ? raw : 2
  if (!props.absoluteStrokeWidth) return width
  const sizeRaw = props.size == null || props.size === '' ? 24 : Number(props.size)
  const size = Number.isFinite(sizeRaw) && sizeRaw > 0 ? sizeRaw : 24
  return (width * 24) / size
})

/** Group `scale()` shrinks strokes; undo that so the weight stays in 24px Lucide units. */
const pathStroke = computed(() => stroke.value / SCALE)

const box = computed(() => props.size ?? 24)
</script>

<template>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    :width="box"
    :height="box"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    :stroke-width="stroke"
    stroke-linecap="round"
    stroke-linejoin="round"
    class="lucide lucide-stars-icon lucide-stars"
    aria-hidden="true"
    v-bind="$attrs"
  >
    <defs>
      <clipPath :id="clipId">
        <rect x="0" y="0" :width="LEFT_CENTER" height="24" />
      </clipPath>
    </defs>
    <g :clip-path="`url(#${clipId})`">
      <g :transform="`translate(${LEFT_CENTER} ${LEFT_Y}) scale(${SCALE}) translate(-12 -12)`">
        <path :d="STAR" :stroke-width="pathStroke" />
      </g>
    </g>
    <g :transform="`translate(${RIGHT_CENTER} 12) scale(${SCALE}) translate(-12 -12)`">
      <path :d="STAR" :stroke-width="pathStroke" />
    </g>
  </svg>
</template>
