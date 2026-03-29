<template>

  <div class="auth-glass-page-shell">

    <AuthGlassCard>

      <h2 class="auth-glass-title auth-glass-title--spaced">
         {{ t('changePassword.title') }}
      </h2>
       <!-- Step 1: Current Password -->
      <form
        v-if="!verificationId"
        class="w-full space-y-6"
        @submit.prevent="initiatePasswordChange"
      >

        <div>
           <label for="currentPassword" class="block text-sm font-medium text-blue-900 mb-2">{{
            t('changePassword.currentPasswordLabel')
          }}</label
          >
          <div class="relative">
             <input
              id="currentPassword"
              v-model="currentPassword"
              type="password"
              required
              class="input-field w-full text-base h-10 pl-3 pr-10"
              :disabled="isLoading"
              :placeholder="t('changePassword.currentPasswordPlaceholder')"
            /> <Key class="input-field-trailing-icon" aria-hidden="true" />
          </div>

        </div>
         <button
          type="submit"
          class="auth-form-wide-submit ui-btn--neutral-slate h-8"
          :disabled="isLoading || !currentPassword"
          :class="{ 'opacity-75 cursor-not-allowed': isLoading }"
        >
           <template v-if="isLoading"
            > <Loader2 class="animate-spin -ml-1 mr-3 h-5 w-5 text-current" /> {{
              t('changePassword.verifying')
            }} </template
          > <template v-else> {{ t('changePassword.continueButton') }} </template> </button
        >
      </form>
       <!-- Step 2: Verification Code and New Password -->
      <form v-else class="w-full space-y-6" @submit.prevent="completePasswordChange">

        <div>
           <label for="verificationCode" class="block text-sm font-medium text-blue-900 mb-2">{{
            t('changePassword.verificationCodeLabel')
          }}</label
          >
          <div class="relative">
             <input
              id="verificationCode"
              v-model="verificationCode"
              type="text"
              required
              :placeholder="t('changePassword.verificationCodePlaceholder')"
              class="input-field w-full text-base h-10 pl-3 pr-10"
              :disabled="isLoading"
            /> <Mail class="input-field-trailing-icon" aria-hidden="true" />
          </div>

        </div>

        <div>
           <label for="newPassword" class="block text-sm font-medium text-blue-900 mb-2">{{
            t('changePassword.newPasswordLabel')
          }}</label
          >
          <div class="relative">
             <input
              id="newPassword"
              v-model="newPassword"
              type="password"
              required
              class="input-field w-full text-base h-10 pl-3 pr-10"
              :disabled="isLoading"
              :placeholder="t('changePassword.newPasswordPlaceholder')"
            /> <Key class="input-field-trailing-icon" aria-hidden="true" />
          </div>

        </div>

        <div>
           <label for="confirmPassword" class="block text-sm font-medium text-blue-900 mb-2">{{
            t('changePassword.confirmPasswordLabel')
          }}</label
          >
          <div class="relative">
             <input
              id="confirmPassword"
              v-model="confirmPassword"
              type="password"
              required
              class="input-field w-full text-base h-10 pl-3 pr-10"
              :disabled="isLoading"
              :placeholder="t('changePassword.confirmPasswordPlaceholder')"
            /> <Key class="input-field-trailing-icon" aria-hidden="true" />
          </div>

        </div>

        <div
          v-if="showValidationErrors"
          class="mt-4 p-3 bg-yellow-50 border border-yellow-200 text-yellow-800 rounded-md"
        >

          <p class="font-medium mb-1"> {{ t('changePassword.validationErrorTitle') }} </p>

          <ul class="list-disc list-inside">

            <li v-for="error in passwordValidationErrors" :key="error" class="text-sm">
               {{ error }}
            </li>

          </ul>

        </div>

        <div class="flex gap-3">
           <button
            type="button"
            class="ui-btn--neutral-muted flex-1 h-8"
            :disabled="isLoading"
            @click="resetForm"
          >
             {{ t('changePassword.startOverButton') }} </button
          > <button
            type="submit"
            class="ui-btn--auth-signup flex-1 h-8"
            :disabled="isLoading || !isValidPasswordChange"
          >
             <template v-if="isLoading"
              > <Loader2 class="animate-spin -ml-1 mr-3 h-5 w-5 text-current" /> {{
                t('changePassword.changingPassword')
              }} </template
            > <template v-else> {{ t('changePassword.changePasswordButton') }} </template> </button
          >
        </div>

      </form>

      <div
        v-if="success"
        class="w-full mt-4 p-3 bg-emerald-100 border border-emerald-200 text-emerald-700 rounded-md text-center"
      >
         {{ success }}
      </div>

      <p class="mt-4 text-sm text-white text-center w-full">
         <RouterLink to="/profile" class="font-medium text-blue-900 hover:text-blue-700"
          > {{ t('changePassword.backToProfile') }} </RouterLink
        >
      </p>

    </AuthGlassCard>

  </div>

</template>

<script setup lang="ts">
import { Loader2, Key, Mail, KeyRound } from 'lucide-vue-next'
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

import { AuthGlassCard } from '@packages/ui'
import { api } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const { t, locale } = useI18n()
useSeoHead({ title: t('changePassword.title') })

const router = useRouter()
const auth = useAuth()
const { showError, clearError } = useError()

// Form state
const currentPassword = ref('')
const verificationId = ref('')
const verificationCode = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const isLoading = ref(false)
const success = ref('')

// Password validation
const passwordValidationErrors = computed(() => {
  const errors = []

  if (newPassword.value || confirmPassword.value) {
    if (newPassword.value.length < 8) {
      errors.push(t('changePassword.validationErrors.length'))
    }
    if (newPassword.value !== confirmPassword.value) {
      errors.push(t('changePassword.validationErrors.match'))
    }
  }

  return errors
})

const showValidationErrors = computed(() => {
  return (newPassword.value || confirmPassword.value) && passwordValidationErrors.value.length > 0
})

const isValidPasswordChange = computed(() => {
  return (
    verificationCode.value &&
    newPassword.value &&
    confirmPassword.value &&
    newPassword.value === confirmPassword.value &&
    newPassword.value.length >= 8
  )
})

// Methods
const initiatePasswordChange = async () => {
  try {
    isLoading.value = true
    clearError()
    success.value = ''

    const response = await api.post('/auth/change-password/initiate', {
      current_password: currentPassword.value,
    })

    verificationId.value = response.data.verification_id
    success.value = t('changePassword.verificationCodeSent')
  } catch (err) {
    if (err.response?.status === 401) {
      showError(t('changePassword.wrongPassword'))
    } else {
      showError(err.response?.data?.error || t('changePassword.initiateFailedError'))
    }
    verificationId.value = ''
  } finally {
    isLoading.value = false
  }
}

const completePasswordChange = async () => {
  if (!isValidPasswordChange.value) return

  try {
    isLoading.value = true
    clearError()
    success.value = ''

    await api.post('/auth/change-password/complete', {
      verification_id: verificationId.value,
      verification_code: verificationCode.value,
      new_password: newPassword.value,
    })

    success.value = t('changePassword.successMessage')

    // Log out after successful password change
    setTimeout(() => {
      auth.logout()
      router.push('/login')
    }, 2000)
  } catch (err) {
    if (err.response?.status === 400) {
      showError(t('changePassword.invalidCode'))
    } else {
      showError(err.response?.data?.error || t('changePassword.completeFailedError'))
    }
  } finally {
    isLoading.value = false
  }
}

const resetForm = () => {
  verificationId.value = ''
  verificationCode.value = ''
  newPassword.value = ''
  confirmPassword.value = ''
  currentPassword.value = ''
  clearError()
  success.value = ''
}
</script>

