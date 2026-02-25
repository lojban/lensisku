<template>
  <Transition name="slide-up">
    <div
      v-if="show"
      class="fixed left-0 right-0 bottom-0 z-30 flex items-center justify-center gap-4 px-4 py-3 bg-white border-t border-gray-200 shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.1)] safe-area-pb"
    >
      <p class="text-sm text-gray-700 flex-1 text-center sm:text-left">
        {{ t('anonymousProgress.bannerMessage', 'Sign up to save your progress across devices') }}
      </p>
      <div class="flex items-center gap-2 shrink-0">
        <RouterLink
          to="/signup"
          class="btn-aqua-orange text-sm px-3 py-1.5"
        >
          {{ t('anonymousProgress.signUp', 'Sign up') }}
        </RouterLink>
        <RouterLink
          to="/login"
          class="btn-empty text-sm px-3 py-1.5 border border-gray-300 rounded-md hover:bg-gray-50"
        >
          {{ t('anonymousProgress.logIn', 'Log in') }}
        </RouterLink>
        <button
          type="button"
          class="p-1.5 text-gray-400 hover:text-gray-600 rounded"
          :aria-label="t('anonymousProgress.dismiss', 'Dismiss')"
          @click="dismiss"
        >
          <X class="h-4 w-4" />
        </button>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { X } from 'lucide-vue-next'
import { useAuth } from '@/composables/useAuth'

const DISMISS_KEY = 'lensisku_anon_banner_dismissed'

const { t } = useI18n()
const route = useRoute()
const auth = useAuth()

const dismissed = ref(false)

const isAuthPage = computed(() => {
  const name = route.name?.toString() || ''
  return name.includes('Login') || name.includes('Signup') || name.includes('SignUp')
})

const show = computed(() => {
  if (auth.state.isLoggedIn || dismissed.value || isAuthPage.value) return false
  const path = route.path || ''
  return path.includes('/collections')
})

function dismiss () {
  dismissed.value = true
  try {
    sessionStorage.setItem(DISMISS_KEY, '1')
  } catch (_) {}
}

watch(
  () => route.path,
  () => {
    try {
      if (sessionStorage.getItem(DISMISS_KEY)) dismissed.value = true
    } catch (_) {}
  },
  { immediate: true }
)
</script>

<style scoped>
.safe-area-pb {
  padding-bottom: max(0.75rem, env(safe-area-inset-bottom, 0px));
}
.slide-up-enter-active,
.slide-up-leave-active {
  transition: transform 0.2s ease-out;
}
.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(100%);
}
</style>
