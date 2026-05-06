<template>
  <BackgroundComponent />
  <div v-if="isWinterSeason" class="snowflakes" aria-hidden="true">
    <div
      v-for="(flake, index) in snowflakes"
      :key="index"
      class="sihesle"
      :style="{ left: `${flake.left}%`, 'animation-delay': `${flake.delay1}s, ${flake.delay2}s` }"
    >
      {{ index % 2 === 0 ? '❅' : '❆' }}
    </div>
  </div>

  <div v-if="isWinterSeason && showPyro" class="pyro" />
  <AppFixedBanners
    :show-test-data-warning="showTestDataWarning"
    :show-unconfirmed-warning="showUnconfirmedWarning"
    :discord-chat-url="discordChatUrl"
    :is-resending-confirmation="isResendingConfirmation"
    :resend-confirmation-success="resendConfirmationSuccess"
    @resend-confirmation="handleResendConfirmation"
  />
  <!-- Mobile-optimized header (omit on routes with meta.hideTopBar, e.g. full-bleed experiences) -->

  <header v-if="!route.meta.hideTopBar" class="app-header-bar">
    <div class="px-1 sm:px-2 max-w-4xl mx-auto">
      <!-- Main header content -->
      <div class="flex items-center justify-between h-14 sm:h-12">
        <!-- Logo + Toggle Menu Button -->
        <div class="flex items-center">
          <button
            class="z-15 cursor-pointer rounded-md p-3 text-gray-600 transition-colors duration-200 hover:bg-gray-100 sm:hidden"
            :aria-label="$t('toggleMenu')"
            @click.stop="isMenuOpen = !isMenuOpen"
          >
            <Menu v-if="!isMenuOpen" class="h-6 w-6" /> <X v-else class="h-6 w-6" />
          </button>
          <!-- Logo - Always visible -->
          <NavLink
            to="/"
            class="navbar-item flex items-center italic"
            :class="{
              'cursor-default pointer-events-none': isHomePage,
            }"
            @click="triggerPyro"
          >
            <div class="flex h-8 w-8 shrink-0 items-center justify-center -skew-x-12">
              <div
                role="img"
                :aria-label="$t('logoText')"
                class="logo-svg-container"
                :class="{ 'animate-rotate-3d': showPyro }"
                v-html="logoSvgRaw"
              ></div>
            </div>
            <span class="select-none font-medium leading-none">{{ $t('logoText') }}</span>
          </NavLink>
        </div>
        <!-- Desktop Navigation - Hidden on mobile -->
        <nav class="hidden sm:ml-4 sm:flex items-center space-x-0 md:space-x-1 lg:space-x-2">
          <NavLink to="/collections" class="navbar-item">
            <GraduationCap class="h-5 w-5" /> {{ $t('nav.courses') }}
          </NavLink>
          <NavLink to="/recent" class="navbar-item">
            <Clock4 class="h-5 w-5" /> {{ $t('nav.recent') }}
          </NavLink>
          <div ref="moreNavRef" class="relative group">
            <button
              type="button"
              class="navbar-item"
              :aria-expanded="isMoreNavOpen"
              aria-haspopup="true"
              @click.stop="isMoreNavOpen = !isMoreNavOpen"
            >
              <span class="hidden lg:inline"> {{ $t('nav.more') }} </span>
              <ChevronDown class="h-5 w-5" :stroke-width="2.5" :absolute-stroke-width="true" />
            </button>
            <div
              class="nav-dropdown-panel"
              :class="isMoreNavOpen ? 'flex' : 'hidden group-hover:flex'"
            >
              <NavLink
                v-if="auth.state.isLoggedIn"
                to="/users"
                class="navbar-item justify-start py-2"
                @click="closeNavMenus"
              >
                <Users class="h-4 w-4" />
                {{
                  auth.state.authorities?.includes('manage_roles')
                    ? $t('nav.iamUsers')
                    : $t('nav.users')
                }}
              </NavLink>
              <NavLink
                to="/languages"
                class="navbar-item justify-start py-2"
                @click="closeNavMenus"
              >
                <Globe class="h-4 w-4" /> {{ $t('nav.languages') }}
              </NavLink>
              <NavLink
                to="/assistant"
                class="navbar-item justify-start py-2"
                @click="closeNavMenus"
              >
                <Bot class="h-4 w-4" /> {{ $t('nav.assistant') }}
              </NavLink>
              <NavLink
                to="/semantic-graph"
                class="navbar-item justify-start py-2"
                @click="closeNavMenus"
              >
                <Share2 class="h-4 w-4" /> {{ $t('nav.semanticGraph') }}
              </NavLink>
              <NavLink
                v-if="!auth.state.isLoggedIn"
                to="/export/cached"
                class="navbar-item justify-start py-2"
                @click="closeNavMenus"
              >
                <Download class="h-4 w-4" /> {{ $t('nav.cachedExports') }}
              </NavLink>
              <NavLink
                v-if="auth.state.isLoggedIn"
                to="/export"
                class="navbar-item justify-start py-2"
                @click="closeNavMenus"
              >
                <Upload class="h-4 w-4" /> {{ $t('nav.export') }}
              </NavLink>
              <NavLink
                v-if="auth.state.isLoggedIn && auth.state.authorities?.includes('bulk_import')"
                to="/bulk-import"
                class="navbar-item justify-start py-2"
                @click="closeNavMenus"
              >
                <Download class="h-4 w-4" /> {{ $t('nav.bulkImport') }}
              </NavLink>
              <div
                class="border-t border-gray-100 mt-1 pt-1 px-2 py-2"
                role="group"
                :aria-label="$t('buttonTheme.label')"
              >
                <p class="text-xs text-gray-500 mb-1">{{ $t('buttonTheme.label') }}</p>

                <div class="flex flex-col gap-0.5">
                  <button
                    type="button"
                    class="navbar-item justify-start py-2 text-sm w-full"
                    :class="{ 'nav-link-active': buttonTheme === 'aqua' }"
                    @click="setTheme('aqua')"
                  >
                    {{ $t('buttonTheme.aqua') }}
                  </button>
                  <button
                    type="button"
                    class="navbar-item justify-start py-2 text-sm w-full"
                    :class="{ 'nav-link-active': buttonTheme === 'flat' }"
                    @click="setTheme('flat')"
                  >
                    {{ $t('buttonTheme.flat') }}
                  </button>
                </div>
              </div>
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
      <AppMobileNavMenu :show="isMenuOpen" @close="isMenuOpen = false" @logout="handleLogout" />
    </div>
  </header>
  <!-- Global Error Display -->
  <div class="flex justify-center">
    <div v-if="error?.message" class="w-full max-w-lg px-4">
      <Error
        v-if="error?.message"
        :message="error.message"
        :details="error.details != null ? String(error.details) : ''"
        @close="clearError"
      />
    </div>
  </div>
  <ToastFloat
    :show="!!successToast"
    :message="successToast?.message ?? ''"
    :duration="successToast?.duration ?? DEFAULT_SUCCESS_TOAST_DURATION_MS"
    :extra-component="successToast?.extraComponent ?? null"
    :extra-props="successToast?.extraProps ?? null"
    :close-label="$t('modal.close')"
    type="success"
    @close="clearSuccess"
  />
  <!-- Main content -->
  <main
    class="main-content"
    :class="[
      { 'scrollbar-always': route.meta.alwaysShowScrollbar },
      route.meta.fullHeight ? 'main-content--no-scroll' : '',
      route.meta.hideTopBar ? 'main-content--no-topbar' : '',
    ]"
  >
    <div
      id="main-child"
      class="max-w-4xl mx-auto relative flex flex-col"
      :class="[
        route.meta.contentTopPaddingMainOnly || route.meta.authFullBleed ? 'pt-0' : 'pt-3',
        route.meta.fullHeight ? 'main-child-full-height w-full' : 'min-h-[calc(100vh-12rem)]',
        route.meta.authFullBleed ? 'main-child--auth-fullbleed' : '',
        route.path.startsWith('/lingo') ? 'lg:pl-64' : '',
      ]"
    >
      <div
        class="flex-1"
        :class="[
          route.meta.contentTopPaddingMainOnly || route.meta.authFullBleed ? 'px-0' : 'px-3',
          { 'main-child-inner-full-height': route.meta.fullHeight },
          !route.meta.authFullBleed &&
          !route.meta.fullHeight &&
          !route.meta.contentTopPaddingMainOnly
            ? 'flex flex-col gap-4'
            : '',
        ]"
      >
        <div
          v-if="auth.state.isLoading"
          class="flex h-full w-full items-center justify-center min-h-[50vh]"
        >
          <div
            class="h-8 w-8 animate-spin rounded-full border-4 border-gray-200 border-t-blue-600"
          ></div>
        </div>
        <router-view v-else v-slot="{ Component, route }">
          <component
            :is="Component"
            v-bind="metaProps(route.meta.props)"
            v-on="
              isHomePage
                ? {
                    search: performSearch,
                    'view-message': viewMessage,
                    'view-thread': viewThread,
                  }
                : {}
            "
          />
        </router-view>
      </div>

      <footer
        v-if="!route.meta.hideFooter"
        class="mt-6 max-w-full break-words px-3 py-3 text-center text-xs text-gray-500 leading-relaxed"
      >
        {{ $t('footer.publicDomainNotice') }}
      </footer>
    </div>
  </main>
  <!-- Floating action menu (FAB trigger + standard dropdown panel) -->
  <div
    v-if="auth.state.isLoggedIn && route.name !== 'flashcard-study' && !route.meta.fullHeight"
    class="max-w-4xl mx-auto relative"
  >
    <div
      class="fixed md:absolute bottom-8 right-4 md:right-6 lg:-right-4 lg:-mr-4 z-50 flex flex-col items-end gap-3"
    >
      <Dropdown>
        <template #trigger="{ open }">
          <span class="fab-elevation-shell">
            <button
              type="button"
              class="inline-flex items-center justify-center ui-btn--fab"
              :aria-label="$t('fab.actionsTitle')"
            >
              <Plus
                class="h-8 w-8 shrink-0 transition-transform duration-200"
                stroke-width="2.75"
                :class="{ 'rotate-45': open }"
              />
            </button>
          </span>
        </template>
        <ToolbarSelectDropdownItem class="!px-4 !py-3 !text-base" @click="handleAssistantChat">
          <span class="flex items-center gap-3">
            <Bot class="h-6 w-6 shrink-0 text-indigo-600" stroke-width="2" />
            {{ $t('nav.assistant') }}
          </span>
        </ToolbarSelectDropdownItem>
        <ToolbarSelectDropdownItem class="!px-4 !py-3 !text-base" @click="handleSemanticGraph">
          <span class="flex items-center gap-3">
            <Share2 class="h-6 w-6 shrink-0 text-cornflower-500" stroke-width="2" />
            {{ $t('nav.semanticGraph') }}
          </span>
        </ToolbarSelectDropdownItem>
        <ToolbarSelectDropdownItem class="!px-4 !py-3 !text-base" @click="handleNewFreeThread">
          <span class="flex items-center gap-3">
            <AudioWaveform class="h-6 w-6 shrink-0 text-purple-600" stroke-width="2" />
            {{ $t('fab.newDiscussion') }}
          </span>
        </ToolbarSelectDropdownItem>
        <ToolbarSelectDropdownItem class="!px-4 !py-3 !text-base" @click="handleNewDefinition">
          <span class="flex items-center gap-3">
            <FilePlus class="h-6 w-6 shrink-0 text-emerald-600" stroke-width="2" />
            {{ $t('fab.addDefinition') }}
          </span>
        </ToolbarSelectDropdownItem>
      </Dropdown>
    </div>
  </div>
  <FooterComponent v-if="!route.meta.fullHeight" />
</template>

<script setup lang="ts">
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
  Plus,
  FilePlus,
  AudioWaveform,
  BookmarkCheck,
  Clock4,
  GraduationCap,
  Bot,
  Share2,
} from 'lucide-vue-next'
import { Menu } from 'lucide-vue-next' // Explicitly import Menu if it was missed by auto-sort
import { ref, onMounted, watch, computed, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'

import { useI18n } from 'vue-i18n'
import { useHead } from '@vueuse/head'
import { jwtDecode } from 'jwt-decode'

import Error from '@/components/Error.vue'
import ToastFloat from '@/components/ToastFloat.vue'
import { resendConfirmation } from '@/api'
import { Dropdown, ToolbarSelectDropdownItem } from '@packages/ui'

import BackgroundComponent from './components/BackgroundComponent.vue'
import FooterComponent from './components/FooterComponent.vue'
import AppFixedBanners from './components/layout/AppFixedBanners.vue'
import AppMobileNavMenu from './components/layout/AppMobileNavMenu.vue'
import NavLink from './components/NavLink.vue'
import { normalizeSearchQuery } from '@/utils/searchQueryUtils'
import { queryStr } from '@/utils/routeQuery'
import { provideAuth } from './composables/useAuth'
import { provideError } from './composables/useError'
import {
  DEFAULT_SUCCESS_TOAST_DURATION_MS,
  provideSuccessToast,
} from './composables/useSuccessToast'
import { useButtonTheme } from './composables/useButtonTheme'
import { localeCaptureGroupRegex } from './config/locales'

import logoSvgRaw from '../public/assets/icons/favicon.svg?raw'

const i18n = useI18n()
const $t = i18n.t
const $locale = i18n.locale

// Default SEO: meta description so Google shows a proper snippet instead of footer text.
// Pages can override by setting description in useSeoHead (same key 'description').
useHead({
  meta: [
    {
      name: 'description',
      content: i18n.t('seo.defaultDescription'),
      key: 'description',
    },
  ],
})
const router = useRouter()
const route = useRoute()

const isHomePage = computed(
  () => route.name === 'Home' || (typeof route.name === 'string' && route.name.startsWith('Home-'))
)

function metaProps(p: unknown): Record<string, unknown> {
  return p !== null && typeof p === 'object' && !Array.isArray(p)
    ? (p as Record<string, unknown>)
    : {}
}
const searchQuery = ref('')
const searchMode = ref('messages')
const auth = provideAuth()
const { buttonTheme, initButtonTheme, setButtonTheme: setButtonThemePreference } = useButtonTheme()
const { error, clearError } = provideError()
const { successToast, clearSuccess } = provideSuccessToast()
const isMenuOpen = ref(false)
const isMoreNavOpen = ref(false)
const moreNavRef = ref(null)
const showPyro = ref(false)
const discordChatUrl = 'https://discord.gg/4KhzRzpmVr'
const showTestDataWarning = import.meta.env.VITE_SHOW_TEST_DATA_WARNING === 'true'
const isResendingConfirmation = ref(false)
const resendConfirmationSuccess = ref(false)

const showUnconfirmedWarning = computed(() => {
  return (
    auth.state.isLoggedIn &&
    !auth.state.isLoading &&
    (auth.state.role.toLowerCase() === 'unconfirmed' || !auth.state.email_confirmed)
  )
})

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
}

const handleAssistantChat = () => {
  router.push('/assistant')
}

const handleSemanticGraph = () => {
  router.push('/semantic-graph')
}

const handleNewFreeThread = () => {
  router.push('/comments/new-thread')
}

const triggerPyro = () => {
  showPyro.value = !showPyro.value
  setTimeout(() => {
    showPyro.value = false
  }, 3000)
}

const performSearch = ({ query, mode }: { query: string; mode: string }) => {
  searchQuery.value = normalizeSearchQuery(query) as string
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
  searchQuery.value = normalizeSearchQuery(queryStr(route.query.q)) as string
  searchMode.value = queryStr(route.query.mode) || 'messages'
}

const closeNavMenus = () => {
  isMenuOpen.value = false
  isMoreNavOpen.value = false
}

const setTheme = (theme: Parameters<typeof setButtonThemePreference>[0]) => {
  setButtonThemePreference(theme)
  closeNavMenus()
}

const handleLogout = () => {
  auth.logout()
  router.push('/login')
  closeNavMenus()
}

const handleResendConfirmation = async () => {
  if (isResendingConfirmation.value) return

  try {
    // Get email from JWT token
    const accessToken = auth.state.accessToken || localStorage.getItem('accessToken')
    if (!accessToken) {
      console.error('No access token available')
      return
    }

    const decoded = jwtDecode<{ email?: string }>(accessToken)
    const email = decoded.email

    if (!email) {
      console.error('No email found in token')
      return
    }

    isResendingConfirmation.value = true
    resendConfirmationSuccess.value = false

    const response = await resendConfirmation(email)

    if (response.data.success) {
      resendConfirmationSuccess.value = true
      // Hide success message after 5 seconds
      setTimeout(() => {
        resendConfirmationSuccess.value = false
      }, 5000)
    }
  } catch (err) {
    console.error('Failed to resend confirmation email:', err)
    // Error handling could be improved with a toast notification
  } finally {
    isResendingConfirmation.value = false
  }
}

const viewMessage = (messageId: string | number) => {
  router.push({ name: 'message', params: { id: String(messageId) } })
}

const viewThread = (subject: string) => {
  router.push({
    name: 'thread',
    params: { subject: encodeURIComponent(subject) },
  })
}

// Close mobile menu and clear errors on route change
watch(
  () => route.fullPath,
  async () => {
    closeNavMenus()
    clearError()
    if (!route.meta.scrollMainToTopOnNavigate) return
    await nextTick()
    const mainContent = document.querySelector('.main-content') as HTMLElement | null
    mainContent?.scrollTo({ top: 0, behavior: 'auto' })
  }
)

// Handle click outside mobile menu to close it
const handleClickOutside = (event) => {
  const header = document.querySelector('header')
  if (isMenuOpen.value && header && !header.contains(event.target)) {
    isMenuOpen.value = false
  }
  if (isMoreNavOpen.value && moreNavRef.value && !moreNavRef.value.contains(event.target)) {
    isMoreNavOpen.value = false
  }
}

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

// Global keyboard handler for "/" key to navigate to homepage and focus search
const handleGlobalKeyDown = async (event) => {
  // Only handle "/" key
  if (event.key !== '/') {
    return
  }

  // Don't trigger if user is typing in an input, textarea, or select
  const activeElement = document.activeElement as HTMLElement | null
  const tagName = activeElement?.tagName ?? ''
  const isContentEditable = activeElement?.isContentEditable === true

  if (['INPUT', 'TEXTAREA', 'SELECT'].includes(tagName) || isContentEditable) {
    return
  }

  // Prevent default behavior
  event.preventDefault()

  // Get current locale from route or use default
  const currentPath = route.path
  const localeMatch = currentPath.match(localeCaptureGroupRegex)
  const currentLocale = localeMatch ? localeMatch[1] : $locale.value || 'en'

  // Navigate to homepage if not already there
  if (!isHomePage.value) {
    await router.push(`/${currentLocale}`)
  }

  // Focus the search input after navigation
  // Wait for the route to update and component to mount
  await nextTick()

  // Try multiple times to find the search input as the component might need time to mount
  const focusSearchInput = () => {
    // Try to find the search input in the SearchForm component
    // The search form has class "search-form" and contains an input
    const searchInput = document.querySelector(
      '.search-form input[type="text"], .search-form input:not([type="hidden"])'
    ) as HTMLElement | null
    if (searchInput) {
      searchInput.focus()
      return true
    }
    return false
  }

  // Try immediately
  if (!focusSearchInput()) {
    // If not found, try again after a short delay to allow component to mount
    setTimeout(() => {
      if (!focusSearchInput()) {
        // Final fallback: try to find any input with input-field class in the main content area
        const fallbackInput = document.querySelector(
          'main .input-field[type="text"], main input.input-field:not([type="hidden"])'
        ) as HTMLElement | null
        if (fallbackInput) {
          fallbackInput.focus()
        }
      }
    }, 150)
  }
}

onMounted(() => {
  initButtonTheme()
  // Close mobile menu when window resizes to desktop width
  const handleResize = () => {
    if (window.innerWidth >= 640) {
      // sm breakpoint
      isMenuOpen.value = false
      isMoreNavOpen.value = false
    }
  }

  window.addEventListener('resize', handleResize)
  handleResize() // Check initial size

  document.addEventListener('click', handleClickOutside)

  // Add global keyboard handler for "/" key
  window.addEventListener('keydown', handleGlobalKeyDown)

  // Cleanup
  return () => {
    window.removeEventListener('resize', handleResize)
    document.removeEventListener('click', handleClickOutside)
    window.removeEventListener('keydown', handleGlobalKeyDown)
  }
})
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
}

.main-content > * {
  @apply bg-transparent md:bg-zinc-50/75 md:border-x;
  min-height: 100%;
}

/* Login / signup: global BackgroundComponent + body::before stay visible; no zinc column over the art */
.main-content > #main-child.main-child--auth-fullbleed {
  @apply bg-transparent md:bg-transparent md:border-0;
}

@media (min-width: 640px) {
  .main-content {
    height: calc(100vh - 49px - 24px);
  }
}

/* No global header: only reserve fixed footer strip (h-6 in FooterComponent). */
.main-content.main-content--no-topbar:not(.main-content--no-scroll) {
  height: calc(100vh - 24px);
}

@media (min-width: 640px) {
  .main-content.main-content--no-topbar:not(.main-content--no-scroll) {
    height: calc(100vh - 24px);
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
  background-image: linear-gradient(
    to right,
    #375abb 0%,
    #8bb4e3 21%,
    #84b4e9 38%,
    #3f8ae0 40%,
    #95e0ff 86%,
    #63abf2 100%
  );
  box-shadow:
    inset 0 1px #0028ab,
    inset 0 -1px #0028ab,
    inset 1px 0 #0028ab,
    inset -1px 0px #0028ab;
  border-radius: 8px;
  border: 2px solid rgb(236, 236, 236);
}

::-webkit-scrollbar-thumb:horizontal {
  background-image: linear-gradient(
    to bottom,
    #375abb 0%,
    #8bb4e3 21%,
    #84b4e9 36%,
    #3f8ae0 44%,
    #95e0ff 86%,
    #63abf2 100%
  );
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
  background-image: linear-gradient(
    to left,
    #375abb 0%,
    #8bb4e3 21%,
    #84b4e9 38%,
    #3f8ae0 40%,
    #95e0ff 86%,
    #63abf2 100%
  );
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
  background-image: linear-gradient(
    to top,
    #375abb 0%,
    #8bb4e3 21%,
    #84b4e9 36%,
    #3f8ae0 44%,
    #95e0ff 86%,
    #63abf2 100%
  );
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

.main-content.scrollbar-always {
  overflow-y: scroll;
}

/* Full-height pages (e.g. Lingo study): no scrollbar, child fills main.
   Use small viewport height so layout fits when mobile browser chrome is visible;
   100vh alone is often taller than the visible area and pushes content below the fold. */
.main-content.main-content--no-scroll {
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  /* legacy */
  height: calc(100vh - 57px);
  /* prefer svh when supported (mobile toolbars / dynamic UI) */
  height: calc(100svh - 57px);
  padding-bottom: env(safe-area-inset-bottom, 0px);
}

@media (min-width: 640px) {
  .main-content.main-content--no-scroll {
    height: calc(100vh - 49px);
    height: calc(100svh - 49px);
  }
}

/* Full-height child + no app header: fill viewport (global footer already hidden via meta.fullHeight). */
.main-content.main-content--no-scroll.main-content--no-topbar {
  height: 100vh;
  height: 100svh;
}

@media (min-width: 640px) {
  .main-content.main-content--no-scroll.main-content--no-topbar {
    height: 100vh;
    height: 100svh;
  }
}

.main-content.main-content--no-scroll > #main-child {
  flex: 1 1 0;
  min-height: 0;
  overflow: hidden;
}

/* Prevent page scroll when full-height study is active */
html:has(.main-content.main-content--no-scroll),
body:has(.main-content.main-content--no-scroll) {
  overflow: hidden;
  height: 100%;
}

.main-child-full-height {
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.main-child-inner-full-height {
  flex: 1 1 0;
  min-height: 0;
  padding-left: 0;
  padding-right: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
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
