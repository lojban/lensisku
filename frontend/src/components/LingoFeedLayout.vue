<template>
  <!-- Main/aside split only. Content inset is `.lingo-main-inner` (brandbook §10.3). -->
  <div class="lingo-feed-layout">
    <aside v-if="showAside" class="lingo-feed-layout__aside">
      <div class="flex flex-col gap-y-4">
        <LingoUserProgress />
        <slot name="aside">
          <LingoPromo />
          <LingoQuestsPanel />
        </slot>
      </div>
    </aside>
    <div class="lingo-feed-layout__main">
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
