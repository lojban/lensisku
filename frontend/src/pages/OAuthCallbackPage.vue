<template>
  <div class="auth-page-shell">
    <AuthFormCard>
      <h2 class="auth-form-title">{{ t('oauth.completing') }}</h2>
      <p v-if="error" class="w-full text-center text-sm text-red-600" role="alert">{{ error }}</p>
      <p v-else class="w-full text-center text-sm text-gray-600">{{ t('oauth.completing') }}</p>
      <p v-if="error" class="mt-4 w-full text-center text-sm text-gray-600">
        <RouterLink
          to="/login"
          class="font-medium text-blue-600 underline-offset-2 hover:text-blue-800 hover:underline"
        >
          {{ t('signupPage.loginLink') }}
        </RouterLink>
      </p>
    </AuthFormCard>
  </div>
</template>

<script setup lang="ts">
import { AuthFormCard } from '@packages/ui'
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'

import { useAuth } from '@/composables/useAuth'
import { finishSocialLogin, isAuthRedirect } from '@/composables/useSocialLogin'
import { useSeoHead } from '@/composables/useSeoHead'
import { queryStr, paramStr } from '@/utils/routeQuery'

const route = useRoute()
const router = useRouter()
const auth = useAuth()
const { t } = useI18n()
const error = ref('')

useSeoHead({ title: t('oauth.completing'), robots: 'noindex, nofollow' })

onMounted(async () => {
  const provider = paramStr(route.params.provider)
  const oauthError = queryStr(route.query.error)
  const code = queryStr(route.query.code)
  const state = queryStr(route.query.state)

  if (oauthError === 'access_denied') {
    error.value = t('oauth.cancelled')
    return
  }
  if (oauthError || !provider || !code || !state) {
    error.value = t('oauth.failed')
    return
  }

  try {
    const result = await finishSocialLogin(provider, code, state)
    auth.login(result.access_token, result.refresh_token, result.username)
    const stored = sessionStorage.getItem('redirectPath')
    sessionStorage.removeItem('redirectPath')
    const candidate = result.return_to || stored || '/'
    const target = candidate && !isAuthRedirect(candidate) ? candidate : '/'
    await router.replace(target)
  } catch (err: unknown) {
    const data = (err as { response?: { status?: number; data?: { error?: string } } }).response
    if (data?.status === 409 || data?.data?.error === 'account_collision') {
      error.value = t('oauth.collision')
    } else if (data?.status === 503) {
      error.value = t('oauth.notConfigured')
    } else {
      error.value = t('oauth.failed')
    }
  }
})
</script>
