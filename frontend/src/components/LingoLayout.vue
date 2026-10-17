<template>
  <div
    class="lingo-layout flex flex-col bg-slate-50"
    :class="fill ? 'h-full min-h-0' : 'min-h-screen'"
  >
    <LingoMobileHeader />
    <div class="flex min-h-0 flex-1 flex-col pt-14 lg:pt-0">
      <LingoSidebar class="hidden lg:block" />
      <main
        class="lingo-main min-h-0 flex-1 pt-6"
        :class="fill ? 'flex flex-col overflow-hidden' : 'min-h-full'"
      >
        <div
          class="mx-auto w-full max-w-[1056px] px-4 lg:px-6"
          :class="fill ? 'flex h-full min-h-0 flex-1 flex-col' : ''"
        >
          <slot />
        </div>
      </main>
    </div>
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
