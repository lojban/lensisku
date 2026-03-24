<template>

  <div class="w-full min-h-[calc(100vh-12rem)] flex items-center justify-center relative">
     <BackgroundComponent
      id="login-background"
      classes="fixed inset-0 w-screen h-screen bg-cover bg-center bg-no-repeat"
    />
    <div
      class="w-full max-w-md p-8 mx-4 rounded-2xl border border-white/40 backdrop-blur-xl bg-black/25 shadow-lg transition-all duration-300 hover:shadow-xl flex flex-col items-center"
    >

      <h2
        class="text-2xl sm:text-3xl font-bold mb-6 text-white text-center [text-shadow:0_1px_3px_rgba(0,0,0,0.9),0_0_12px_rgba(0,0,0,0.6)]"
      >
         {{ t('loginPage.title') }}
      </h2>

      <form class="space-y-6 w-full" @submit.prevent="performLogin">

        <div>
           <label
            for="username"
            class="block text-sm font-medium text-white [text-shadow:0_1px_2px_rgba(0,0,0,0.9)]"
            >{{ t('loginPage.usernameLabel') }}</label
          >
          <div class="relative">
             <input
              id="username"
              v-model="username"
              type="text"
              required
              class="input-field w-full h-auto text-base pl-3 pr-10"
              :disabled="isLoading"
              :placeholder="t('loginPage.usernamePlaceholder')"
            /> <User class="h-5 w-5 text-gray-400 absolute right-3 top-1/2 -translate-y-1/2" />
          </div>

        </div>

        <div>
           <label
            for="password"
            class="block text-sm font-medium text-white [text-shadow:0_1px_2px_rgba(0,0,0,0.9)]"
            >{{ t('loginPage.passwordLabel') }}</label
          >
          <div class="relative">
             <input
              id="password"
              v-model="password"
              type="password"
              required
              class="input-field w-full h-auto text-base pl-3 pr-10"
              :disabled="isLoading"
              :placeholder="t('loginPage.passwordPlaceholder')"
            /> <Key class="h-5 w-5 text-gray-400 absolute right-3 top-1/2 -translate-y-1/2" />
          </div>

        </div>

        <div>
           <button
            type="submit"
            class="w-full flex justify-center items-center btn-aqua-orange h-8 gap-2 py-3 rounded-full text-lg font-semibold transition-all"
            :disabled="isLoading"
            :class="{ 'opacity-75 cursor-not-allowed': isLoading }"
          >
             <template v-if="isLoading"
              > <Loader2 class="animate-spin h-5 w-5" /> <span>{{
                t('loginPage.authenticating')
              }}</span
              > </template
            > <template v-else
              > <KeyRound class="h-5 w-5" /> <span>{{ t('loginPage.loginButton') }}</span
              > </template
            > </button
          >
        </div>

      </form>

      <p class="mt-4 text-sm text-white text-center w-full [text-shadow:0_1px_2px_rgba(0,0,0,0.9)]">
         <RouterLink to="/reset-password" class="font-medium text-white hover:text-blue-200"
          > {{ t('loginPage.forgotPasswordLink') }} </RouterLink
        >
      </p>

      <p class="mt-4 text-sm text-white text-center w-full [text-shadow:0_1px_2px_rgba(0,0,0,0.9)]">
         {{ t('loginPage.noAccountPrompt') }} <RouterLink
          to="/signup"
          class="font-medium text-white hover:text-blue-200"
          > {{ t('loginPage.signUpLink') }} </RouterLink
        >
      </p>

    </div>

  </div>

</template>

<script setup lang="ts">
import { Loader2, User, Key, KeyRound } from 'lucide-vue-next'
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { login } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

import BackgroundComponent from '../components/BackgroundComponent.vue'

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

