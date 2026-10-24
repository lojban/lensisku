<template>
  <div
    v-if="!auth.state.isLoggedIn"
    class="lingo-guest-save-alert mx-auto flex w-full max-w-[600px] shrink-0 flex-col gap-2 px-3 py-2.5 sm:flex-row sm:items-center sm:justify-between sm:gap-3"
    role="status"
  >
    <p class="lingo-guest-save-alert__copy">
      {{ t('anonymousProgress.loginToSaveProgress') }}
    </p>
    <div class="flex shrink-0 items-center justify-center gap-1.5 sm:justify-end">
      <RouterLink
        :to="loginTo"
        class="lingo-btn lingo-btn--secondary lingo-btn--sm !normal-case !tracking-normal"
      >
        {{ t('anonymousProgress.logIn') }}
      </RouterLink>
      <RouterLink
        :to="signupTo"
        class="lingo-btn lingo-btn--ghost lingo-btn--sm !normal-case !tracking-normal"
      >
        {{ t('anonymousProgress.signUp') }}
      </RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuth } from '@/composables/useAuth'

const props = defineProps({
  /** Path to return to after login/signup (course tree). */
  returnTo: {
    type: String,
    required: true,
  },
})

const { t } = useI18n()
const auth = useAuth()

const loginTo = computed(() => ({
  path: '/lingo/login',
  query: { returnTo: props.returnTo },
}))
const signupTo = computed(() => ({
  path: '/lingo/signup',
  query: { returnTo: props.returnTo },
}))
</script>
