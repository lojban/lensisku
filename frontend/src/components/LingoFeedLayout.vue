<template>
  <!--
    Pin header (+ optional toolbar); only `__scroll` moves (full-bleed scrollbar).
    Content inset: `--lingo-feed-inline-pad` on header / toolbar / body (brandbook §8).
  -->
  <div
    class="lingo-feed-layout"
    :class="{
      'lingo-feed-layout--with-header': hasHeader,
      'lingo-feed-layout--with-toolbar': hasToolbar,
    }"
  >
    <div v-if="hasHeader" class="lingo-feed-layout__header-band">
      <div class="lingo-feed-layout__header-row">
        <slot name="header" />
        <div v-if="showAside" class="lingo-feed-layout__header-aside">
          <LingoUserProgress />
        </div>
      </div>
    </div>

    <div v-if="hasToolbar" class="lingo-feed-layout__toolbar">
      <slot name="toolbar" />
    </div>

    <div class="lingo-feed-layout__scroll">
      <div class="lingo-feed-layout__body">
        <aside v-if="showAside" class="lingo-feed-layout__aside">
          <div class="flex flex-col gap-y-4">
            <div v-if="!hasHeader">
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
const hasToolbar = computed(() => !!slots.toolbar)
</script>
