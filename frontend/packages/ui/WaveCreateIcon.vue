<script setup lang="ts">
/**
 * New wave (brand book §5c): Lucide `AudioWaveform` at full width, plus a plus.
 * The plus uses the same bottom-right corner as Lucide `StarPlus`.
 */
defineOptions({ name: 'WaveCreateIcon', inheritAttrs: false })
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
    class="lucide lucide-audio-waveform-plus-icon lucide-audio-waveform-plus"
    aria-hidden="true"
    v-bind="$attrs"
  >
    <g transform="translate(0 1.5) scale(1 0.52)">
      <path
        vector-effect="non-scaling-stroke"
        d="M2 13a2 2 0 0 0 2-2V7a2 2 0 0 1 4 0v13a2 2 0 0 0 4 0V4a2 2 0 0 1 4 0v13a2 2 0 0 0 4 0v-4a2 2 0 0 1 2-2"
      />
    </g>
    <path d="M15 18h6" />
    <path d="M18 15v6" />
  </svg>
</template>
