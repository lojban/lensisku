<template>

  <div
    class="flex min-h-full w-full flex-col items-center justify-center px-4 py-10 sm:px-6 sm:py-12"
  >
    <div
      class="card-elevated flex w-full max-w-md flex-shrink-0 flex-col items-center rounded-2xl border border-gray-200/90 bg-white/95 p-8 ring-1 ring-gray-900/5 backdrop-blur-sm"
    >

      <h2 class="mb-6 text-center text-2xl font-bold text-gray-900 sm:text-3xl">
         {{ t('signupPage.title') }}
      </h2>

      <form class="w-full space-y-6" @submit.prevent="performSignup">

        <div>
           <label
            for="username"
            class="mb-1 block text-sm font-medium text-gray-700"
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
            class="mb-1 block text-sm font-medium text-gray-700"
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
            class="mb-1 block text-sm font-medium text-gray-700"
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
          <Button
            variant="palette-teal"
            size="lg"
            type="submit"
            class="w-full"
            :loading="isLoading"
            :disabled="isLoading"
          >
            <template #icon>
              <Plus class="h-6 w-6 shrink-0" />
            </template>
            {{
              isLoading ? t('signupPage.creatingAccount') : t('signupPage.createAccountButton')
            }}
          </Button>
        </div>

      </form>

      <p class="mt-4 w-full text-center text-sm text-gray-600">
         {{ t('signupPage.haveAccountPrompt') }}
         <RouterLink
          to="/login"
          class="font-medium text-blue-600 underline-offset-2 hover:text-blue-800 hover:underline"
          > {{ t('signupPage.loginLink') }} </RouterLink
        >
      </p>

    </div>

  </div>

</template>

<script setup lang="ts">
import { User, Mail, Key, Plus } from 'lucide-vue-next'

import { Button } from '@packages/ui'
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { signup } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

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

