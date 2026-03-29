<template>

  <div class="auth-page-shell">
    <AuthFormCard>

      <h2 class="auth-form-title">
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
            /> <User class="input-field-trailing-icon" aria-hidden="true" />
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
            /> <Mail class="input-field-trailing-icon" aria-hidden="true" />
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
              :type="showPassword ? 'text' : 'password'"
              required
              autocomplete="new-password"
              class="input-field h-10 w-full pl-3 pr-10 text-base"
              :disabled="isLoading"
            />
            <button
              type="button"
              class="input-field-password-toggle"
              :aria-label="
                showPassword ? t('signupPage.hidePassword') : t('signupPage.showPassword')
              "
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

    </AuthFormCard>

  </div>

</template>

<script setup lang="ts">
import { User, Mail, Eye, EyeOff, Plus } from 'lucide-vue-next'

import { AuthFormCard, Button } from '@packages/ui'
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
const showPassword = ref(false)
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

