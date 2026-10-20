<template>
  <!-- Main/aside split only. Content inset is `.lingo-main-inner` (brandbook §10.3). -->
  <div class="lingo-feed-layout" :class="{ 'lingo-feed-layout--with-header': hasHeader }">
    <div v-if="hasHeader" class="lingo-feed-layout__header-band">
      <div class="lingo-feed-layout__header-row">
        <div class="lingo-feed-layout__header-main">
          <slot name="header" />
        </div>
        <div v-if="showAside" class="lingo-feed-layout__header-aside">
          <LingoUserProgress />
        </div>
      </div>
    </div>

    <div class="lingo-feed-layout__body">
      <aside v-if="showAside" class="lingo-feed-layout__aside">
        <div class="flex flex-col gap-y-4">
          <!-- With levels header: hearts sit in the stripe on lg; keep them here on small screens. -->
          <div :class="hasHeader ? 'lg:hidden' : undefined">
            <LingoUserProgress />
          </div>
          <slot name="aside">
            <StudyStreak class="lingo-card" />
          </slot>
        </div>
      </aside>
      <div class="lingo-feed-layout__main">
        <slot />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, useSlots } from 'vue'
import LingoUserProgress from './LingoUserProgress.vue'
import StudyStreak from './StudyStreak.vue'

defineProps({
  showAside: { type: Boolean, default: true },
})

const slots = useSlots()
const hasHeader = computed(() => !!slots.header)
</script>
