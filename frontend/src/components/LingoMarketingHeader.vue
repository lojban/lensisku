<template>
  <header class="lingo-marketing-header border-b-2 border-slate-200 bg-white px-4">
    <div class="mx-auto flex h-20 max-w-screen-lg items-center justify-between">
      <RouterLink to="/lingo/courses" class="flex items-center gap-x-3 py-4">
        <GraduationCap class="h-10 w-10 text-green-600" />
        <h1 class="text-2xl font-extrabold tracking-wide text-green-600">
          {{ t('lingo.appName') }}
        </h1>
      </RouterLink>

      <div class="flex items-center gap-3">
        <template v-if="auth.state.isLoggedIn">
          <RouterLink to="/profile" class="text-sm font-medium text-slate-600 hover:text-green-600">
            {{ auth.state.username }}
          </RouterLink>
          <button
            type="button"
            class="rounded-lg px-3 py-1.5 text-sm text-slate-500 hover:bg-slate-100"
            @click="handleSignOut"
          >
            {{ t('lingo.signOut') }}
          </button>
        </template>
        <template v-else>
          <RouterLink to="/lingo/login" class="btn-aqua-zinc text-sm">
            {{ t('lingo.login') }}
          </RouterLink>
        </template>
      </div>
    </div>
  </header>
</template>

<script setup>
import { GraduationCap } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useAuth } from '@/composables/useAuth'

const auth = useAuth()
const router = useRouter()
const { t } = useI18n()

function handleSignOut() {
  auth.logout()
  router.push('/lingo/courses')
}
</script>
