<template>
  <BackgroundComponent />
  <div v-if="isWinterSeason" class="snowflakes" aria-hidden="true">
    <div v-for="(flake, index) in snowflakes" :key="index" class="sihesle"
      :style="{ left: `${flake.left}%`, 'animation-delay': `${flake.delay1}s, ${flake.delay2}s` }">
      {{ index % 2 === 0 ? '❅' : '❆' }}
    </div>
  </div>
  <div v-if="isWinterSeason && showPyro" class="pyro" />

  <div v-if="showTestDataWarning"
    class="select-none top-14 md:top-12 opacity-80 fixed w-fit mx-auto left-0 right-0 z-10 text-xs py-0 px-2 text-center border bg-red-100 border-red-200">
    {{ $t('testDataWarning') }}
    <a :href="discordChatUrl" target="_blank" rel="noopener noreferrer"
      class="text-blue-500 hover:text-red-800 underline">
      {{ $t('discord') }}
    </a>
  </div>
  <!-- Mobile-optimized header -->
  <header class="bg-white border-b border-gray-200 sticky top-0 z-40">
    <div class="px-1 sm:px-2 max-w-4xl mx-auto">
      <!-- Main header content -->
      <div class="flex items-center justify-between h-14 sm:h-12">
        <!-- Logo + Toggle Menu Button -->
        <div class="flex items-center">
          <button class="sm:hidden p-3 text-gray-600 hover:bg-gray-100 rounded-md z-15" :aria-label="$t('toggleMenu')"
            @click.stop="isMenuOpen = !isMenuOpen">
            <Menu v-if="!isMenuOpen" class="h-6 w-6" />
            <X v-else class="h-6 w-6" />
          </button>

          <!-- Logo - Always visible -->
          <NavLink to="/" class="flex items-center space-x-2 px-2 sm:px-3 py-1.5" @click="triggerPyro">
            <div v-html="logoSvgRaw" role="img" :aria-label="$t('logoText')" class="w-7 h-7" :class="{ 'animate-rotate-3d': showPyro }"></div>
            <span class="select-none font-medium">{{ $t('logoText') }}</span>
          </NavLink>
        </div>

        <!-- Desktop Navigation - Hidden on mobile -->
        <nav class="hidden sm:ml-4 sm:flex items-center space-x-0 md:space-x-1 lg:space-x-2">
          <NavLink to="/collections" class="navbar-item">
            <GalleryVerticalEnd class="h-5 w-5" />
            {{ $t('nav.courses') }}
          </NavLink>
          <NavLink to="/recent" class="navbar-item">
            <Clock4 class="h-5 w-5" />
            {{ $t('nav.recent') }}
          </NavLink>
          <div class="relative group">
            <button class="navbar-item">
              <span class="hidden lg:inline"> {{ $t('nav.more') }} </span>
              <ChevronDown class="h-5 w-5" :stroke-width="2.5" :absolute-stroke-width="true" />
            </button>
            <div
              class="absolute hidden group-hover:flex flex-col mt-0 bg-white border border-gray-200 rounded-md shadow-lg z-50 p-1 w-auto max-w-96">
              <NavLink v-if="auth.state.isLoggedIn" to="/users" class="navbar-item justify-start py-2" @click="isMenuOpen = false">
                <Users class="h-4 w-4" />
                {{ auth.state.authorities?.includes('manage_roles') ? $t('nav.iamUsers') : $t('nav.users') }}
              </NavLink>
              <NavLink to="/languages" class="navbar-item justify-start py-2">
                <Globe class="h-4 w-4" />
                {{ $t('nav.languages') }}
              </NavLink>
              <NavLink v-if="!auth.state.isLoggedIn" to="/export/cached" class="navbar-item justify-start py-2">
                <Download class="h-4 w-4" />
                {{ $t('nav.cachedExports') }}
              </NavLink>
              <NavLink v-if="auth.state.isLoggedIn" to="/export" class="navbar-item justify-start py-2">
                <Upload class="h-4 w-4" />
                {{ $t('nav.export') }}
              </NavLink>
              <NavLink v-if="auth.state.isLoggedIn && auth.state.authorities?.includes('bulk_import')" to="/bulk-import"
                class="navbar-item justify-start py-2">
                <Download class="h-4 w-4" />
                {{ $t('nav.bulkImport') }}
              </NavLink>
            </div>
          </div>
        </nav>

        <!-- Auth Buttons - Optimized for mobile -->
        <div class="flex items-center space-x-2">
          <!-- Only show auth buttons when loading is complete -->
          <template v-if="!auth.state.isLoading">
            <template v-if="auth.state.isLoggedIn">
              <NavLink v-if="auth.state.isLoggedIn" to="/reactions" class="navbar-item">
                <BookmarkCheck class="h-5 w-5" />
              <span class="hidden sm:inline">{{ $t('nav.myActivity') }}</span>
            </NavLink>
            <NavLink to="/profile" class="navbar-item">
              <User class="h-5 w-5" />
              <span class="hidden sm:inline">{{ auth.state.username }}</span>
            </NavLink>
            <button class="navbar-item hidden sm:flex" @click="handleLogout">
              <LogOut class="h-5 w-5" />
              <span class="hidden md:inline">{{ $t('nav.logout') }}</span>
            </button>
          </template>
          <template v-else>
            <NavLink to="/signup" class="btn-signup">
              <UserPlus class="h-5 w-5" />
              <span class="hidden sm:inline">{{ $t('nav.signUp') }}</span>
            </NavLink>
            <NavLink to="/login" class="btn-login">
              <LogIn class="h-5 w-5" />
                <span class="hidden sm:inline">{{ $t('nav.logIn') }}</span>
              </NavLink>
            </template>
          </template>
        </div>
      </div>

      <!-- Mobile Navigation Menu -->
      <div v-show="isMenuOpen" class="fixed sm:hidden top-14 left-0 right-0 bg-white shadow-md py-2 space-y-1 z-50">
        <NavLink to="/collections"
          class="flex items-center px-4 py-2 text-base text-gray-600 hover:bg-gray-100 rounded-md transition-colors gap-2"
          @click="isMenuOpen = false">
          <GalleryVerticalEnd class="h-5 w-5" />
          {{ $t('nav.courses') }}
        </NavLink>

        <NavLink to="/recent"
          class="flex items-center px-4 py-2 text-base text-gray-600 hover:bg-gray-100 rounded-md transition-colors gap-2"
          @click="isMenuOpen = false">
          <Clock4 class="h-5 w-5" />
          {{ $t('mobileNav.recentChanges') }}
        </NavLink>

        <NavLink v-if="auth.state.isLoggedIn" to="/users"
          class="flex items-center px-4 py-2 text-base text-gray-600 hover:bg-gray-100 rounded-md transition-colors gap-2"
          @click="isMenuOpen = false">
          <Users class="h-5 w-5" />
          {{ auth.state.authorities?.includes('manage_roles') ? $t('nav.iamUsers') : $t('nav.users') }}
        </NavLink>

        <NavLink to="/languages"
          class="flex items-center px-4 py-2 text-base text-gray-600 hover:bg-gray-100 rounded-md transition-colors gap-2"
          @click="isMenuOpen = false">
          <Globe class="h-5 w-5" />
          {{ $t('nav.languages') }}
        </NavLink>

        <NavLink v-if="!auth.state.isLoggedIn" to="/export/cached"
          class="flex items-center px-4 py-2 text-base text-gray-600 hover:bg-gray-100 rounded-md transition-colors gap-2"
          @click="isMenuOpen = false">
          <Download class="h-5 w-5" />
          {{ $t('nav.cachedExports') }}
        </NavLink>

        <NavLink v-if="auth.state.isLoggedIn" to="/export"
          class="flex items-center px-4 py-2 text-base text-gray-600 hover:bg-gray-100 rounded-md transition-colors gap-2"
          @click="isMenuOpen = false">
          <Upload class="h-5 w-5" />
          {{ $t('nav.export') }}
        </NavLink>

        <NavLink v-if="auth.state.isLoggedIn && auth.state.authorities?.includes('bulk_import')" to="/bulk-import"
          class="flex items-center px-4 py-2 text-base text-gray-600 hover:bg-gray-100 rounded-md transition-colors gap-2"
          @click="isMenuOpen = false">
          <Download class="h-5 w-5" />
          {{ $t('nav.bulkImport') }}
        </NavLink>

        <div v-if="auth.state.isLoggedIn" class="my-1 border-t border-gray-200"></div>

        <button
          v-if="auth.state.isLoggedIn"
          class="flex items-center px-4 py-2 text-base text-[#007bff] hover:bg-gray-100 rounded-md transition-colors gap-2 w-full text-left"
          @click="handleLogout">
          <LogOut class="h-5 w-5" />
          {{ $t('nav.logout') }}
        </button>
      </div>
    </div>
  </header>
  <!-- Global Error Display -->
  <div class="flex justify-center">
    <div v-if="error?.message" class="w-full max-w-lg px-4">
      <Error v-if="error?.message" :message="error.message" :details="error.details" @close="clearError" />
    </div>
  </div>

  <!-- Main content -->
  <main class="main-content">
    <div class="p-3 pb-12 max-w-4xl mx-auto min-h-[calc(100vh-12rem)] relative" id="main-child">
      <router-view v-slot="{ Component, route }">
        <component :is="Component" v-bind="route.meta.props" v-on="route.name === 'home'
          ? {
            search: performSearch,
            'view-message': viewMessage,
            'view-thread': viewThread,
          }
          : {}
          " />
      </router-view>

      <footer class="mt-10 text-center text-xs text-gray-500 leading-relaxed">
        {{ $t('footer.publicDomainNotice') }}
      </footer>
    </div>
  </main>

  <!-- Floating Action Button -->
  <div class="max-w-4xl mx-auto relative" v-if="auth.state.isLoggedIn && route.name !== 'flashcard-study'">
    <div
      class="fixed md:absolute bottom-6 right-4 md:bottom-8 md:right-8 lg:-right-4 lg:-mr-4 z-50 flex flex-col items-end gap-3">
      <button class="p-2 flex items-center justify-center w-[52px] h-[52px] btn-aqua-emerald rounded-corner"
        @click="showActionModal = true">
        <Plus class="h-6 w-6 transition-all duration-200" :class="{ 'rotate-45': showActionModal }" />
      </button>
    </div>
  </div>

  <!-- Action Modal -->
  <ModalComponent :show="showActionModal" @close="showActionModal = false" :title="$t('fab.actionsTitle', 'Actions')">
    <div class="flex flex-wrap gap-5 p-4 justify-center">
      <IconButton v-if="auth.state.isLoggedIn" :label="$t('fab.addDefinition')" icon-classes="h-5 w-5"
        button-classes="btn-aqua-green h-12 text-base !w-64 !rounded-full" @click="handleNewDefinition" />
      <IconButton v-if="auth.state.isLoggedIn" :label="$t('fab.newDiscussion')"
        button-classes="btn-aqua-purple h-12 text-base !w-64 !rounded-full" @click="handleNewFreeThread">
        <template #icon>
          <AudioWaveform class="h-5 w-5" />
        </template>
      </IconButton>
    </div>
  </ModalComponent>

  <FooterComponent />
</template>

<script setup>
import {
  Users,
  Globe,
  Download,
  Upload,
  LogIn,
  LogOut,
  UserPlus,
  User,
  ChevronDown,
  X,
  Shield,
  Plus,
  AudioWaveform,
  BookmarkCheck,
  Clock4,
  GalleryVerticalEnd
} from 'lucide-vue-next'
import { Menu } from 'lucide-vue-next' // Explicitly import Menu if it was missed by auto-sort
import { ref, onMounted, watch, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'

import { useI18n } from 'vue-i18n'

import Error from '@/components/Error.vue'
import CoursesIcon from '@/components/icons/CoursesIcon.vue'
import IconButton from '@/components/icons/IconButton.vue'

import BackgroundComponent from './components/BackgroundComponent.vue'
import FooterComponent from './components/FooterComponent.vue'
import NavLink from './components/NavLink.vue'
import ModalComponent from '@/components/ModalComponent.vue'
import { provideAuth } from './composables/useAuth'
import { provideError } from './composables/useError'
import { localeCaptureGroupRegex, supportedLocales } from './config/locales';

import logoSvgRaw from '@/assets/icons/favicon.svg?raw';

const i18n = useI18n() // Use i18n composable
const $t = i18n.t
const $locale = i18n.locale
const router = useRouter()
const route = useRoute()
const searchQuery = ref('')
const searchMode = ref('messages')
const auth = provideAuth()
const { error, clearError } = provideError()
const isMenuOpen = ref(false)
const showActionModal = ref(false)
const showPyro = ref(false)
const discordChatUrl = 'https://discord.gg/4KhzRzpmVr'
const showTestDataWarning = import.meta.env.VITE_SHOW_TEST_DATA_WARNING === 'true'

const rnd = (max, min = 1) => ((Math.random() * max) / min).toFixed(2)

const isWinterSeason = computed(() => {
  const now = new Date()
  const year = now.getFullYear()
  const startDate = new Date(year, 11, 5) // December 5
  const endDate = new Date(year + 1, 0, 1) // January 1 next year
  return now >= startDate && now < endDate
})

const generateSnowflakes = () =>
  Array(6)
    .fill(null)
    .map(() => ({
      left: rnd(100),
      delay1: rnd(30),
      delay2: rnd(3),
    }))

const snowflakes = ref(generateSnowflakes())

const handleNewDefinition = () => {
  router.push('/valsi/add')
  showActionModal.value = false // Close modal
}

const handleNewFreeThread = () => {
  router.push('/comments/new-thread')
  showActionModal.value = false // Close modal
}

const triggerPyro = () => {
  showPyro.value = !showPyro.value
  setTimeout(() => {
    showPyro.value = false
  }, 3000)
}

const performSearch = ({ query, mode }) => {
  searchQuery.value = query
  searchMode.value = mode
  updateRouteParams()
}

const updateRouteParams = () => {
  router.push({
    query: {
      q: searchQuery.value || undefined,
      mode: searchMode.value,
    },
  })
}

const syncFromRoute = () => {
  searchQuery.value = route.query.q || ''
  searchMode.value = route.query.mode || 'messages'
}

const handleLogout = () => {
  auth.logout()
  router.push('/login')
  isMenuOpen.value = false
}

const viewMessage = (messageId) => {
  router.push({ name: 'message', params: { id: messageId } })
}

const viewThread = (subject) => {
  router.push({
    name: 'thread',
    params: { subject: encodeURIComponent(subject) },
  })
}

// Close mobile menu and clear errors on route change
watch(
  () => route.fullPath,
  () => {
    isMenuOpen.value = false
    clearError()
  }
)

// Handle click outside mobile menu to close it
const handleClickOutside = (event) => {
  const header = document.querySelector('header')
  if (isMenuOpen.value && header && !header.contains(event.target)) {
    isMenuOpen.value = false
  }
}

onMounted(() => {
  // Close mobile menu when window resizes to desktop width
  const handleResize = () => {
    if (window.innerWidth >= 640) {
      // sm breakpoint
      isMenuOpen.value = false
    }
  }

  window.addEventListener('resize', handleResize)
  handleResize() // Check initial size

  document.addEventListener('click', handleClickOutside)
  auth.checkAuthStatus()

  // Cleanup
  return () => {
    window.removeEventListener('resize', handleResize)
    document.removeEventListener('click', handleClickOutside)
  }
})

watch(() => route.query, syncFromRoute, { deep: true })

// Watch route changes to update $locale
watch(
  () => router.currentRoute.value.path,
  (path) => {
    const localeMatch = path.match(localeCaptureGroupRegex)
    if (localeMatch && localeMatch[1] !== $locale.value) {
      $locale.value = localeMatch[1]
    }
  },
  { immediate: true }
)

// Also set initial $locale based on route on mount
onMounted(() => {
  const path = router.currentRoute.value.path
  const localeMatch = path.match(localeCaptureGroupRegex)
  if (localeMatch) {
    $locale.value = localeMatch[1]
  }
})

const setLocale = (newLocale) => {
  const currentRoute = router.currentRoute.value
  const currentPath = currentRoute.path
  // Get the current $locale from the path
  const currentLocaleMatch = currentPath.match(localeCaptureGroupRegex)
  const currentLocale = currentLocaleMatch ? currentLocaleMatch[1] : ''
  
  if (currentLocale && supportedLocales.includes(newLocale)) {
    // Replace the $locale prefix in the current path
    const newPath = currentPath.replace(`/${currentLocale}`, `/${newLocale}`)
    router.push(newPath)
  } else {
    // Fallback: just add the new $locale prefix
    router.push(`/${newLocale}`)
  }
}
</script>

<style>
body {
  min-height: 100vh;
}

body::before {
  content: '';
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.7);
  z-index: -1;
}

header,
footer {
  background: repeating-linear-gradient(#f8f8f8, #f8f8f8 4px, #ffffff 4px, #ffffff 8px);
  place-content: center center;
}

.main-content {
  height: calc(100vh - 57px - 24px);
  padding-bottom: 24px;
}

.main-content>* {
  @apply bg-transparent md:bg-zinc-50/75 md:border-x;
  min-height: calc(100% - 24px);
}

@media (min-width: 640px) {
  .main-content {
    height: calc(100vh - 49px - 24px);
  }
}

/* Aqua Scrollbar Styles */
::-webkit-scrollbar {
  width: 15px;
  height: 15px;
}

::-webkit-scrollbar-track {
  background: rgb(236, 236, 236);
  border-radius: 8px;
}

::-webkit-scrollbar-thumb {
  background-image: linear-gradient(to right,
      #375abb 0%,
      #8bb4e3 21%,
      #84b4e9 38%,
      #3f8ae0 40%,
      #95e0ff 86%,
      #63abf2 100%);
  box-shadow:
    inset 0 1px #0028ab,
    inset 0 -1px #0028ab,
    inset 1px 0 #0028ab,
    inset -1px 0px #0028ab;
  border-radius: 8px;
  border: 2px solid rgb(236, 236, 236);
}

::-webkit-scrollbar-thumb:horizontal {
  background-image: linear-gradient(to bottom,
      #375abb 0%,
      #8bb4e3 21%,
      #84b4e9 36%,
      #3f8ae0 44%,
      #95e0ff 86%,
      #63abf2 100%);
  border-top-width: 0px;
  border-bottom-width: 0px;
  border-radius: 6px;
  box-shadow:
    inset 0 1px #0028ab,
    inset 0 -1px #0028ab,
    inset 1px 0 #0028ab,
    inset -1px 0px #0028ab;
  border-radius: 8px;
  border: 3px solid rgb(236, 236, 236);
}

::-webkit-scrollbar-thumb:hover {
  background-image: linear-gradient(to left,
      #375abb 0%,
      #8bb4e3 21%,
      #84b4e9 38%,
      #3f8ae0 40%,
      #95e0ff 86%,
      #63abf2 100%);
  border-radius: 6px;
  box-shadow:
    inset 0 1px #0028ab,
    inset 0 -1px #0028ab,
    inset 1px 0 #0028ab,
    inset -1px 0px #0028ab;
  border-radius: 8px;
  /* background: linear-gradient(to right, #89b6ff 0%, #6da6ff 100%); */
}

::-webkit-scrollbar-thumb:hover:horizontal {
  background-image: linear-gradient(to top,
      #375abb 0%,
      #8bb4e3 21%,
      #84b4e9 36%,
      #3f8ae0 44%,
      #95e0ff 86%,
      #63abf2 100%);
  border-top-width: 0px;
  border-bottom-width: 0px;
  border-radius: 6px;
  box-shadow:
    inset 0 1px #0028ab,
    inset 0 -1px #0028ab,
    inset 1px 0 #0028ab,
    inset -1px 0px #0028ab;
  border-radius: 8px;
  /* background: linear-gradient(to bottom, #89b6ff 0%, #6da6ff 100%); */
}

@media (max-width: 640px) {
  ::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }

  ::-webkit-scrollbar-thumb {
    border-width: 0px;
    border-right-width: 1px;
  }

  ::-webkit-scrollbar-thumb:horizontal {
    border-width: 0px;
    border-bottom-width: 1px;
  }
}

.content-wrapper {
  min-height: 100%;
  padding-bottom: 2rem;
}

.main-content {
  overflow-y: auto;
}

.sihesle {
  color: #fff;
  font-size: 1em;
  font-family: Arial, sans-serif;
  text-shadow: 0 0 5px #000;
}

@keyframes sihesle_farlu {
  0% {
    top: -10%;
  }

  100% {
    top: 100%;
  }
}

@keyframes sihesle_slilu {
  0% {
    transform: translateX(0) rotate(0deg);
  }

  50% {
    transform: translateX(80px) rotate(180deg);
  }

  100% {
    transform: translateX(0) rotate(359deg);
  }
}

.sihesle {
  position: fixed;
  top: -10%;
  z-index: 9999;
  user-select: none;
  cursor: default;
  animation-name: sihesle_farlu, sihesle_slilu;
  animation-duration: 40s, 7s;
  animation-timing-function: linear, ease-in-out;
  animation-iteration-count: infinite, infinite;
  animation-play-state: running, running;
}

.pyro {
  z-index: 60;
  position: fixed;
  width: 5px;
  height: 5px;
  border-radius: 50%;
  box-shadow:
    -120px -218.66667px blue,
    248px -16.66667px #00ff84,
    190px 16.33333px #002bff,
    -113px -308.66667px #ff009d,
    -109px -287.66667px #ffb300,
    -50px -313.66667px #ff006e,
    226px -31.66667px #ff4000,
    180px -351.66667px #ff00d0,
    -12px -338.66667px #00f6ff,
    220px -388.66667px #99ff00,
    -69px -27.66667px #ff0400,
    -111px -339.66667px #6200ff,
    155px -237.66667px #00ddff,
    -152px -380.66667px #00ffd0,
    -50px -37.66667px #00ffdd,
    -95px -175.66667px #a6ff00,
    -88px 10.33333px #0d00ff,
    112px -309.66667px #005eff,
    69px -415.66667px #ff00a6,
    168px -100.66667px #ff004c,
    -244px 24.33333px #ff6600,
    97px -325.66667px #ff0066,
    -211px -182.66667px #00ffa2,
    236px -126.66667px #b700ff,
    140px -196.66667px #9000ff,
    125px -175.66667px #00bbff,
    118px -381.66667px #ff002f,
    144px -111.66667px #ffae00,
    36px -78.66667px #f600ff,
    -63px -196.66667px #c800ff,
    -218px -227.66667px #d4ff00,
    -134px -377.66667px #ea00ff,
    -36px -412.66667px #ff00d4,
    209px -106.66667px #00fff2,
    91px -278.66667px #000dff,
    -22px -191.66667px #9dff00,
    139px -392.66667px #a6ff00,
    56px -2.66667px #0099ff,
    -156px -276.66667px #ea00ff,
    -163px -233.66667px #00fffb,
    -238px -346.66667px #00ff73,
    62px -363.66667px #0088ff,
    244px -170.66667px #0062ff,
    224px -142.66667px #b300ff,
    141px -208.66667px #9000ff,
    211px -285.66667px #ff6600,
    181px -128.66667px #1e00ff,
    90px -123.66667px #c800ff,
    189px 70.33333px #00ffc8,
    -18px -383.66667px #00ff33,
    100px -6.66667px #ff008c;
  animation:
    1s bang ease-out 1 backwards,
    1s gravity ease-in 1 backwards,
    3s position linear 1 backwards;
  animation-delay: 0s, 0s, 0s;
}

@keyframes bang {
  from {
    box-shadow:
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white,
      0 0 white;
  }
}

@keyframes gravity {
  to {
    transform: translateY(50px);
    opacity: 0;
  }
}

@keyframes marquee {
  from {
    transform: translateX(100%);
  }

  to {
    transform: translateX(-100%);
  }
}

@keyframes position {

  0%,
  19.9% {
    margin-top: 10%;
    margin-left: 40%;
  }

  20%,
  39.9% {
    margin-top: 40%;
    margin-left: 30%;
  }

  40%,
  59.9% {
    margin-top: 20%;
    margin-left: 70%;
  }

  60%,
  79.9% {
    margin-top: 30%;
    margin-left: 20%;
  }

  80%,
  99.9% {
    margin-top: 30%;
    margin-left: 80%;
  }
}
</style>

<style>
@keyframes rotate-3d {
  0% {
    transform: rotateY(0deg) rotateX(0deg);
  }

  25% {
    transform: rotateY(180deg) rotateX(0deg);
  }

  50% {
    transform: rotateY(180deg) rotateX(180deg);
  }

  75% {
    transform: rotateY(0deg) rotateX(180deg);
  }

  100% {
    transform: rotateY(0deg) rotateX(0deg);
  }
}

.animate-rotate-3d {
  animation: rotate-3d 3s ease-in-out;
}

#background-container {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: -2;
  background-size: cover;
  background-position: center;
  transition: background-image 1s ease-in-out;
}
</style>
