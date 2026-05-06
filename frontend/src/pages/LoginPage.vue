<template>
  <div class="auth-page-shell">
    <AuthFormCard>
      <h2 class="auth-form-title">{{ t('loginPage.title') }}</h2>

      <form class="w-full space-y-6" @submit.prevent="performLogin">
        <div>
          <label for="username" class="mb-1 block text-sm font-medium text-gray-700">{{
            t('loginPage.usernameLabel')
          }}</label>
          <div class="relative">
            <input
              id="username"
              v-model="username"
              type="text"
              required
              class="input-field h-10 w-full pl-3 pr-10 text-base"
              :disabled="isLoading"
              :placeholder="t('loginPage.usernamePlaceholder')"
            />
            <User class="input-field-trailing-icon" aria-hidden="true" />
          </div>
        </div>

        <div>
          <label for="password" class="mb-1 block text-sm font-medium text-gray-700">{{
            t('loginPage.passwordLabel')
          }}</label>
          <div class="relative">
            <input
              id="password"
              v-model="password"
              :type="showPassword ? 'text' : 'password'"
              required
              autocomplete="current-password"
              class="input-field h-10 w-full pl-3 pr-10 text-base"
              :disabled="isLoading"
              :placeholder="t('loginPage.passwordPlaceholder')"
            />
            <button
              type="button"
              class="input-field-password-toggle"
              :aria-label="showPassword ? t('loginPage.hidePassword') : t('loginPage.showPassword')"
              :aria-pressed="showPassword"
              :disabled="isLoading"
              @click="showPassword = !showPassword"
            >
              <Eye v-if="!showPassword" class="h-5 w-5 shrink-0" aria-hidden="true" />
              <EyeOff v-else class="h-5 w-5 shrink-0" aria-hidden="true" />
            </button>
          </div>
        </div>

        <div>
          <Button
            variant="warning-orange"
            size="lg"
            type="submit"
            class="w-full"
            :loading="isLoading"
            :disabled="isLoading"
          >
            <template #icon> <KeyRound class="h-6 w-6 shrink-0" /> </template>
            {{ isLoading ? t('loginPage.authenticating') : t('loginPage.loginButton') }}
          </Button>
        </div>
      </form>

      <p class="mt-4 w-full text-center text-sm text-gray-600">
        <RouterLink
          to="/reset-password"
          class="font-medium text-blue-600 underline-offset-2 hover:text-blue-800 hover:underline"
        >
          {{ t('loginPage.forgotPasswordLink') }}
        </RouterLink>
      </p>

      <p class="mt-4 w-full text-center text-sm text-gray-600">
        {{ t('loginPage.noAccountPrompt') }}
        <RouterLink
          to="/signup"
          class="font-medium text-blue-600 underline-offset-2 hover:text-blue-800 hover:underline"
        >
          {{ t('loginPage.signUpLink') }}
        </RouterLink>
      </p>
    </AuthFormCard>
  </div>
</template>

<script setup lang="ts">
import { User, Eye, EyeOff, KeyRound } from 'lucide-vue-next'

import { AuthFormCard, Button } from '@packages/ui'
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { login } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const username = ref('')
const password = ref('')
const showPassword = ref(false)
const isLoading = ref(false)
const router = useRouter()
const auth = useAuth()
const { showError, clearError } = useError()
const { t } = useI18n()

useSeoHead({ title: t('loginPage.title'), robots: 'noindex, nofollow' })

const performLogin = async () => {
  clearError()
  isLoading.value = true
  try {
    const response = await login({
      username_or_email: username.value,
      password: password.value,
    })
    if (response.data.access_token) {
      auth.login(response.data.access_token, response.data.refresh_token, username.value)
      const redirectPath = sessionStorage.getItem('redirectPath')
      sessionStorage.removeItem('redirectPath')
      router.push(redirectPath || '/')
    }
  } catch (err) {
    if (err.response?.status === 429) {
      showError(t('loginPage.rateLimitError'))
    } else if (err.response?.data?.error_description) {
      showError(err.response.data.error_description)
    } else if (err.response?.data) {
      showError(err.response.data)
    } else {
      showError(t('loginPage.loginError'))
    }
  } finally {
    isLoading.value = false
  }
}
</script>
