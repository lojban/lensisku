<template>

  <div
    class="flex min-h-full w-full flex-col items-center justify-center px-4 py-10 sm:px-6 sm:py-12"
  >
    <div
      class="card-elevated flex w-full max-w-md flex-col items-center rounded-2xl border border-gray-200/90 bg-white/95 p-8 ring-1 ring-gray-900/5 backdrop-blur-sm"
    >

      <h2 class="mb-6 text-center text-2xl font-bold text-gray-900 sm:text-3xl">
         {{ t('loginPage.title') }}
      </h2>

      <form class="w-full space-y-6" @submit.prevent="performLogin">

        <div>
           <label
            for="username"
            class="mb-1 block text-sm font-medium text-gray-700"
            >{{ t('loginPage.usernameLabel') }}</label
          >
          <div class="relative">
             <input
              id="username"
              v-model="username"
              type="text"
              required
              class="input-field h-10 w-full pl-3 pr-10 text-base"
              :disabled="isLoading"
              :placeholder="t('loginPage.usernamePlaceholder')"
            /> <User class="h-5 w-5 text-gray-400 absolute right-3 top-1/2 -translate-y-1/2" />
          </div>

        </div>

        <div>
           <label
            for="password"
            class="mb-1 block text-sm font-medium text-gray-700"
            >{{ t('loginPage.passwordLabel') }}</label
          >
          <div class="relative">
             <input
              id="password"
              v-model="password"
              type="password"
              required
              class="input-field h-10 w-full pl-3 pr-10 text-base"
              :disabled="isLoading"
              :placeholder="t('loginPage.passwordPlaceholder')"
            /> <Key class="h-5 w-5 text-gray-400 absolute right-3 top-1/2 -translate-y-1/2" />
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
            <template #icon>
              <KeyRound class="h-6 w-6 shrink-0" />
            </template>
            {{
              isLoading ? t('loginPage.authenticating') : t('loginPage.loginButton')
            }}
          </Button>
        </div>

      </form>

      <p class="mt-4 w-full text-center text-sm text-gray-600">
         <RouterLink
          to="/reset-password"
          class="font-medium text-blue-600 underline-offset-2 hover:text-blue-800 hover:underline"
          > {{ t('loginPage.forgotPasswordLink') }} </RouterLink
        >
      </p>

      <p class="mt-4 w-full text-center text-sm text-gray-600">
         {{ t('loginPage.noAccountPrompt') }}
         <RouterLink
          to="/signup"
          class="font-medium text-blue-600 underline-offset-2 hover:text-blue-800 hover:underline"
          > {{ t('loginPage.signUpLink') }} </RouterLink
        >
      </p>

    </div>

  </div>

</template>

<script setup lang="ts">
import { User, Key, KeyRound } from 'lucide-vue-next'

import { Button } from '@packages/ui'
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { login } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const username = ref('')
const password = ref('')
const isLoading = ref(false)
const router = useRouter()
const route = useRoute()
const auth = useAuth()
const { showError, clearError } = useError()
const { t, locale } = useI18n()

useSeoHead({ title: t('loginPage.title') })

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

