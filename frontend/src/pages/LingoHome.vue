<template>
  <main class="lingo-home flex min-h-screen flex-col items-center justify-center gap-6 px-4 py-8">
    <div class="mx-auto flex w-full max-w-[988px] flex-col items-center gap-6 lg:flex-row">
      <div class="flex h-48 w-48 shrink-0 items-center justify-center rounded-2xl bg-green-100 lg:h-80 lg:w-80">
        <GraduationCap class="h-24 w-24 text-green-600 lg:h-32 lg:w-32" />
      </div>

      <div class="flex flex-col items-center gap-8">
        <h1 class="max-w-[480px] text-center text-xl font-bold text-slate-700 lg:text-3xl">
          {{ t('lingo.heroTitle', 'Learn, practice and master with Lingo.') }}
        </h1>

        <div class="flex w-full max-w-[330px] flex-col items-center gap-3">
          <template v-if="auth.state.isLoggedIn">
            <RouterLink to="/lingo/courses" class="btn-aqua-teal w-full justify-center text-base">
              {{ t('lingo.continueLearning', 'Continue Learning') }}
            </RouterLink>
          </template>
          <template v-else>
            <RouterLink to="/lingo/signup" class="btn-aqua-teal w-full justify-center text-base">
              {{ t('lingo.getStarted', 'Get Started') }}
            </RouterLink>
            <RouterLink to="/lingo/login" class="btn-aqua-white w-full justify-center text-base">
              {{ t('lingo.haveAccount', 'I already have an account') }}
            </RouterLink>
          </template>
        </div>
      </div>
    </div>
  </main>
</template>

<script setup>
import { GraduationCap } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { useAuth } from '@/composables/useAuth'
import { useSeoHead } from '@/composables/useSeoHead'

const auth = useAuth()
const { t, locale } = useI18n()

useSeoHead({ title: t('lingo.appName', 'Lingo') }, locale.value)
</script>
