<template>
  <!-- Same chrome as App.vue header: logo left, drawer toggle, logout only on the right. -->
  <header class="lingo-mobile-header">
    <div class="lingo-mobile-header-inner">
      <div class="flex h-14 items-center justify-between sm:h-12">
        <div class="flex items-center">
          <button
            type="button"
            class="z-20 cursor-pointer rounded-md p-3 text-gray-600 transition-colors duration-200 hover:bg-gray-100"
            :aria-label="t('toggleMenu')"
            :aria-expanded="showSidebar"
            @click.stop="showSidebar = !showSidebar"
          >
            <Menu v-if="!showSidebar" class="h-6 w-6" />
            <X v-else class="h-6 w-6" />
          </button>
          <a
            :href="homePath"
            class="input-field flex cursor-pointer select-none items-center gap-2 !h-10 !py-1 no-underline"
            @click.prevent="handleLogoClick"
          >
            <div class="flex h-8 w-8 shrink-0 items-center justify-center -skew-x-12">
              <!-- eslint-disable vue/no-v-html -->
              <div
                role="img"
                :aria-label="t('logoText')"
                class="logo-svg-container"
                v-html="logoSvgRaw"
              ></div>
              <!-- eslint-enable vue/no-v-html -->
            </div>
            <span class="select-none font-medium italic leading-none">{{ t('logoText') }}</span>
          </a>
        </div>
        <div class="flex items-center space-x-1 sm:space-x-2">
          <template v-if="!auth.state.isLoading && auth.state.isLoggedIn">
            <Button variant="topbar" @click="handleLogout">
              <LogOut class="h-5 w-5" />
              <span>{{ t('nav.logout') }}</span>
            </Button>
          </template>
        </div>
      </div>
    </div>
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
          <div class="lingo-drawer-toolbar">
            <button
              type="button"
              class="lingo-icon-btn lingo-icon-btn--muted"
              :aria-label="t('modal.close')"
              @click="showSidebar = false"
            >
              <X class="h-5 w-5" />
            </button>
          </div>
          <LingoSidebar @navigate="showSidebar = false" />
        </div>
      </Transition>
    </Teleport>
  </header>
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { LogOut, Menu, X } from '@lucide/vue'
import { Button } from '@packages/ui'
import { useI18n } from 'vue-i18n'
import LingoSidebar from './LingoSidebar.vue'
import logoSvgRaw from '../../public/assets/icons/favicon.svg?raw'
import { localeCaptureGroupRegex } from '../config/locales'
import { useAuth } from '@/composables/useAuth'

const { t, locale } = useI18n()
const route = useRoute()
const router = useRouter()
const auth = useAuth()

const showSidebar = ref(false)

const homePath = computed(() => {
  const localeMatch = route.path.match(localeCaptureGroupRegex)
  return `/${localeMatch ? localeMatch[1] : (locale.value as string) || 'en'}`
})

async function handleLogoClick() {
  showSidebar.value = false
  await router.push({ path: homePath.value, query: {} })
  await nextTick()
  document.querySelector('.feed-page__body')?.scrollTo({ top: 0, behavior: 'instant' })
}

function handleLogout() {
  auth.logout()
  showSidebar.value = false
  router.push('/login')
}
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
