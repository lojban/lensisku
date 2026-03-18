<template>
  <header class="lingo-mobile-header fixed left-0 right-0 top-0 z-50 flex h-14 items-center border-b border-white/20 bg-green-500 px-4 lg:hidden">
    <button
      type="button"
      class="flex h-10 w-10 items-center justify-center rounded-lg text-white transition hover:bg-white/20"
      aria-label="Menu"
      @click="showSidebar = true"
    >
      <Menu class="h-6 w-6" />
    </button>
    <RouterLink to="/lingo" class="ml-3 flex items-center gap-2">
      <GraduationCap class="h-8 w-8 text-white" />
      <span class="text-xl font-bold text-white">{{ t('lingo.appName') }}</span>
    </RouterLink>

    <!-- Mobile sidebar overlay -->
    <Teleport to="body">
      <Transition name="lingo-drawer">
        <div
          v-if="showSidebar"
          class="fixed inset-0 z-[100] bg-black/50 lg:hidden"
          aria-hidden
          @click="showSidebar = false"
        />
      </Transition>
      <Transition name="lingo-drawer-panel">
        <aside
          v-if="showSidebar"
          class="fixed left-0 top-0 z-[101] h-full w-64 overflow-y-auto bg-white shadow-xl lg:hidden"
        >
          <div class="flex items-center justify-between border-b border-slate-200 p-4">
            <span class="text-lg font-bold text-green-600">Menu</span>
            <button
              type="button"
              class="rounded-lg p-2 text-slate-500 hover:bg-slate-100"
              aria-label="Close"
              @click="showSidebar = false"
            >
              <X class="h-5 w-5" />
            </button>
          </div>
          <div class="p-4" @click="showSidebar = false">
            <LingoSidebar :hide-logo="true" />
          </div>
        </aside>
      </Transition>
    </Teleport>
  </header>
</template>

<script setup>
import { ref } from 'vue'
import { Menu, X, GraduationCap } from 'lucide-vue-next'
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
