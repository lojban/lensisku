<template>
  <div v-if="providers.length" class="auth-social-stack">
    <div class="auth-social-divider" role="separator">
      <span class="auth-social-divider-line" />
      <span>{{ t('oauth.or') }}</span>
      <span class="auth-social-divider-line" />
    </div>
    <p v-if="error" class="text-center text-sm text-red-600" role="alert">{{ error }}</p>
    <Button
      v-for="provider in visibleProviders"
      :key="provider.id"
      variant="neutral"
      size="lg"
      type="button"
      class="w-full"
      :loading="starting === provider.id"
      :disabled="!!starting"
      @click="start(provider.id)"
    >
      <template #icon>
        <component :is="provider.icon" class="h-5 w-5 shrink-0" aria-hidden="true" />
      </template>
      {{ t(provider.labelKey) }}
    </Button>
  </div>
</template>

<script setup lang="ts">
import { Button } from '@packages/ui'
import { computed, h, onMounted, ref, type FunctionalComponent } from 'vue'
import { useI18n } from 'vue-i18n'

import {
  beginSocialLogin,
  fetchConfiguredSocialProviders,
  type SocialProviderId,
} from '@/composables/useSocialLogin'

const GithubIcon: FunctionalComponent = (props) =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor', 'aria-hidden': 'true', ...props }, [
    h('path', {
      d: 'M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12',
    }),
  ])

const GoogleIcon: FunctionalComponent = (props) =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor', 'aria-hidden': 'true', ...props }, [
    h('path', {
      d: 'M21.35 11.1h-9.18v2.96h5.27c-.23 1.25-1.4 3.66-5.27 3.66-3.17 0-5.76-2.62-5.76-5.85s2.59-5.85 5.76-5.85c1.8 0 3.01.77 3.7 1.43l2.52-2.44C16.54 3.54 14.48 2.6 12.17 2.6 7.02 2.6 2.86 6.74 2.86 11.87s4.16 9.27 9.31 9.27c5.37 0 8.92-3.77 8.92-9.08 0-.61-.07-1.07-.16-1.96z',
    }),
  ])

const props = defineProps<{
  returnTo?: string
}>()

const { t } = useI18n()
const providers = ref<string[]>([])
const starting = ref<string | null>(null)
const error = ref('')

const catalog: Record<
  SocialProviderId,
  { id: SocialProviderId; icon: FunctionalComponent; labelKey: string }
> = {
  github: { id: 'github', icon: GithubIcon, labelKey: 'oauth.continueWithGithub' },
  google: { id: 'google', icon: GoogleIcon, labelKey: 'oauth.continueWithGoogle' },
}

const visibleProviders = computed(() =>
  providers.value.flatMap((id) => {
    if (id === 'github' || id === 'google') {
      return [catalog[id]]
    }
    return []
  })
)

onMounted(async () => {
  try {
    providers.value = await fetchConfiguredSocialProviders()
  } catch {
    providers.value = []
  }
})

async function start(provider: string) {
  error.value = ''
  starting.value = provider
  try {
    await beginSocialLogin(provider, props.returnTo)
  } catch (err: unknown) {
    const status = (err as { response?: { status?: number } }).response?.status
    error.value = status === 503 ? t('oauth.notConfigured') : t('oauth.failed')
    starting.value = null
  }
}
</script>
