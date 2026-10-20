<template>
  <LingoLayout>
    <LingoFeedLayout>
      <div class="flex w-full flex-col items-center">
        <div class="flex h-20 w-20 items-center justify-center rounded-2xl bg-green-100">
          <ShoppingBag class="h-10 w-10 text-green-600" />
        </div>

        <h1 class="my-6 text-center text-2xl font-bold text-neutral-800">{{ t('lingo.shop') }}</h1>
        <p class="mb-6 text-center text-lg text-slate-500">{{ t('lingo.shopDescription') }}</p>

        <ul class="w-full max-w-lg">
          <li class="flex w-full items-center gap-x-4 border-t-2 border-slate-100 p-4">
            <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-lg bg-rose-100">
              <Heart class="h-7 w-7 fill-rose-500 text-rose-500" />
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-base font-bold text-neutral-700 lg:text-xl">
                {{ t('lingo.refillHearts') }}
              </p>
            </div>
            <button
              type="button"
              class="lingo-btn lingo-btn--default lingo-btn--sm !normal-case !tracking-normal"
              :disabled="refillDisabled"
              :aria-disabled="refillDisabled"
              @click="onRefill"
            >
              <span v-if="heartsFull">{{ t('lingo.full') }}</span>
              <span v-else class="inline-flex items-center gap-1.5">
                <Sparkles class="h-4 w-4 text-orange-500" />
                {{ pointsToRefill }}
              </span>
            </button>
          </li>

          <li class="flex w-full items-center gap-x-4 border-t-2 border-slate-100 p-4 pt-8">
            <div
              class="flex h-14 w-14 shrink-0 items-center justify-center rounded-lg bg-indigo-100"
            >
              <InfinityIcon class="h-7 w-7 text-indigo-500" />
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-base font-bold text-neutral-700 lg:text-xl">
                {{ t('lingo.unlimitedHearts') }}
              </p>
            </div>
            <button
              type="button"
              class="lingo-btn lingo-btn--primary lingo-btn--sm !normal-case !tracking-normal"
              @click="onUpgrade"
            >
              {{ isPro ? t('lingo.settings') : t('lingo.upgrade') }}
            </button>
          </li>
        </ul>

        <p v-if="shopMessage" class="mt-4 text-center text-sm text-green-700" role="status">
          {{ shopMessage }}
        </p>
      </div>
    </LingoFeedLayout>
  </LingoLayout>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { ShoppingBag, Heart, Sparkles, Infinity as InfinityIcon } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import LingoLayout from '@/components/LingoLayout.vue'
import LingoFeedLayout from '@/components/LingoFeedLayout.vue'
import { useLingoProgress } from '@/composables/useLingoProgress'
import { useSeoHead } from '@/composables/useSeoHead'

const { t } = useI18n()
const { hearts, points, isPro, refillHearts, enablePro, disablePro, maxHearts, pointsToRefill } =
  useLingoProgress()

const shopMessage = ref('')

const heartsFull = computed(() => isPro.value || hearts.value >= maxHearts)
const refillDisabled = computed(
  () => heartsFull.value || points.value < pointsToRefill || isPro.value
)

function onRefill() {
  shopMessage.value = ''
  if (refillHearts()) {
    shopMessage.value = t('lingo.heartsRefilled')
  } else {
    shopMessage.value = t('lingo.cannotRefill')
  }
}

function onUpgrade() {
  shopMessage.value = ''
  if (isPro.value) {
    disablePro()
    shopMessage.value = t('lingo.proDisabled')
  } else {
    enablePro()
    shopMessage.value = t('lingo.proEnabled')
  }
}

useSeoHead({ title: t('lingo.shop') })
</script>
