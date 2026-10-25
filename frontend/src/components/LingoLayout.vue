<template>
  <div class="lingo-layout flex h-full min-h-0 flex-col bg-white">
    <LingoMobileHeader />
    <!-- Full-width scrollport so the scrollbar sits on the viewport edge.
         The rail and the page column stay one centered row. -->
    <div class="lingo-shell-scroll">
      <div class="lingo-shell-cluster">
        <LingoSidebar class="hidden lg:flex" />
        <main class="lingo-main" :class="fill ? 'flex flex-col' : ''">
          <div class="lingo-main-inner" :class="fill ? 'flex flex-col' : ''">
            <slot />
          </div>
        </main>
      </div>
    </div>
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

<style>
/* Live layout. Tailwind component CSS is easy to miss on a running dev server,
   and the old rules (fixed rail, padded main, inner scroller) must lose. */
.lingo-layout {
  overflow: hidden;
}

.lingo-shell-scroll {
  flex: 1 1 0;
  min-height: 0;
  width: 100%;
  overflow-x: hidden;
  overflow-y: auto;
  /* Reserve the scrollbar lane so content arriving later does not shift the layout. */
  scrollbar-gutter: stable;
  overscroll-behavior-y: contain;
  background: #fff;
}

.lingo-shell-cluster {
  display: grid;
  grid-template-columns: minmax(0, 1056px);
  justify-content: center;
  width: 100%;
}

@media (min-width: 1024px) {
  .lingo-shell-cluster {
    grid-template-columns: 256px minmax(0, 1056px);
    width: max-content;
    max-width: 100%;
    margin-inline: auto;
    justify-content: start;
  }

  .lingo-shell-cluster > .lingo-sidebar-rail {
    position: sticky;
    left: auto;
    top: 0;
    z-index: 40;
    align-self: start;
    height: 100svh;
    width: 256px;
    flex: none;
  }
}

.lingo-shell-cluster > .lingo-main {
  min-width: 0;
  padding-left: 0;
  overflow: visible;
}

.lingo-shell-cluster .lingo-main-inner {
  height: auto;
  min-height: 0;
  max-width: 1056px;
  overflow: visible;
}

.lingo-shell-cluster .lingo-feed-layout {
  flex: none;
  height: auto;
  min-height: 0;
  overflow: visible;
}

.lingo-shell-cluster .lingo-feed-layout__scroll {
  flex: none;
  overflow: visible;
}

/*
 * Mobile: tuck the scrollport 1px under the top bar's bottom border so the
 * sticky feed pin starts behind that border (no sub-pixel gap can show between
 * them). The pin's 1px top padding keeps its content exactly where it was.
 * Desktop has no mobile top bar, so this stays below lg only.
 */
@media (max-width: 1023.98px) {
  .lingo-shell-scroll {
    margin-top: -1px;
  }
  .lingo-shell-scroll .lingo-feed-layout__pin {
    padding-top: 1px;
  }
}

.lingo-feed-layout__pin {
  position: sticky;
  top: 0;
  z-index: 30;
  background: #fff;
}
</style>
