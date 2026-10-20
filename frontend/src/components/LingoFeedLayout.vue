<template>
  <div class="flex w-full flex-col-reverse gap-8 lg:flex-row-reverse lg:gap-12">
    <aside
      v-if="showAside"
      class="hidden w-full shrink-0 lg:sticky lg:top-6 lg:block lg:w-[368px] lg:self-start"
    >
      <div class="flex flex-col gap-y-4">
        <LingoUserProgress />
        <slot name="aside">
          <LingoPromo />
          <LingoQuestsPanel />
        </slot>
      </div>
    </aside>
    <div class="relative min-w-0 flex-1 pb-10">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import LingoUserProgress from './LingoUserProgress.vue'
import LingoPromo from './LingoPromo.vue'
import LingoQuestsPanel from './LingoQuestsPanel.vue'

defineProps({
  showAside: { type: Boolean, default: true },
})
</script>
