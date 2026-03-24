<template>

  <div class="w-full min-h-[calc(100vh-12rem)] flex items-center justify-center relative">
     <BackgroundComponent
      id="login-background"
      classes="fixed inset-0 w-screen h-screen bg-cover bg-center bg-no-repeat"
    />
    <div
      class="w-full max-w-md p-8 mx-4 rounded-2xl border border-white/40 flex-shrink-0 backdrop-blur-xl bg-black/25 shadow-lg transition-all duration-300 hover:shadow-xl flex flex-col items-center"
    >

      <h2
        class="text-2xl sm:text-3xl font-bold mb-6 text-white text-center [text-shadow:0_1px_3px_rgba(0,0,0,0.9),0_0_12px_rgba(0,0,0,0.6)]"
      >
         {{ t('signupPage.title') }}
      </h2>

      <form class="space-y-6 w-full" @submit.prevent="performSignup">

        <div>
           <label
            for="username"
            class="block text-sm font-medium text-white [text-shadow:0_1px_2px_rgba(0,0,0,0.9)]"
            >{{ t('signupPage.usernameLabel') }}</label
          >
          <div class="relative">
             <input
              id="username"
              v-model="username"
              type="text"
              required
              class="input-field w-full text-base h-10 pl-3 pr-10"
              :disabled="isLoading"
            /> <User class="h-5 w-5 text-gray-400 absolute right-3 top-1/2 -translate-y-1/2" />
          </div>

        </div>

        <div>
           <label
            for="email"
            class="block text-sm font-medium text-white [text-shadow:0_1px_2px_rgba(0,0,0,0.9)]"
            >{{ t('signupPage.emailLabel') }}</label
          >
          <div class="relative">
             <input
              id="email"
              v-model="email"
              type="email"
              required
              class="input-field w-full text-base h-10 pl-3 pr-10"
              :disabled="isLoading"
            /> <Mail class="h-5 w-5 text-gray-400 absolute right-3 top-1/2 -translate-y-1/2" />
          </div>

        </div>

        <div>
           <label
            for="password"
            class="block text-sm font-medium text-white [text-shadow:0_1px_2px_rgba(0,0,0,0.9)]"
            >{{ t('signupPage.passwordLabel') }}</label
          >
          <div class="relative">
             <input
              id="password"
              v-model="password"
              type="password"
              required
              class="input-field w-full text-base h-10 pl-3 pr-10"
              :disabled="isLoading"
            /> <Key class="h-5 w-5 text-gray-400 absolute right-3 top-1/2 -translate-y-1/2" />
          </div>

        </div>

        <div>
           <button
            type="submit"
            class="w-full flex justify-center items-center btn-aqua-teal h-8 gap-2 py-3 rounded-full text-lg font-semibold transition-all"
            :disabled="isLoading"
            :class="{ 'opacity-75 cursor-not-allowed': isLoading }"
          >
             <template v-if="isLoading"
              > <Loader2 class="animate-spin h-5 w-5" /> <span>{{
                t('signupPage.creatingAccount')
              }}</span
              > </template
            > <template v-else
              > <Plus class="h-5 w-5" /> <span>{{ t('signupPage.createAccountButton') }}</span
              > </template
            > </button
          >
        </div>

      </form>

      <p class="mt-4 text-sm text-white text-center w-full [text-shadow:0_1px_2px_rgba(0,0,0,0.9)]">
         {{ t('signupPage.haveAccountPrompt') }} <RouterLink
          to="/login"
          class="font-medium text-white hover:text-green-200"
          > {{ t('signupPage.loginLink') }} </RouterLink
        >
      </p>

    </div>

  </div>

</template>

<script setup lang="ts">
import { Loader2, User, Mail, Key, Plus } from 'lucide-vue-next'
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { signup } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

import BackgroundComponent from '../components/BackgroundComponent.vue'

const username = ref('')
const email = ref('')
const password = ref('')
const isLoading = ref(false)
const router = useRouter()
const auth = useAuth()
const { showError, clearError } = useError()
const { t, locale } = useI18n()

useSeoHead({ title: t('signupPage.title') })

const performSignup = async () => {
  clearError()
  isLoading.value = true
  try {
    const response = await signup({
      username: username.value,
      email: email.value,
      password: password.value,
    })
    if (response.data.token) {
      // Assuming signup response provides tokens needed for login
      auth.login(response.data.token, response.data.refresh_token, username.value) // Adjust if API response differs
      const redirectPath = sessionStorage.getItem('redirectPath')
      sessionStorage.removeItem('redirectPath')
      router.push(redirectPath || '/') // Redirect to stored path or home
    }
  } catch (err) {
    if (err.response?.status === 409 || err.response?.data?.error === 'user_exists') {
      showError(t('signupPage.userExists'))
    } else if (err.response?.data?.error_description) {
      showError(err.response.data.error_description)
    } else if (err.response?.data?.error) {
      showError(err.response.data.error)
    } else {
      showError(t('signupPage.signupError'))
    }
  } finally {
    isLoading.value = false
  }
}
</script>

