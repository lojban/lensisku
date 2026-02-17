<template>
  <div class="w-full min-h-[calc(100vh-12rem)] flex items-center justify-center relative">
    <div class="w-full max-w-md p-8 mx-auto rounded-2xl border border-blue-200 flex-shrink-0
             backdrop-blur-xl bg-blue-50/90 shadow-lg 
             transition-all duration-300 hover:shadow-xl
             flex flex-col items-center space-y-6">
      <div v-if="!sessionId" class="w-full space-y-6">
        <h2 class="text-2xl sm:text-3xl font-bold text-blue-900 text-center">
          {{ t('passwordReset.resetTitle') }}
        </h2>
        <form class="space-y-6 w-full" @submit.prevent="requestPasswordReset">
          <div>
            <label for="email" class="block text-sm font-medium text-blue-700">{{ t('passwordReset.emailLabel') }}</label>
            <div class="relative">
              <input id="email" v-model="email" type="email" required class="input-field w-full h-10 pl-3 pr-10"
                :disabled="isLoading" :placeholder="t('passwordReset.emailPlaceholder')">
              <Mail class="h-5 w-5 text-gray-400 absolute right-3 top-1/2 -translate-y-1/2" />
            </div>
          </div>

          <button type="submit"
            class="w-full flex justify-center items-center btn-aqua-slate h-8 gap-2 py-3 rounded-full text-lg font-semibold transition-all"
            :disabled="isLoading || !email">
            <template v-if="isLoading">
              <Loader2 class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" />
              {{ t('passwordReset.sendingRequest') }}
            </template>
            <template v-else>
              {{ t('passwordReset.requestButton') }}
            </template>
          </button>
        </form>

        <!-- Error Display -->
        <AlertComponent v-if="error" type="error" class="mx-auto">
          {{ error.message }}
        </AlertComponent>

        <!-- Success Message -->
        <AlertComponent v-if="requestSent" type="success" class="mx-auto">
          {{ t('passwordReset.requestSentSuccess') }}
        </AlertComponent>
      </div>

      <!-- Reset Password Form -->
      <div v-else class="w-full">
        <h2 class="text-2xl sm:text-3xl font-bold text-blue-900 text-center">
          {{ t('passwordReset.setNewPasswordTitle') }}
        </h2>
        <form class="space-y-6 w-full" @submit.prevent="resetPassword">
          <div>
            <label for="newPassword" class="block text-sm font-medium text-blue-700">{{ t('passwordReset.newPasswordLabel') }}</label>
            <div class="relative">
              <input id="newPassword" v-model="newPassword" type="password" required
                class="input-field w-full h-10 pl-3 pr-10" :disabled="isLoading" :placeholder="t('passwordReset.newPasswordPlaceholder')">
              <Key class="h-5 w-5 text-gray-400 absolute right-3 top-1/2 -translate-y-1/2" />
            </div>
          </div>

          <div>
            <label for="confirmPassword" class="block text-sm font-medium text-blue-700">{{ t('passwordReset.confirmPasswordLabel') }}</label>
            <div class="relative">
              <input id="confirmPassword" v-model="confirmPassword" type="password" required
                class="input-field w-full h-10 pl-3 pr-10" :disabled="isLoading" :placeholder="t('passwordReset.confirmPasswordPlaceholder')">
              <Key class="h-5 w-5 text-gray-400 absolute right-3 top-1/2 -translate-y-1/2" />
            </div>
          </div>

          <div v-if="showValidationErrors"
            class="mt-4 p-3 bg-yellow-50 border border-yellow-200 text-yellow-800 rounded-md">
            <p class="font-medium mb-1">
              {{ t('passwordReset.validationErrorTitle') }}
            </p>
            <ul class="list-disc list-inside">
              <li v-for="validationError in passwordValidationErrors" :key="validationError" class="text-sm">
                {{ validationError }}
              </li>
            </ul>
          </div>

          <button 
            type="submit" 
            class="w-full flex justify-center items-center btn-aqua-slate h-8 gap-2 py-3 rounded-full text-lg font-semibold transition-all" 
            :disabled="isLoading || !isValidPasswordReset"
          >
            <template v-if="isLoading">
              <Loader2 class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" />
              {{ t('passwordReset.resettingPassword') }}
            </template>
            <template v-else>
              {{ t('passwordReset.resetPasswordButton') }}
            </template>
          </button>
        </form>

        <!-- Error Display -->
        <div v-if="error" class="mb-4 text-center text-red-600">
          {{ error.message }}
        </div>

        <!-- Success Message -->
        <div v-if="resetSuccess"
          class="w-full mt-4 p-3 bg-emerald-100 border border-emerald-200 text-emerald-700 rounded-md text-center">
          {{ t('passwordReset.resetSuccess') }}
          <RouterLink to="/login" class="text-green-700 hover:text-green-800 font-medium">
            {{ t('passwordReset.loginLink') }}
          </RouterLink>
          .
        </div>
      </div>
      <!-- Back to Login Link -->
      <p class="text-sm text-white text-center w-full">
        <RouterLink to="/login" class="font-medium text-blue-900 hover:text-blue-700">
          {{ t('passwordReset.backToLogin') }}
        </RouterLink>
      </p>
    </div>
  </div>
</template>

<script setup>
import { Loader2, Key, Mail } from 'lucide-vue-next'
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { requestPasswordReset as apiRequestPasswordReset, restorePassword as apiRestorePassword, api } from '@/api'
import AlertComponent from '@/components/AlertComponent.vue'
import { useSeoHead } from '@/composables/useSeoHead'

const route = useRoute()
const router = useRouter()
const { t, locale } = useI18n()

const email = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const error = ref(null)
const isLoading = ref(false)
const requestSent = ref(false)
const resetSuccess = ref(false)

// Get session_id and token from URL if they exist
const sessionId = ref(route.query.session_id || '')
const token = ref(route.query.token || '')

const isValidPasswordReset = computed(() => {
  return (
    newPassword.value &&
    confirmPassword.value &&
    newPassword.value === confirmPassword.value &&
    newPassword.value.length >= 8
  )
})

const passwordValidationErrors = computed(() => {
  const errors = []

  if (newPassword.value === '' && confirmPassword.value === '') return errors
  if (!newPassword.value) {
    errors.push(t('passwordReset.validationErrors.required'))
  } else {
    if (newPassword.value.length < 8) {
      errors.push(t('passwordReset.validationErrors.length'))
    }
  }

  if (!confirmPassword.value) {
    errors.push(t('passwordReset.validationErrors.confirmationRequired'))
  } else if (newPassword.value !== confirmPassword.value) {
    errors.push(t('passwordReset.validationErrors.match'))
  }

  return errors
})

const showValidationErrors = computed(() => {
  return (newPassword.value || confirmPassword.value) && passwordValidationErrors.value.length > 0
})

const requestPasswordReset = async () => {
  try {
    isLoading.value = true
    error.value = null

    const response = await apiRequestPasswordReset(email.value)

    if (response.data.success) {
      requestSent.value = true
      email.value = ''
    } else {
      error.value = { message: response.data.message }
    }
  } catch (err) {
    if (err.response?.status === 429) {
      error.value = { message: t('passwordReset.rateLimitError') }
    } else {
      error.value = { message: err.response?.data?.message || t('passwordReset.requestFailedError') }
    }
  } finally {
    isLoading.value = false
  }
}

const resetPassword = async () => {
  if (!isValidPasswordReset.value) {
    error.value = { message: t('passwordReset.invalidResetDataError') }
    return
  }

  try {
    isLoading.value = true
    error.value = null;

    const response = await apiRestorePassword({
      token: token.value,
      session_id: sessionId.value,
      new_password: newPassword.value,
    })

    if (response.data.success) {
      resetSuccess.value = true
      // Redirect to login after 2 seconds
      setTimeout(() => {
        router.push('/login')
      }, 2000)
    } else {
      error.value = { message: response.data.message }
    }
  } catch (err) {
    error.value = { message: err.response?.data?.error || t('passwordReset.resetFailedError') }
  } finally {
    isLoading.value = false
  }
}

useSeoHead({ title: sessionId.value ? t('passwordReset.setNewPasswordTitle') : t('passwordReset.resetTitle') }, locale.value)
</script>
