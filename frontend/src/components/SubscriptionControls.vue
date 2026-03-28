<template>
   <button
    v-if="auth.state.isLoggedIn"
    :class="[
      isSubscribed ? 'ui-btn--toggle active' : 'ui-btn--toggle inactive',
      { 'cursor-wait': isLoading },
    ]"
    :disabled="isLoading"
    @click="toggleSubscription"
  >
     <span class="inline-block align-middle"
      > <BellRing v-if="isSubscribed" class="h-4 w-4" /> <BellOff v-else class="h-4 w-4" /> </span
    > {{
      isSubscribed
        ? t('components.subscriptionControls.gettingAlerts')
        : t('components.subscriptionControls.notGettingAlerts')
    }} </button
  >
</template>

<script setup lang="ts">
import { BellOff, BellRing } from 'lucide-vue-next'
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import { getSubscriptionState, subscribeToValsi, unsubscribeFromValsi } from '@/api'

const { t } = useI18n()

const props = defineProps({
  valsiId: {
    type: Number,
    required: true,
  },
  auth: {
    type: Object,
    required: true,
  },
})

const isSubscribed = ref(false)
const isLoading = ref(false)
const error = ref(null)

const fetchState = async () => {
  if (!props.auth.state.isLoggedIn) return

  try {
    const response = await getSubscriptionState(props.valsiId)
    isSubscribed.value = response.data.is_subscribed
  } catch (err) {
    error.value = t('components.subscriptionControls.loadError')
    console.error(err)
  }
}

const toggleSubscription = async () => {
  isLoading.value = true
  error.value = null

  try {
    if (isSubscribed.value) {
      await unsubscribeFromValsi(props.valsiId, 'edit')
      isSubscribed.value = false
    } else {
      await subscribeToValsi(props.valsiId, 'edit')
      isSubscribed.value = true
    }
  } catch (err) {
    error.value = isSubscribed.value
      ? t('components.subscriptionControls.unsubscribeError')
      : t('components.subscriptionControls.subscribeError')
    console.error(err)
  } finally {
    isLoading.value = false
  }
}

onMounted(fetchState)
</script>

