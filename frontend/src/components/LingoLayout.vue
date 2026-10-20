<template>
  <div
    class="lingo-layout flex flex-col bg-white"
    :class="fill ? 'h-full min-h-0' : 'min-h-screen'"
  >
    <LingoMobileHeader />
    <LingoSidebar class="hidden lg:flex" />
    <main class="lingo-main" :class="fill ? 'flex min-h-0 flex-col overflow-hidden' : ''">
      <div class="lingo-main-inner" :class="fill ? 'min-h-0 flex-1' : ''">
        <slot />
      </div>
    </main>
    <!-- Bottom tab bar omitted: Learn/Courses/Leaderboard live in the sidebar / drawer. -->
  </div>
</template>

<script setup lang="ts">
import LingoMobileHeader from './LingoMobileHeader.vue'
import LingoSidebar from './LingoSidebar.vue'

defineProps({
  fill: {
    type: Boolean,
    default: false,
  },
})
</script>
