<template>

  <div
    class="lingo-auth flex min-h-screen flex-col items-center justify-center bg-slate-100 px-4 py-8"
  >
     <RouterLink to="/lingo/courses" class="mb-6 flex items-center gap-x-3"
      > <GraduationCap class="h-12 w-12 text-green-600" />
      <h1 class="text-2xl font-extrabold tracking-wide text-green-600">{{ t('lingo.appName') }}</h1>
       </RouterLink
    >
    <h2 class="mb-6 text-xl font-semibold text-slate-800"> {{ t('signupPage.title') }} </h2>

    <form class="flex w-full max-w-[400px] flex-col gap-4" @submit.prevent="performSignup">
       <input
        v-model="username"
        type="text"
        :placeholder="t('signupPage.usernameLabel')"
        required
        class="input-field w-full rounded-lg px-4 py-2.5"
        :disabled="isLoading"
      /> <input
        v-model="email"
        type="email"
        :placeholder="t('signupPage.emailLabel')"
        required
        class="input-field w-full rounded-lg px-4 py-2.5"
        :disabled="isLoading"
      /> <input
        v-model="password"
        type="password"
        :placeholder="t('signupPage.passwordLabel')"
        required
        minlength="6"
        class="input-field w-full rounded-lg px-4 py-2.5"
        :disabled="isLoading"
      />
      <p v-if="error" class="text-sm text-red-600" role="alert">{{ error }}</p>
       <button type="submit" class="ui-btn--auth-signup h-12 w-full text-base" :disabled="isLoading">
         <Loader2 v-if="isLoading" class="mx-auto h-5 w-5 animate-spin" /> <span v-else>{{
          t('signupPage.createAccountButton')
        }}</span
        > </button
      >
    </form>

    <p class="mt-6 text-center text-sm text-slate-500">
       {{ t('signupPage.haveAccountPrompt') }} <RouterLink
        to="/lingo/login"
        class="font-medium text-green-600 hover:underline"
        > {{ t('signupPage.loginLink') }} </RouterLink
      >
    </p>

  </div>

</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { GraduationCap, Loader2 } from 'lucide-vue-next'
import { signup } from '@/api'
import { useAuth } from '@/composables/useAuth'
import { useSeoHead } from '@/composables/useSeoHead'
import { queryStr } from '@/utils/routeQuery'

const username = ref('')
const email = ref('')
const password = ref('')
const error = ref('')
const isLoading = ref(false)

const router = useRouter()
const route = useRoute()
const auth = useAuth()
const { t, locale } = useI18n()

const returnTo = computed(() => queryStr(route.query.returnTo) || '/lingo/courses')

useSeoHead({ title: t('signupPage.title') })

async function performSignup() {
  error.value = ''
  isLoading.value = true
  try {
    const response = await signup({
      username: username.value,
      email: email.value,
      password: password.value,
    })
    const token = response.data.access_token ?? response.data.token
    const refresh = response.data.refresh_token
    if (token) {
      auth.login(token, refresh, username.value)
      router.push(returnTo.value)
    }
  } catch (err) {
    if (err.response?.status === 409 || err.response?.data?.error === 'user_exists') {
      error.value = t('signupPage.userExists')
    } else {
      error.value =
        err.response?.data?.error_description ||
        err.response?.data?.message ||
        t('signupPage.signupError')
    }
  } finally {
    isLoading.value = false
  }
}
</script>

