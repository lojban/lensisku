<script setup lang="ts">
/**
 * Several favorites (brand book §5c): a star, with the left half of that star beside it.
 * One favorite is `StarIcon`. Adding to a collection is `StarCreateIcon`.
 *
 * Layout knobs (both stars share the same scale):
 * - scale 0.703, edge pad 1.2 → gap ≈ 0.5 between half and full star
 * - left half center y 11.15 (a little above the full star)
 * - path stroke-width 2.845 (= 2 / 0.703) so weight matches Lucide after scale()
 */
defineOptions({ name: 'StarsIcon', inheritAttrs: false })
</script>

<template>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    class="lucide lucide-stars-icon lucide-stars"
    aria-hidden="true"
    v-bind="$attrs"
  >
    <!-- half: nested svg clips to the left of the full star -->
    <svg x="0" y="0" width="8.231" height="24" overflow="hidden">
      <g transform="translate(8.231 11.15) scale(0.703) translate(-12 -12)">
        <path
          stroke-width="2.845"
          d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"
        />
      </g>
    </svg>
    <g transform="translate(15.769 12) scale(0.703) translate(-12 -12)">
      <path
        stroke-width="2.845"
        d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"
      />
    </g>
  </svg>
</template>
