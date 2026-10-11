<template>
  <span class="relative inline-block h-6 w-6 shrink-0" aria-hidden="true">
    <ThumbsDown class="absolute -bottom-px right-0 h-3.5 w-3.5 fill-white" :stroke-width="1.6" />
    <ThumbsUp class="absolute left-0 -top-px h-4 w-4 fill-white" :stroke-width="1.6" />
  </span>
</template>

<script setup lang="ts">
import { ThumbsDown, ThumbsUp } from '@lucide/vue'
</script>
