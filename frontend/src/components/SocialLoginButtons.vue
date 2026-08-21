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
import { Chrome, Github, type LucideIcon } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import {
  beginSocialLogin,
  fetchConfiguredSocialProviders,
  type SocialProviderId,
} from '@/composables/useSocialLogin'

const props = defineProps<{
  returnTo?: string
}>()

const { t } = useI18n()
const providers = ref<string[]>([])
const starting = ref<string | null>(null)
const error = ref('')

const catalog: Record<
  SocialProviderId,
  { id: SocialProviderId; icon: LucideIcon; labelKey: string }
> = {
  github: { id: 'github', icon: Github, labelKey: 'oauth.continueWithGithub' },
  google: { id: 'google', icon: Chrome, labelKey: 'oauth.continueWithGoogle' },
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
