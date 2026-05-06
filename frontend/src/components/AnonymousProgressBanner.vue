<template>
  <Transition :name="transitionName">
    <div v-if="show" ref="bannerRef" :class="bannerClasses">
      <p
        class="text-sm text-gray-700 flex-1 min-w-0 text-center sm:text-left order-1 sm:order-none"
      >
        {{ t('anonymousProgress.bannerMessage') }}
      </p>

      <div class="order-2 flex shrink-0 items-center justify-center gap-2 sm:order-none">
        <RouterLink to="/signup" class="anon-banner-cta ui-btn--warning-orange">
          {{ t('anonymousProgress.signUp') }}
        </RouterLink>
        <RouterLink
          to="/login"
          class="anon-banner-cta ui-btn--empty border border-gray-300 hover:bg-gray-50"
        >
          {{ t('anonymousProgress.logIn') }}
        </RouterLink>
        <button
          type="button"
          class="anon-banner-dismiss"
          :aria-label="t('anonymousProgress.dismiss')"
          @click="dismiss"
        >
          <X class="h-5 w-5" />
        </button>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { X } from 'lucide-vue-next'
import { useAuth } from '@/composables/useAuth'
import { getAllProgressForMerge } from '@/composables/useAnonymousProgress'

const props = defineProps({
  /** 'top' for flashcard study (fixed at top); 'bottom' default */
  position: { type: String, default: 'bottom' },
})

const emit = defineEmits(['visible'])

const DISMISS_KEY = 'lensisku_anon_banner_dismissed'

const { t } = useI18n()

const isTop = computed(() => props.position === 'top')
const transitionName = computed(() => (isTop.value ? 'slide-down' : 'slide-up'))
const bannerClasses = computed(() => [
  'anon-progress-banner fixed left-0 right-0 z-30 flex flex-col sm:flex-row sm:items-center sm:justify-center gap-3 sm:gap-4 px-4 py-3 bg-white border-gray-200',
  isTop.value
    ? 'top-14 sm:top-12 border-b shadow-[0_4px_6px_-1px_rgba(0,0,0,0.1)] safe-area-pt'
    : 'bottom-0 border-t shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.1)] safe-area-pb',
])
const route = useRoute()
const auth = useAuth()
const bannerRef = ref(null)

const dismissed = ref(false)

const isAuthPage = computed(() => {
  const name = route.name?.toString() || ''
  return name.includes('Login') || name.includes('Signup') || name.includes('SignUp')
})

/** Show when on a collection/course context and either has progress to save or is on study/levels (where progress is made). */
const isRelevantCollectionPath = computed(() => {
  const path = (route.path || '').replace(/\/$/, '')
  if (!path.includes('/collections')) return false
  const segments = path.split('/').filter(Boolean)
  // On list page only (e.g. /en/collections): show banner only if user has progress to save
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
</style>
