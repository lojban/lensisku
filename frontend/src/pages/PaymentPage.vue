<template>
  <div class="max-w-4xl mx-auto px-4 py-8">
    <div class="bg-white p-6 rounded-lg shadow-md">
      <h1 class="text-2xl font-bold mb-4">{{ t('payment.title') }}</h1>
      <!-- Current Balance -->
      <div class="mb-6 p-4 bg-gray-50 rounded-lg">
        <h2 class="text-lg font-semibold mb-2">{{ t('payment.currentBalance') }}</h2>

        <p class="text-2xl">${{ (balance / 100).toFixed(2) }}</p>
      </div>
      <!-- Payment Form -->
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">{{
            t('payment.amountLabel')
          }}</label>
          <input
            v-model="amount"
            type="number"
            min="1"
            step="0.01"
            class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
            :placeholder="t('payment.amountPlaceholder')"
          />
          <p class="text-sm text-gray-500 mt-1">{{ t('payment.minAmount') }}</p>
        </div>
        <!-- Success Message -->
        <div v-if="success" class="text-green-600 text-sm">{{ t('payment.paymentSuccess') }}</div>
        <!-- Paypal Button Container -->
        <div id="paypal-button-container" class="w-full" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { loadScript } from '@paypal/paypal-js'
import { ref, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { getBalance } from '@/api'
import { useError } from '@/composables/useError'
import { useSeoHead } from '@/composables/useSeoHead'

const { t } = useI18n()

// State
const amount = ref(1.0)
const balance = ref(0)
const { showError } = useError()
const success = ref('')
const paypalLoaded = ref(false)

// Methods
const fetchBalance = async () => {
  try {
    const response = await getBalance()
    balance.value = response.data.balance_cents
  } catch (err) {
    console.error('Failed to fetch balance:', err)
    showError(t('payment.loadBalanceError'))
  }
}

const initializePayPal = async () => {
  try {
    const paypal = await loadScript({
      clientId: import.meta.env.VITE_PAYPAL_CLIENT_ID,
      currency: 'USD',
      disableFunding: 'card,venmo',
      enableFunding: 'paylater',
    })

    if (paypal.Buttons) {
      paypal
        .Buttons({
          style: {
            layout: 'vertical',
            color: 'blue',
            shape: 'rect',
            label: 'paypal',
          },
          createOrder: (data, actions) => {
            return actions.order.create({
              intent: 'CAPTURE',
              purchase_units: [
                {
                  amount: {
                    currency_code: 'USD',
                    value: amount.value.toFixed(2),
                    breakdown: {
                      item_total: {
                        currency_code: 'USD',
                        value: amount.value.toFixed(2),
                      },
                    },
                  },
                },
              ],
            })
          },
          onApprove: async (_data, _actions) => {
            try {
              // Handle successful payment
              success.value = t('payment.paymentSuccess')
              await fetchBalance() // Refresh balance

              // You might want to call your backend to verify the payment
            } catch (err) {
              console.error('Payment error:', err)
              showError(t('payment.paymentError'))
            }
          },
          onError: (err) => {
            console.error('PayPal error:', err)
            showError(t('payment.paypalError'))
          },
        })
        .render('#paypal-button-container')

      paypalLoaded.value = true
    }
  } catch (err) {
    console.error('Failed to load PayPal SDK:', err)
    showError(t('payment.loadSDKError'))
  }
}

// Watch amount changes to reinitialize PayPal buttons
watch(amount, (newVal) => {
  if (paypalLoaded.value && newVal >= 1) {
    initializePayPal()
  }
})

useSeoHead({ title: t('payment.title'), robots: 'noindex, nofollow' })

onMounted(() => {
  fetchBalance()
  initializePayPal()
})
</script>

<style scoped>
#paypal-button-container {
  min-height: 200px;
  margin: 20px 0;
}
</style>
