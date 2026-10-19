<template>
  <header class="lingo-mobile-header">
    <button
      type="button"
      class="lingo-icon-btn lingo-icon-btn--on-green"
      :aria-label="t('toggleMenu')"
      :aria-expanded="showSidebar"
      @click="showSidebar = true"
    >
      <Menu class="h-6 w-6" />
    </button>
    <RouterLink to="/lingo/courses" class="flex items-center gap-2 no-underline">
      <GraduationCap class="h-8 w-8 text-white" />
      <span class="text-xl font-extrabold tracking-wide text-white">{{ t('lingo.appName') }}</span>
    </RouterLink>
    <Teleport to="body">
      <Transition name="lingo-drawer">
        <div
          v-if="showSidebar"
          class="lingo-drawer-backdrop"
          aria-hidden
          @click="showSidebar = false"
        />
      </Transition>
      <Transition name="lingo-drawer-panel">
        <div v-if="showSidebar" class="lingo-drawer-panel" role="dialog" aria-modal="true">
          <button
            type="button"
            class="lingo-icon-btn lingo-icon-btn--muted absolute right-3 top-3 z-10"
            aria-label="Close"
            @click="showSidebar = false"
          >
            <X class="h-5 w-5" />
          </button>
          <LingoSidebar @navigate="showSidebar = false" />
        </div>
      </Transition>
    </Teleport>
  </header>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Menu, X, GraduationCap } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import LingoSidebar from './LingoSidebar.vue'

const { t } = useI18n()

const showSidebar = ref(false)
</script>

<style scoped>
.lingo-drawer-enter-active,
.lingo-drawer-leave-active {
  transition: opacity 0.2s ease;
}
.lingo-drawer-enter-from,
.lingo-drawer-leave-to {
  opacity: 0;
}
.lingo-drawer-panel-enter-active,
.lingo-drawer-panel-leave-active {
  transition: transform 0.2s ease;
}
.lingo-drawer-panel-enter-from,
.lingo-drawer-panel-leave-to {
  transform: translateX(-100%);
}
</style>
