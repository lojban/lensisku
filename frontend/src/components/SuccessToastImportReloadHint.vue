<template>
  <div class="mt-2 flex flex-col gap-2 border-t border-green-300/80 pt-2 text-sm font-normal text-green-900">
    <p class="leading-snug">
      {{ t('collectionCustomTextBulk.importDiscardHint') }}
    </p>
    <button
      type="button"
      class="ui-btn--neutral-muted inline-flex w-full shrink-0 items-center justify-center gap-2 px-3 py-2 text-sm sm:w-auto"
      @click="reload"
    >
      <RefreshCw class="h-4 w-4 shrink-0" aria-hidden="true" />
      {{ t('collectionCustomTextBulk.importReloadPage') }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { RefreshCw } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

function reload() {
  globalThis.location.reload()
}
</script>
