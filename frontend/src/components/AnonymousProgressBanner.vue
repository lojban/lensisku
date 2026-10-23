<template>
  <Transition :name="transitionName">
    <div v-if="show" ref="bannerRef" :class="bannerClasses">
      <p :class="messageClass">
        {{ t('anonymousProgress.bannerMessage') }}
      </p>

      <div :class="actionsClass">
        <RouterLink :to="signupTo" :class="signupClass">
          {{ t('anonymousProgress.signUp') }}
        </RouterLink>
        <RouterLink :to="loginTo" :class="loginClass">
          {{ t('anonymousProgress.logIn') }}
        </RouterLink>
        <button
          type="button"
          :class="dismissClass"
          :aria-label="t('anonymousProgress.dismiss')"
          @click="dismiss"
        >
          <X class="h-4 w-4" />
        </button>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { X } from '@lucide/vue'
import { useAuth } from '@/composables/useAuth'
import { getAllProgressForMerge } from '@/composables/useAnonymousProgress'

const props = defineProps({
  /** 'top' fixed under app chrome; 'bottom' fixed; 'inline' in-flow (Lingo study). */
  position: { type: String, default: 'bottom' },
  /** 'lingo' matches Lingo chrome; default is classic flat buttons. */
  variant: { type: String, default: 'default' },
})

const emit = defineEmits(['visible'])

const DISMISS_KEY = 'lensisku_anon_banner_dismissed'

const { t } = useI18n()

const isLingo = computed(() => props.variant === 'lingo')
const isTop = computed(() => props.position === 'top')
const isInline = computed(() => props.position === 'inline')
const transitionName = computed(() => {
  if (isInline.value) return 'fade'
  return isTop.value ? 'slide-down' : 'slide-up'
})

const bannerClasses = computed(() => {
  if (isLingo.value && isInline.value) {
    return [
      'anon-progress-banner anon-progress-banner--lingo',
      'mx-auto flex w-full max-w-[600px] shrink-0 flex-col gap-2 px-3 py-2.5 sm:flex-row sm:items-center sm:justify-between sm:gap-3',
      'border-b-2 border-green-200 bg-green-50 text-slate-700',
    ]
  }
  return [
    'anon-progress-banner fixed left-0 right-0 z-30 flex flex-col sm:flex-row sm:items-center sm:justify-center gap-3 sm:gap-4 px-4 py-3 bg-white border-gray-200',
    isTop.value
      ? 'top-14 sm:top-12 border-b shadow-[0_4px_6px_-1px_rgba(0,0,0,0.1)] safe-area-pt'
      : 'bottom-0 border-t shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.1)] safe-area-pb',
  ]
})

const messageClass = computed(() =>
  isLingo.value
    ? 'min-w-0 flex-1 text-center text-xs font-bold text-slate-700 sm:text-left sm:text-sm'
    : 'text-sm text-gray-700 flex-1 min-w-0 text-center sm:text-left order-1 sm:order-none'
)

const actionsClass = computed(() =>
  isLingo.value
    ? 'flex shrink-0 items-center justify-center gap-1.5 sm:justify-end'
    : 'order-2 flex shrink-0 items-center justify-center gap-2 sm:order-none'
)

const signupClass = computed(() =>
  isLingo.value
    ? 'lingo-btn lingo-btn--secondary lingo-btn--sm !normal-case !tracking-normal'
    : 'anon-banner-cta ui-btn--warning-orange'
)

const loginClass = computed(() =>
  isLingo.value
    ? 'lingo-btn lingo-btn--ghost lingo-btn--sm !normal-case !tracking-normal'
    : 'anon-banner-cta ui-btn--empty border border-gray-300 hover:bg-gray-50'
)

const dismissClass = computed(() =>
  isLingo.value
    ? 'lingo-btn lingo-btn--ghost lingo-btn--sm lingo-btn--icon !min-w-9 !px-0'
    : 'anon-banner-dismiss ui-btn--empty inline-flex items-center justify-center rounded-md p-1.5 text-gray-500 hover:bg-gray-100'
)

const route = useRoute()
const auth = useAuth()
const bannerRef = ref(null)

const dismissed = ref(false)

const returnTo = computed(() => route.fullPath || '/lingo/courses')
const signupTo = computed(() => ({ path: '/signup', query: { returnTo: returnTo.value } }))
const loginTo = computed(() => ({ path: '/login', query: { returnTo: returnTo.value } }))

const isAuthPage = computed(() => {
  const name = route.name?.toString() || ''
  return name.includes('Login') || name.includes('Signup') || name.includes('SignUp')
})

/** Show when on a collection/course context and either has progress to save or is on study/levels (where progress is made). */
const isRelevantCollectionPath = computed(() => {
  const path = (route.path || '').replace(/\/$/, '')
  if (!path.includes('/collections') && !path.includes('/lingo')) return false
  const segments = path.split('/').filter(Boolean)
  const isListOnly = segments.length === 2 && segments[1] === 'collections'
  if (isListOnly) {
    try {
      const payloads = getAllProgressForMerge()
      return payloads.length > 0
    } catch (_) {
      return true
    }
  }
  return true
})

const show = computed(() => {
  if (auth.state.isLoggedIn || dismissed.value || isAuthPage.value) return false
  return isRelevantCollectionPath.value
})

function dismiss() {
  dismissed.value = true
  try {
    sessionStorage.setItem(DISMISS_KEY, '1')
  } catch {
    /* ignore */
  }
}

watch(
  () => route.path,
  () => {
    try {
      if (sessionStorage.getItem(DISMISS_KEY)) dismissed.value = true
    } catch {
      /* ignore */
    }
  },
  { immediate: true }
)

watch(show, (v) => emit('visible', v), { immediate: true })
</script>

<style scoped>
.safe-area-pb {
  padding-bottom: max(0.75rem, env(safe-area-inset-bottom, 0px));
}
.safe-area-pt {
  padding-top: max(0.75rem, env(safe-area-inset-top, 0px));
}
.slide-up-enter-active,
.slide-up-leave-active {
  transition: transform 0.2s ease-out;
}
.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(100%);
}
.slide-down-enter-active,
.slide-down-leave-active {
  transition: transform 0.2s ease-out;
}
.slide-down-enter-from,
.slide-down-leave-to {
  transform: translateY(-100%);
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease-out;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
